use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};
use futures_util::StreamExt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub loader: String,
    pub version: String,
    pub java_path: String,
    pub icon_path: Option<String>,
}

#[derive(Serialize, Clone)]
struct ProgressPayload {
    task: String,
    progress: u8,
}

fn get_dot_visualclient_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().unwrap_or_else(|| dirs::home_dir().unwrap().join("AppData").join("Roaming")).join(".visualclient")
    }
    #[cfg(target_os = "macos")]
    {
        dirs::config_dir().unwrap_or_else(|| dirs::home_dir().unwrap().join("Library").join("Application Support")).join("visualclient")
    }
    #[cfg(target_os = "linux")]
    {
        dirs::home_dir().unwrap().join(".visualclient")
    }
}

fn get_instances_file() -> PathBuf {
    get_dot_visualclient_dir().join("launcher").join("instances.json")
}

#[tauri::command]
pub fn get_instances() -> Result<Vec<Instance>, String> {
    let file_path = get_instances_file();
    let content = crate::crypto::read_encrypted_file(&file_path)?;
    if content.is_empty() {
        return Ok(Vec::new());
    }
    let instances: Vec<Instance> = serde_json::from_str(&content).unwrap_or_default();
    Ok(instances)
}

async fn download_java(version: u32, target_dir: &PathBuf, app: &AppHandle) -> Result<PathBuf, String> {
    let extract_dir = target_dir.join(format!("jdk{}", version));
    if extract_dir.exists() {
        if let Ok(exe_path) = find_java_executable(&extract_dir) {
            if let Ok(output) = std::process::Command::new(&exe_path)
                .env_remove("JAVA_HOME")
                .env_remove("PATH")
                .env_remove("Path")
                .arg("-version")
                .output() {
                
                let output_str = String::from_utf8_lossy(&output.stderr);
                if output.status.success() && output_str.contains("version") {
                    let _ = app.emit("download_progress", ProgressPayload {
                        task: format!("Using existing Java {}", version),
                        progress: 100,
                    });
                    return Ok(exe_path);
                }
            }
        }
        let _ = fs::remove_dir_all(&extract_dir);
    }

    let os = if cfg!(target_os = "windows") { "windows" }
             else if cfg!(target_os = "macos") { "mac" }
             else { "linux" };
    let arch = if cfg!(target_arch = "x86_64") { "x64" }
               else if cfg!(target_arch = "aarch64") { "aarch64" }
               else { "x64" };

    let mut url = format!("https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jre/hotspot/normal/eclipse", version, os, arch);
    
    let mut res = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    
    if !res.status().is_success() {
        url = format!("https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jdk/hotspot/normal/eclipse", version, os, arch);
        res = reqwest::get(&url).await.map_err(|e| e.to_string())?;
        
        if !res.status().is_success() {
            url = format!("https://api.adoptium.net/v3/binary/latest/21/ga/{}/{}/jre/hotspot/normal/eclipse", os, arch);
            res = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            if !res.status().is_success() {
                return Err(format!("Failed to find Java version {} on Adoptium", version));
            }
        }
    }

    let total_size = res.content_length().unwrap_or(0);
    let is_zip = os == "windows" || res.url().path().ends_with(".zip");
    
    let tmp_file = target_dir.join(format!("java_download_{}.archive", version));
    let mut file = std::fs::File::create(&tmp_file).map_err(|e| e.to_string())?;
    
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
                task: format!("Downloading Java {}", version),
                progress,
            });
        }
    }
    
    let _ = app.emit("download_progress", ProgressPayload {
        task: format!("Extracting Java {}", version),
        progress: 100,
    });
    
    let extract_dir = target_dir.join(format!("jdk{}", version));
    if extract_dir.exists() {
        fs::remove_dir_all(&extract_dir).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&extract_dir).map_err(|e| e.to_string())?;

    if is_zip {
        let file = std::fs::File::open(&tmp_file).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = match file.enclosed_name() {
                Some(path) => {
                    let mut components = path.components();
                    components.next();
                    let stripped = components.as_path();
                    if stripped.as_os_str().is_empty() {
                        continue;
                    }
                    extract_dir.join(stripped)
                },
                None => continue,
            };
            
            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).map_err(|e| e.to_string())?;
                    }
                }
                let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
            }
        }
    } else {
        let file = std::fs::File::open(&tmp_file).map_err(|e| e.to_string())?;
        let tar = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(tar);
        for file in archive.entries().map_err(|e| e.to_string())? {
            let mut file = file.map_err(|e| e.to_string())?;
            let path = file.path().map_err(|e| e.to_string())?.into_owned();
            let mut components = path.components();
            components.next();
            let stripped = components.as_path();
            if stripped.as_os_str().is_empty() {
                continue;
            }
            let outpath = extract_dir.join(stripped);
            if file.header().entry_type().is_dir() {
                fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).map_err(|e| e.to_string())?;
                    }
                }
                file.unpack(&outpath).map_err(|e| e.to_string())?;
            }
        }
    }
    
    let _ = fs::remove_file(&tmp_file);

    find_java_executable(&extract_dir)
}

