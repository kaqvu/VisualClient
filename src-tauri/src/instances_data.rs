use std::fs;
use serde::{Deserialize, Serialize};
use fastnbt::from_bytes;
use flate2::read::GzDecoder;
use std::io::Read;
use craftping::tokio::ping;

#[derive(Serialize)]
pub struct ServerItem {
    pub name: String,
    pub ip: String,
    pub accept_textures: Option<u8>,
    pub icon_base64: Option<String>,
}

#[derive(Serialize)]
pub struct WorldItem {
    pub folder_name: String,
    pub name: String,
    pub last_played: i64,
    pub icon_base64: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
struct ServersDat {
    servers: Option<Vec<ServerEntry>>,
}

#[derive(Deserialize, Serialize, Clone)]
struct ServerEntry {
    name: Option<String>,
    ip: Option<String>,
    #[serde(rename = "acceptTextures", skip_serializing_if = "Option::is_none")]
    accept_textures: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
}

#[derive(Deserialize)]
struct LevelDat {
    #[serde(rename = "Data")]
    data: LevelData,
}

#[derive(Deserialize)]
struct LevelData {
    #[serde(rename = "LevelName")]
    level_name: Option<String>,
    #[serde(rename = "LastPlayed")]
    last_played: Option<i64>,
}

#[tauri::command]
pub fn get_instance_servers(id: String) -> Result<Vec<ServerItem>, String> {
    let vc_dir = crate::launcher::get_dot_visualclient_dir();
    let profiles_dir = vc_dir.join("profiles").join(&id);
    let servers_dat = profiles_dir.join("servers.dat");
    
    if !servers_dat.exists() {
        return Ok(Vec::new());
    }
    
    let bytes = fs::read(&servers_dat).map_err(|e| e.to_string())?;
    
    let data: ServersDat = match from_bytes(&bytes) {
        Ok(d) => d,
        Err(_) => return Ok(Vec::new()),
    };
    
    let mut result = Vec::new();
    if let Some(servers) = data.servers {
        for s in servers {
            if let (Some(name), Some(ip)) = (s.name, s.ip) {
                result.push(ServerItem { 
                    name, 
                    ip, 
                    accept_textures: s.accept_textures,
                    icon_base64: s.icon 
                });
            }
        }
    }
    
    Ok(result)
}

#[tauri::command]
pub fn update_instance_server(
    id: String,
    original_ip: String,
    new_name: String,
    new_ip: String,
    accept_textures: Option<u8>
) -> Result<(), String> {
    let vc_dir = crate::launcher::get_dot_visualclient_dir();
    let profiles_dir = vc_dir.join("profiles").join(&id);
    let servers_dat = profiles_dir.join("servers.dat");
    
    let mut servers_data = if servers_dat.exists() {
        let bytes = fs::read(&servers_dat).map_err(|e| e.to_string())?;
        fastnbt::from_bytes::<ServersDat>(&bytes).unwrap_or(ServersDat { servers: Some(Vec::new()) })
    } else {
        ServersDat { servers: Some(Vec::new()) }
    };
    
    if let Some(ref mut servers) = servers_data.servers {
        for s in servers.iter_mut() {
            if let Some(ref ip) = s.ip {
                if ip == &original_ip {
                    s.name = Some(new_name);
                    s.ip = Some(new_ip);
                    s.accept_textures = accept_textures;
                    break;
                }
            }
        }
    }
    
    let out_bytes = fastnbt::to_bytes(&servers_data).map_err(|e| e.to_string())?;
    fs::write(&servers_dat, out_bytes).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn remove_instance_server(id: String, ip_to_remove: String) -> Result<(), String> {
    let vc_dir = crate::launcher::get_dot_visualclient_dir();
    let profiles_dir = vc_dir.join("profiles").join(&id);
    let servers_dat = profiles_dir.join("servers.dat");
    
    if !servers_dat.exists() {
        return Ok(());
    }
    
    let bytes = fs::read(&servers_dat).map_err(|e| e.to_string())?;
    let mut servers_data: ServersDat = fastnbt::from_bytes(&bytes).map_err(|e| e.to_string())?;
    
    if let Some(ref mut servers) = servers_data.servers {
        servers.retain(|s| {
            if let Some(ref ip) = s.ip {
                ip != &ip_to_remove
            } else {
                true
            }
        });
    }
    
    let out_bytes = fastnbt::to_bytes(&servers_data).map_err(|e| e.to_string())?;
    fs::write(&servers_dat, out_bytes).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_server_icon(id: String, ip_to_match: String, icon_base64: String) -> Result<(), String> {
    let vc_dir = crate::launcher::get_dot_visualclient_dir();
    let profiles_dir = vc_dir.join("profiles").join(&id);
    let servers_dat = profiles_dir.join("servers.dat");
    
    if !servers_dat.exists() {
        return Ok(());
    }
    
    let bytes = fs::read(&servers_dat).map_err(|e| e.to_string())?;
    let mut servers_data: ServersDat = fastnbt::from_bytes(&bytes).map_err(|e| e.to_string())?;
    
    if let Some(ref mut servers) = servers_data.servers {
        for s in servers {
            if let Some(ref current_ip) = s.ip {
                if current_ip == &ip_to_match {
                    let clean_base64 = if icon_base64.starts_with("data:image/") {
                        icon_base64.split(',').nth(1).unwrap_or(&icon_base64).to_string()
                    } else {
                        icon_base64.clone()
                    };
                    s.icon = Some(clean_base64);
                    break;
                }
            }
        }
    }
    
    let out_bytes = fastnbt::to_bytes(&servers_data).map_err(|e| e.to_string())?;
    fs::write(&servers_dat, out_bytes).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn add_instance_server(
    id: String,
    name: String,
    ip: String,
    accept_textures: Option<u8>
) -> Result<(), String> {
    let vc_dir = crate::launcher::get_dot_visualclient_dir();
    let profiles_dir = vc_dir.join("profiles").join(&id);
    let servers_dat = profiles_dir.join("servers.dat");
    
    let mut servers_data = if servers_dat.exists() {
        let bytes = fs::read(&servers_dat).map_err(|e| e.to_string())?;
        fastnbt::from_bytes::<ServersDat>(&bytes).unwrap_or(ServersDat { servers: Some(Vec::new()) })
    } else {
        ServersDat { servers: Some(Vec::new()) }
    };
    
    let new_server = ServerEntry {
        name: Some(name),
        ip: Some(ip),
        accept_textures,
        icon: None,
    };
    
    if let Some(ref mut servers) = servers_data.servers {
        servers.push(new_server);
    } else {
        servers_data.servers = Some(vec![new_server]);
    }
    
    let out_bytes = fastnbt::to_bytes(&servers_data).map_err(|e| e.to_string())?;
    fs::write(&servers_dat, out_bytes).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_instance_worlds(id: String) -> Result<Vec<WorldItem>, String> {
    let vc_dir = crate::launcher::get_dot_visualclient_dir();
    let profiles_dir = vc_dir.join("profiles").join(&id);
    let saves_dir = profiles_dir.join("saves");
    
    if !saves_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut worlds = Vec::new();
    
    if let Ok(entries) = fs::read_dir(saves_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                let level_dat = path.join("level.dat");
                if level_dat.exists() {
                    let folder_name = entry.file_name().to_string_lossy().to_string();
                    let bytes = fs::read(&level_dat).unwrap_or_default();
                    
                    let icon_path = path.join("icon.png");
                    let icon_base64 = if icon_path.exists() {
                        if let Ok(bytes) = fs::read(&icon_path) {
                            use base64::{Engine as _, engine::general_purpose::STANDARD};
                            Some(STANDARD.encode(&bytes))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let mut decoder = GzDecoder::new(&bytes[..]);
                    let mut uncompressed = Vec::new();
                    if decoder.read_to_end(&mut uncompressed).is_ok() {
                        if let Ok(dat) = from_bytes::<LevelDat>(&uncompressed) {
                            let name = dat.data.level_name.unwrap_or(folder_name.clone());
                            let last_played = dat.data.last_played.unwrap_or(0);
                            worlds.push(WorldItem {
                                folder_name,
                                name,
                                last_played,
                                icon_base64: icon_base64.clone(),
                            });
                        }
                    } else {
                        if let Ok(dat) = from_bytes::<LevelDat>(&bytes) {
                            let name = dat.data.level_name.unwrap_or(folder_name.clone());
                            let last_played = dat.data.last_played.unwrap_or(0);
                            worlds.push(WorldItem {
                                folder_name,
                                name,
                                last_played,
                                icon_base64: icon_base64.clone(),
                            });
                        }
                    }
                }
            }
        }
    }
    
    worlds.sort_by(|a, b| b.last_played.cmp(&a.last_played));
    
    Ok(worlds)
}

#[derive(Serialize)]
pub struct PingResponse {
    pub motd: String,
    pub online: bool,
    pub players_online: usize,
    pub players_max: usize,
}

fn extract_motd(desc: &Option<serde_json::Value>) -> String {
    match desc {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Object(obj)) => {
            let mut result = String::new();
            if let Some(serde_json::Value::String(text)) = obj.get("text") {
                result.push_str(text);
            }
            if let Some(serde_json::Value::Array(extra)) = obj.get("extra") {
                for item in extra {
                    if let Some(serde_json::Value::String(text)) = item.get("text") {
                        result.push_str(text);
                    }
                }
            }
            if result.is_empty() {
                // fallback serialization
                result = serde_json::to_string(obj).unwrap_or_default();
            }
            result
        },
        Some(val) => serde_json::to_string(val).unwrap_or_default(),
        None => "".to_string(),
    }
}

fn html_escape(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}

#[tauri::command]
pub async fn ping_server(ip: String) -> Result<PingResponse, String> {
    let mut host = ip.clone();
    let mut port = 25565;
    if let Some(idx) = ip.find(':') {
        let (h, p) = ip.split_at(idx);
        host = h.to_string();
        if let Ok(p_num) = p[1..].parse::<u16>() {
            port = p_num;
        }
    }
    
    let stream_result = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        tokio::net::TcpStream::connect(format!("{}:{}", host, port))
    ).await;
    
    let mut stream = match stream_result {
        Ok(Ok(s)) => s,
        _ => return Ok(PingResponse {
            motd: "".to_string(),
            online: false,
            players_online: 0,
            players_max: 0,
        }),
    };
    
    match ping(&mut stream, &host, port).await {
        Ok(res) => {
            let motd = extract_motd(&res.description);
            Ok(PingResponse {
                motd: html_escape(&motd),
                online: true,
                players_online: res.online_players,
                players_max: res.max_players,
            })
        },
        Err(_) => {
            Ok(PingResponse {
                motd: "".to_string(),
                online: false,
                players_online: 0,
                players_max: 0,
            })
        }
    }
}
