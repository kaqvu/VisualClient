use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use std::fs;
use futures_util::{Stream, StreamExt};
use reqwest::Client;

#[derive(Serialize, Clone)]
struct ProgressPayload {
    task: String,
    progress: u8,
}

#[derive(Deserialize)]
struct VersionManifest {
    versions: Vec<VersionEntry>,
}

#[derive(Deserialize)]
struct VersionEntry {
    id: String,
    url: String,
}

#[derive(Deserialize)]
struct VersionJson {
    downloads: Downloads,
    libraries: Vec<Library>,
    #[serde(rename = "assetIndex")]
    asset_index: AssetIndex,
}

#[derive(Deserialize)]
struct Downloads {
    client: DownloadArtifact,
}

#[derive(Deserialize)]
struct DownloadArtifact {
    url: String,
}

#[derive(Deserialize)]
struct Library {
    downloads: LibraryDownloads,
    rules: Option<Vec<Rule>>,
}

#[derive(Deserialize)]
struct LibraryDownloads {
    artifact: Option<Artifact>,
    classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Deserialize)]
struct Artifact {
    path: String,
    url: String,
}

#[derive(Deserialize)]
struct Rule {
    action: String,
    os: Option<OsRule>,
}

#[derive(Deserialize)]
struct OsRule {
    name: String,
}

#[derive(Deserialize)]
struct AssetIndex {
    id: String,
    url: String,
}

#[derive(Deserialize)]
struct AssetIndexJson {
    objects: HashMap<String, AssetObject>,
}

#[derive(Deserialize)]
struct AssetObject {
    hash: String,
}

pub async fn download_minecraft(version: &str, mc_dir: &PathBuf, app: &AppHandle) -> Result<(), String> {
    let client = Client::new();

    let manifest_url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    let manifest: VersionManifest = client.get(manifest_url)
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;

    let version_entry = manifest.versions.iter().find(|v| v.id == version)
        .ok_or_else(|| format!("Version {} not found", version))?;

    let version_json_str = client.get(&version_entry.url)
        .send().await.map_err(|e| e.to_string())?
        .text().await.map_err(|e| e.to_string())?;

    let version_json: VersionJson = serde_json::from_str(&version_json_str).map_err(|e| e.to_string())?;

    let version_dir = mc_dir.join("versions").join(version);
    fs::create_dir_all(&version_dir).map_err(|e| e.to_string())?;
    fs::write(version_dir.join(format!("{}.json", version)), version_json_str).map_err(|e| e.to_string())?;

    let jar_path = version_dir.join(format!("{}.jar", version));
    if !jar_path.exists() {
        download_file(&client, &version_json.downloads.client.url, &jar_path, app, &format!("Downloading {} jar", version)).await?;
    }

    let os_name = if cfg!(target_os = "windows") { "windows" }
                  else if cfg!(target_os = "macos") { "osx" }
                  else { "linux" };

    let mut library_urls = Vec::new();
    for lib in version_json.libraries {
        let mut allowed = true;
        if let Some(rules) = lib.rules {
            allowed = false;
            for rule in rules {
                if rule.action == "allow" {
                    if let Some(os) = &rule.os {
                        if os.name == os_name {
                            allowed = true;
                        }
                    } else {
                        allowed = true;
                    }
                } else if rule.action == "disallow" {
                    if let Some(os) = &rule.os {
                        if os.name == os_name {
                            allowed = false;
                        }
                    } else {
                        allowed = false;
                    }
                }
            }
        }
        if !allowed {
            continue;
        }

        if let Some(artifact) = lib.downloads.artifact {
            library_urls.push((artifact.url, mc_dir.join("libraries").join(artifact.path)));
        }
        
        let natives_classifier = match os_name {
            "windows" => "natives-windows",
            "osx" => "natives-macos",
            "linux" => "natives-linux",
            _ => "",
        };

        if let Some(classifiers) = lib.downloads.classifiers {
            if let Some(artifact) = classifiers.get(natives_classifier) {
                library_urls.push((artifact.url.clone(), mc_dir.join("libraries").join(&artifact.path)));
            }
        }
    }

    let total_libs = library_urls.len();
    for (i, (url, path)) in library_urls.into_iter().enumerate() {
        if !path.exists() && !url.is_empty() {
            let progress = ((i as f64 / total_libs as f64) * 100.0) as u8;
            let _ = app.emit("download_progress", ProgressPayload {
                task: format!("Downloading libraries ({}/{})", i + 1, total_libs),
                progress,
            });
            download_file_silent(&client, &url, &path).await?;
        }
    }

    let asset_index_path = mc_dir.join("assets").join("indexes").join(format!("{}.json", version_json.asset_index.id));
    if let Some(parent) = asset_index_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    
    let asset_index_str = client.get(&version_json.asset_index.url)
        .send().await.map_err(|e| e.to_string())?
        .text().await.map_err(|e| e.to_string())?;
        
    fs::write(&asset_index_path, &asset_index_str).map_err(|e| e.to_string())?;
    
    let asset_index: AssetIndexJson = serde_json::from_str(&asset_index_str).map_err(|e| e.to_string())?;
    
    let mut asset_urls = Vec::new();
    for (_key, object) in asset_index.objects {
        let hash = object.hash;
        let prefix = &hash[0..2];
        let url = format!("https://resources.download.minecraft.net/{}/{}", prefix, hash);
        let path = mc_dir.join("assets").join("objects").join(prefix).join(&hash);
        asset_urls.push((url, path));
    }

    let mut tasks = Vec::new();
    for (i, (url, path)) in asset_urls.into_iter().enumerate() {
        if !path.exists() {
            let client = client.clone();
            tasks.push(async move {
                let _ = download_file_silent(&client, &url, &path).await;
                i
            });
        }
    }
    
    let mut stream = futures_util::stream::iter(tasks).buffer_unordered(20);
    let mut completed = 0;
    let total_tasks = stream.size_hint().1.unwrap_or(0);
    
    if total_tasks > 0 {
        while let Some(_res) = stream.next().await {
            completed += 1;
            let progress = ((completed as f64 / total_tasks as f64) * 100.0) as u8;
            if completed % 50 == 0 || completed == total_tasks {
                let _ = app.emit("download_progress", ProgressPayload {
                    task: format!("Downloading assets ({}/{})", completed, total_tasks),
                    progress,
                });
            }
        }
    }

    Ok(())
}