fn find_java_executable(dir: &Path) -> Result<PathBuf, String> {
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy();
        if name == "java.exe" || name == "java" {
            if entry.path().parent().map(|p| p.ends_with("bin")).unwrap_or(false) {
                return Ok(entry.path().to_path_buf());
            }
        }
    }
    Err("Could not find java executable in extracted archive".to_string())
}

#[tauri::command]
pub async fn create_instance(
    app: AppHandle,
    name: String,
    loader: String,
    version: String,
    java_version: u32,
    icon_path: Option<String>,
) -> Result<(), String> {
    let vc_dir = get_dot_visualclient_dir();
    let mut id = name.clone();
    let mut profiles_dir = vc_dir.join("profiles").join(&id);
    
    let mut i = 1;
    while profiles_dir.exists() {
        id = format!("{} ({})", name, i);
        profiles_dir = vc_dir.join("profiles").join(&id);
        i += 1;
    }
    
    fs::create_dir_all(&profiles_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(vc_dir.join("minecraft")).map_err(|e| e.to_string())?;
    
    let mut final_icon_path = None;
    if let Some(path) = icon_path {
        let ext = Path::new(&path).extension().unwrap_or_default().to_string_lossy().to_string();
        let target_icon = profiles_dir.join(format!("icon.{}", ext));
        if fs::copy(&path, &target_icon).is_ok() {
            final_icon_path = Some(target_icon.to_string_lossy().to_string());
        }
    }
    
    let java_dir = vc_dir.join("java");
    fs::create_dir_all(&java_dir).map_err(|e| e.to_string())?;
    
    let java_path = match download_java(java_version, &java_dir, &app).await {
        Ok(path) => path,
        Err(e) => {
            let _ = app.emit("download_progress", ProgressPayload { task: format!("Error: {}", e), progress: 0 });
            return Err(e);
        }
    };

    let mc_dir_clone = vc_dir.join("minecraft");
    let app_clone = app.clone();
    let version_clone = version.clone();
    let loader_clone = loader.clone();
    
    tokio::spawn(async move {
        crate::minecraft::download_minecraft(&version_clone, &mc_dir_clone, &app_clone).await?;
        if loader_clone == "fabric" {
            crate::minecraft::download_fabric(&version_clone, &mc_dir_clone, &app_clone).await?;
        }
        Ok::<(), String>(())
    }).await.map_err(|e| e.to_string())??;
    
    fs::write(profiles_dir.join("options.txt"), "").map_err(|e| e.to_string())?;
    
    let new_instance = Instance {
        id,
        name,
        loader,
        version,
        java_path: java_path.to_string_lossy().to_string(),
        icon_path: final_icon_path,
    };

    let mut instances = get_instances().unwrap_or_default();
    instances.push(new_instance);
    
    crate::crypto::write_encrypted_file(
        &get_instances_file(),
        &serde_json::to_string_pretty(&instances).map_err(|e| e.to_string())?
    )?;

    let _ = app.emit("download_progress", ProgressPayload {
        task: "Done".to_string(),
        progress: 100,
    });

    Ok(())
}

#[tauri::command]
pub fn rename_instance(id: String, new_name: String) -> Result<(), String> {
    let mut instances = get_instances().unwrap_or_default();
    if let Some(inst) = instances.iter_mut().find(|i| i.id == id) {
        inst.name = new_name;
        crate::crypto::write_encrypted_file(
            &get_instances_file(),
            &serde_json::to_string_pretty(&instances).map_err(|e| e.to_string())?
        )?;
    }
    Ok(())
}

#[tauri::command]
pub fn delete_instance(id: String) -> Result<(), String> {
    let mut instances = get_instances().unwrap_or_default();
    instances.retain(|i| i.id != id);
    crate::crypto::write_encrypted_file(
        &get_instances_file(),
        &serde_json::to_string_pretty(&instances).map_err(|e| e.to_string())?
    )?;

    let vc_dir = get_dot_visualclient_dir();
    let profiles_dir = vc_dir.join("profiles").join(&id);
    if profiles_dir.exists() {
        let _ = fs::remove_dir_all(&profiles_dir);
    }
    
    Ok(())
}

#[tauri::command]
pub fn open_instance_folder(id: String) -> Result<(), String> {
    let vc_dir = get_dot_visualclient_dir();
    let profiles_dir = vc_dir.join("profiles").join(&id);
    if !profiles_dir.exists() {
        fs::create_dir_all(&profiles_dir).map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer").arg(&profiles_dir).spawn().map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(&profiles_dir).spawn().map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(&profiles_dir).spawn().map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn launch_instance(
    app: AppHandle,
    id: String,
    username: String,
    launching_text: String,
) -> Result<(), String> {
    let _ = app.emit("download_progress", ProgressPayload {
        task: launching_text,
        progress: 100,
    });

    let instances = get_instances().unwrap_or_default();
    let instance = instances.iter().find(|i| i.id == id).ok_or("Instance not found")?.clone();

    let vc_dir = get_dot_visualclient_dir();
    let mc_dir = vc_dir.join("minecraft");
    let profiles_dir = vc_dir.join("profiles").join(&id);

    let java_path = if !instance.java_path.is_empty() {
        PathBuf::from(&instance.java_path)
    } else {
        return Err("No Java path".to_string());
    };

    let mut cmd = std::process::Command::new(&java_path);
    cmd.current_dir(&profiles_dir);

    cmd.arg("-Xmx2G");
    cmd.arg("-XX:+UnlockExperimentalVMOptions");
    cmd.arg("-XX:+UseG1GC");
    cmd.arg("--enable-native-access=ALL-UNNAMED");

    let (classpath, asset_index, main_class) = crate::minecraft::get_launch_info(&instance.version, &instance.loader, &mc_dir)?;
    cmd.arg("-cp");
    cmd.arg(classpath);
    
    cmd.arg(&main_class);

    cmd.arg("--username").arg(&username);
    cmd.arg("--version").arg(&instance.version);
    cmd.arg("--gameDir").arg(&profiles_dir);
    cmd.arg("--assetsDir").arg(mc_dir.join("assets"));
    cmd.arg("--assetIndex").arg(&asset_index);
    cmd.arg("--uuid").arg("00000000-0000-0000-0000-000000000000");
    cmd.arg("--accessToken").arg("0");
    cmd.arg("--userType").arg("msa");
    cmd.arg("--versionType").arg("release");

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    cmd.spawn().map_err(|e| format!("Failed to start game: {}", e))?;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let _ = app.emit("download_progress", ProgressPayload {
        task: "Done".to_string(),
        progress: 100,
    });

    Ok(())
}
