use reqwest::Client;
use serde_json::json;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};



#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
    pub mc_token: String,
    pub refresh_token: String,
}

const CLIENT_ID: &str = "00000000402b5328";

#[tauri::command]
pub async fn start_microsoft_login(app: AppHandle) -> Result<MinecraftProfile, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let tx = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));

    let login_url = format!("https://login.live.com/oauth20_authorize.srf?client_id={}&response_type=code&redirect_uri=https://login.live.com/oauth20_desktop.srf&scope=XboxLive.signin%20offline_access", CLIENT_ID);

    let window_tx = tx.clone();
    let tx_for_poll = window_tx.clone();
    
    let window = WebviewWindowBuilder::new(&app, "ms_login", WebviewUrl::External(login_url.parse().unwrap()))
        .title("Microsoft Login")
        .inner_size(500.0, 600.0)
        .on_navigation(move |url| {
            if url.as_str().starts_with("https://login.live.com/oauth20_desktop.srf") {
                let mut code = String::new();
                for (key, value) in url.query_pairs() {
                    if key == "code" {
                        code = value.to_string();
                    }
                }
                if !code.is_empty() {
                    if let Ok(mut lock) = window_tx.lock() {
                        if let Some(tx) = lock.take() {
                            let _ = tx.send(code);
                        }
                    }
                    return false;
                }
            }
            true
        })
        .build()
        .map_err(|e| e.to_string())?;

    let window_for_poll = window.clone();
    
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            match window_for_poll.url() {
                Ok(url) => {
                    if url.as_str().starts_with("https://login.live.com/oauth20_desktop.srf") {
                        let mut code = String::new();
                        for (key, value) in url.query_pairs() {
                            if key == "code" {
                                code = value.to_string();
                            }
                        }
                        if !code.is_empty() {
                            if let Ok(mut lock) = tx_for_poll.lock() {
                                if let Some(tx) = lock.take() {
                                    let _ = tx.send(code);
                                }
                            }
                            break;
                        }
                    }
                }
                Err(_) => {
                    // Window was closed
                    break;
                }
            }
        }
    });

    let code = rx.await.map_err(|_| "Login window closed".to_string())?;
    
    let _ = window.close();

    login_with_code(&code).await
}

#[tauri::command]
pub async fn cancel_microsoft_login(app: AppHandle) -> Result<(), String> {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("ms_login") {
        let _ = window.close();
    }
    Ok(())
}

pub async fn login_with_code(code: &str) -> Result<MinecraftProfile, String> {
    let client = Client::new();

    // 1. Get MS Token
    let res = client.post("https://login.live.com/oauth20_token.srf")
        .form(&[
            ("client_id", CLIENT_ID),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", "https://login.live.com/oauth20_desktop.srf"),
        ])
        .send().await.map_err(|e| e.to_string())?;
        
    let ms_auth: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let access_token = match ms_auth["access_token"].as_str() {
        Some(token) => token,
        None => {
            return Err(format!("No access token. MS Response: {}", ms_auth.to_string()));
        }
    };
    let refresh_token = ms_auth["refresh_token"].as_str().unwrap_or("").to_string();

    // 2. XBL Auth
    let xbl_payload = json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={}", access_token)
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });
    let res = client.post("https://user.auth.xboxlive.com/user/authenticate")
        .json(&xbl_payload)
        .send().await.map_err(|e| e.to_string())?;
    let xbl_auth: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let xbl_token = xbl_auth["Token"].as_str().ok_or("No XBL token")?;
    let uhs = xbl_auth["DisplayClaims"]["xui"][0]["uhs"].as_str().ok_or("No uhs")?;

    // 3. XSTS Auth
    let xsts_payload = json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbl_token]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT"
    });
    let res = client.post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .json(&xsts_payload)
        .send().await.map_err(|e| e.to_string())?;
    let xsts_auth: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    if xsts_auth.get("XErr").is_some() {
        return Err("XSTS Error - Account might not have Minecraft or requires child approval".into());
    }
    let xsts_token = xsts_auth["Token"].as_str().ok_or("No XSTS token")?;

    // 4. Minecraft Token
    let mc_payload = json!({
        "identityToken": format!("XBL3.0 x={};{}", uhs, xsts_token)
    });
    let res = client.post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&mc_payload)
        .send().await.map_err(|e| e.to_string())?;
    let mc_auth: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let mc_token = mc_auth["access_token"].as_str().ok_or("No Minecraft token")?;

    // 5. Minecraft Profile
    let res = client.get("https://api.minecraftservices.com/minecraft/profile")
        .header("Authorization", format!("Bearer {}", mc_token))
        .send().await.map_err(|e| e.to_string())?;
    let profile: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let id = profile["id"].as_str().ok_or("No Minecraft ID (You probably don't own the game)")?;
    let name = profile["name"].as_str().ok_or("No Minecraft Name")?;

    Ok(MinecraftProfile {
        id: id.to_string(),
        name: name.to_string(),
        mc_token: mc_token.to_string(),
        refresh_token,
    })
}