async fn download_file(client: &Client, url: &str, path: &PathBuf, app: &AppHandle, task_name: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let res = client.get(url).send().await.map_err(|e| e.to_string())?;
    let total_size = res.content_length().unwrap_or(0);
    let mut file = fs::File::create(path).map_err(|e| e.to_string())?;
    
    let mut stream = res.bytes_stream();
    let mut downloaded: u64 = 0;
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        use std::io::Write;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        
        if total_size > 0 {
            let progress = ((downloaded as f64 / total_size as f64) * 100.0) as u8;
            let _ = app.emit("download_progress", ProgressPayload {
                task: task_name.to_string(),
                progress,
            });
        }
    }
    Ok(())
}

pub async fn download_fabric(version: &str, mc_dir: &PathBuf, app: &AppHandle) -> Result<(), String> {
    let client = Client::new();
    
    let loaders_url = format!("https://meta.fabricmc.net/v2/versions/loader/{}", version);
    let loaders_res = client.get(&loaders_url).send().await.map_err(|e| e.to_string())?;
    let loaders: serde_json::Value = loaders_res.json().await.map_err(|e| e.to_string())?;
    
    if loaders.as_array().map_or(true, |arr| arr.is_empty()) {
        return Err("No fabric loader found for this version".to_string());
    }
    let loader_version = loaders[0]["loader"]["version"].as_str().ok_or("No fabric loader version found")?;
    
    let profile_url = format!("https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json", version, loader_version);
    let profile_str = client.get(&profile_url).send().await.map_err(|e| e.to_string())?.text().await.map_err(|e| e.to_string())?;
    
    let fabric_version_id = format!("fabric-{}", version);
    let version_dir = mc_dir.join("versions").join(&fabric_version_id);
    fs::create_dir_all(&version_dir).map_err(|e| e.to_string())?;
    fs::write(version_dir.join(format!("{}.json", fabric_version_id)), profile_str.clone()).map_err(|e| e.to_string())?;
    
    let profile: serde_json::Value = serde_json::from_str(&profile_str).map_err(|e| e.to_string())?;
    if let Some(libraries) = profile["libraries"].as_array() {
        for (i, lib) in libraries.iter().enumerate() {
            let name = lib["name"].as_str().unwrap_or("");
            let url = lib["url"].as_str().unwrap_or("");
            if name.is_empty() { continue; }
            
            let parts: Vec<&str> = name.split(':').collect();
            if parts.len() >= 3 {
                let group = parts[0].replace('.', "/");
                let artifact = parts[1];
                let lib_version = parts[2];
                let jar_name = format!("{}-{}.jar", artifact, lib_version);
                let path = mc_dir.join("libraries").join(&group).join(artifact).join(lib_version).join(&jar_name);
                
                if !path.exists() {
                    let dl_url = format!("{}{}/{}/{}/{}", url, group, artifact, lib_version, jar_name);
                    let progress = ((i as f64 / libraries.len() as f64) * 100.0) as u8;
                    let _ = app.emit("download_progress", ProgressPayload {
                        task: format!("Downloading Fabric libraries ({}/{})", i + 1, libraries.len()),
                        progress,
                    });
                    download_file_silent(&client, &dl_url, &path).await?;
                }
            }
        }
    }
    
    Ok(())
}

