use std::fs;
use serde::{Deserialize, Serialize};
use fastnbt::from_bytes;
use flate2::read::GzDecoder;
use std::io::Read;

#[derive(Serialize)]
pub struct ServerItem {
    pub name: String,
    pub ip: String,
}

#[derive(Serialize)]
pub struct WorldItem {
    pub folder_name: String,
    pub name: String,
    pub last_played: i64,
}

#[derive(Deserialize)]
struct ServersDat {
    servers: Option<Vec<ServerEntry>>,
}

#[derive(Deserialize)]
struct ServerEntry {
    name: Option<String>,
    ip: Option<String>,
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
                result.push(ServerItem { name, ip });
            }
        }
    }
    
    Ok(result)
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