pub fn get_launch_info(version: &str, loader: &str, mc_dir: &PathBuf) -> Result<(String, String, String), String> {
    let version_json_path = mc_dir.join("versions").join(version).join(format!("{}.json", version));
    let version_json_str = fs::read_to_string(&version_json_path)
        .map_err(|e| format!("Could not read version JSON: {}", e))?;
    let version_json: VersionJson = serde_json::from_str(&version_json_str).map_err(|e| e.to_string())?;

    let os_name = if cfg!(target_os = "windows") { "windows" }
                  else if cfg!(target_os = "macos") { "osx" }
                  else { "linux" };

    let mut cp_entries = Vec::new();

    for lib in version_json.libraries {
        let mut allowed = true;
        if let Some(rules) = lib.rules {
            allowed = false;
            for rule in rules {
                if rule.action == "allow" {
                    if let Some(os) = &rule.os {
                        if os.name == os_name { allowed = true; }
                    } else { allowed = true; }
                } else if rule.action == "disallow" {
                    if let Some(os) = &rule.os {
                        if os.name == os_name { allowed = false; }
                    } else { allowed = false; }
                }
            }
        }
        if !allowed { continue; }

        if let Some(artifact) = lib.downloads.artifact {
            let path = mc_dir.join("libraries").join(&artifact.path);
            cp_entries.push(path.to_string_lossy().into_owned());
        }
    }

    let jar_path = mc_dir.join("versions").join(version).join(format!("{}.jar", version));
    cp_entries.push(jar_path.to_string_lossy().into_owned());

    let mut main_class = "net.minecraft.client.main.Main".to_string();

    if loader == "fabric" {
        let fabric_version_id = format!("fabric-{}", version);
        let fabric_json_path = mc_dir.join("versions").join(&fabric_version_id).join(format!("{}.json", fabric_version_id));
        if let Ok(fabric_json_str) = fs::read_to_string(&fabric_json_path) {
            if let Ok(fabric_json) = serde_json::from_str::<serde_json::Value>(&fabric_json_str) {
                if let Some(mc) = fabric_json["mainClass"].as_str() {
                    main_class = mc.to_string();
                }
                
                if let Some(libraries) = fabric_json["libraries"].as_array() {
                    for lib in libraries {
                        let name = lib["name"].as_str().unwrap_or("");
                        let parts: Vec<&str> = name.split(':').collect();
                        if parts.len() >= 3 {
                            let group = parts[0].replace('.', "/");
                            let artifact = parts[1];
                            let lib_version = parts[2];
                            let jar_name = format!("{}-{}.jar", artifact, lib_version);
                            let path = mc_dir.join("libraries").join(&group).join(artifact).join(lib_version).join(&jar_name);
                            cp_entries.push(path.to_string_lossy().into_owned());
                        }
                    }
                }
            }
        }
    }

    let separator = if cfg!(target_os = "windows") { ";" } else { ":" };
    Ok((cp_entries.join(separator), version_json.asset_index.id, main_class))
}

async fn download_file_silent(client: &Client, url: &str, path: &PathBuf) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let res = client.get(url).send().await.map_err(|e| e.to_string())?;
    let bytes = res.bytes().await.map_err(|e| e.to_string())?;
    fs::write(path, bytes).map_err(|e| e.to_string())?;
    Ok(())
}
