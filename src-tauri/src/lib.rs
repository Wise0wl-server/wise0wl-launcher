// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use once_cell::sync::Lazy;
use tauri::{Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri::Emitter;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use std::fs;
use uuid;

mod minecraft;
mod modpack;
mod settings;
mod java;
mod downloader;

use minecraft::MinecraftLauncher;
use modpack::Modpack;
use settings::Settings;

const MICROSOFT_CLIENT_ID: &str = "6ff71649-4b80-4545-8552-435c570bd6e8";

fn check_oauth_credentials() -> Result<(), String> {
    if MICROSOFT_CLIENT_ID == "YOUR_CLIENT_ID_HERE" {
        return Err("Microsoft OAuth credentials not configured. Please update MICROSOFT_CLIENT_ID in src-tauri/src/lib.rs".to_string());
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LaunchOptions {
    #[serde(rename = "modpackId")]
    modpack_id: String,
    #[serde(rename = "gameDir")]
    game_dir: PathBuf,
    #[serde(rename = "javaPath")]
    java_path: Option<String>,
    #[serde(rename = "maxMemory")]
    max_memory: Option<u32>,
    #[serde(rename = "minMemory")]
    min_memory: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
    #[serde(rename = "accessToken")]
    access_token: Option<String>,
    uuid: Option<String>,
    username: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftVersionRequest {
    #[serde(rename = "minecraftVersion")]
    minecraft_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthToken {
    access_token: String,
    client_token: String,
    uuid: String,
    name: String,
    expires_at: u64,
}

#[derive(Debug)]
struct XboxLiveAuthResponse {
    token: String,
    user_hash: String,
}

// Global auth token storage
static AUTH_TOKENS: Lazy<Arc<Mutex<HashMap<String, AuthToken>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

// Persistent PKCE code verifier storage
fn get_pkce_file_path() -> PathBuf {
    dirs::config_dir()
        .expect("Failed to get config directory")
        .join("wise0wl-launcher")
        .join("pkce.json")
}

fn load_pkce_map() -> HashMap<String, String> {
    let pkce_path = get_pkce_file_path();
    if pkce_path.exists() {
        if let Ok(content) = fs::read_to_string(&pkce_path) {
            if let Ok(map) = serde_json::from_str(&content) {
                return map;
            }
        }
    }
    HashMap::new()
}

fn save_pkce_map(map: &HashMap<String, String>) -> Result<(), String> {
    let pkce_path = get_pkce_file_path();
    if let Some(parent) = pkce_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create PKCE directory: {}", e))?;
    }
    let content = serde_json::to_string_pretty(map)
        .map_err(|e| format!("Failed to serialize PKCE map: {}", e))?;
    fs::write(&pkce_path, content)
        .map_err(|e| format!("Failed to write PKCE file: {}", e))
}

fn add_pkce_verifier(state: &str, verifier: &str) -> Result<(), String> {
    let mut map = load_pkce_map();
    map.insert(state.to_string(), verifier.to_string());
    save_pkce_map(&map)
}

fn get_and_remove_pkce_verifier(state: &str) -> Option<String> {
    let mut map = load_pkce_map();
    let verifier = map.remove(state);
    let _ = save_pkce_map(&map);
    verifier
}

// Token storage functions
fn get_tokens_file_path() -> PathBuf {
    dirs::config_dir()
        .expect("Failed to get config directory")
        .join("wise0wl-launcher")
        .join("tokens.json")
}

fn load_tokens_from_file() -> HashMap<String, AuthToken> {
    let tokens_path = get_tokens_file_path();
    if tokens_path.exists() {
        if let Ok(content) = fs::read_to_string(&tokens_path) {
            if let Ok(tokens) = serde_json::from_str(&content) {
                return tokens;
            }
        }
    }
    HashMap::new()
}

fn save_tokens_to_file(tokens: &HashMap<String, AuthToken>) -> Result<(), String> {
    let tokens_path = get_tokens_file_path();
    if let Some(parent) = tokens_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create tokens directory: {}", e))?;
    }
    
    let content = serde_json::to_string_pretty(tokens)
        .map_err(|e| format!("Failed to serialize tokens: {}", e))?;
    
    fs::write(&tokens_path, content)
        .map_err(|e| format!("Failed to write tokens file: {}", e))
}

// Initialize tokens from file on startup
fn initialize_tokens() {
    let tokens = load_tokens_from_file();
    let mut stored_tokens = AUTH_TOKENS.lock().unwrap();
    *stored_tokens = tokens;
    println!("Loaded {} tokens from storage", stored_tokens.len());
}

// PKCE helper functions
fn generate_code_verifier() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen_range(33..127)).collect();
    URL_SAFE_NO_PAD.encode(&bytes)
        .trim_end_matches('=')
        .to_string()
}

fn generate_code_challenge(verifier: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let result = hasher.finalize();
    URL_SAFE_NO_PAD.encode(&result)
        .trim_end_matches('=')
        .to_string()
}

// Commands
#[tauri::command]
async fn get_modpacks() -> Result<Vec<Modpack>, String> {
    modpack::get_all_modpacks().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_settings() -> Result<Settings, String> {
    settings::load_settings().map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_settings(settings: Settings) -> Result<(), String> {
    settings::save_settings(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
async fn launch_minecraft(options: LaunchOptions) -> Result<(), String> {
    let launcher = MinecraftLauncher::new();
    launcher.launch(options).await.map_err(|e| e.to_string())
}

fn required_java_version(minecraft_version: &str) -> u32 {
    let parts: Vec<&str> = minecraft_version.split('.').collect();
    let major = parts.get(0).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    match (major, minor) {
        (1, m) if m <= 16 => 8,
        (1, 17) => 16,
        (1, 18..=19) => 17,
        (1, 20) if minor >= 5 => 21,
        (1, 20) => 17,
        _ if major < 1 || (major == 1 && minor < 16) => 8,
        _ => 17,
    }
}

#[tauri::command]
async fn ensure_java_installed_for_mc(request: MinecraftVersionRequest) -> Result<String, String> {
    let java_version = required_java_version(&request.minecraft_version);
    java::ensure_java_installed(java_version).await
        .map(|p| p.to_string_lossy().to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MicrosoftAuthUrl {
    pub url: String,
    pub state: String,
}

#[tauri::command]
async fn get_microsoft_auth_url() -> Result<MicrosoftAuthUrl, String> {
    check_oauth_credentials()?;
    let redirect_uri = "wise0wl-oauth://callback";
    let scopes = "XboxLive.signin offline_access";
    // Generate PKCE code verifier and challenge
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);
    // Generate random state
    let state = uuid::Uuid::new_v4().to_string();
    // Store the code verifier for later use, keyed by state
    add_pkce_verifier(&state, &code_verifier)?;
    let auth_url = format!(
        "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize?\
        client_id={}&\
        response_type=code&\
        redirect_uri={}&\
        scope={}&\
        code_challenge={}&\
        code_challenge_method=S256&\
        response_mode=query&\
        state={}",
        MICROSOFT_CLIENT_ID, redirect_uri, scopes, code_challenge, state
    );
    Ok(MicrosoftAuthUrl { url: auth_url, state })
}

#[tauri::command]
async fn handle_microsoft_callback(code: String, state: String) -> Result<AuthToken, String> {
    check_oauth_credentials()?;
    println!("Received OAuth code: {}", &code[..std::cmp::min(20, code.len())]);
    // For public clients, we need to use PKCE and no client secret
    let token_url = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
    // Get the code verifier that was stored when generating the auth URL
    let code_verifier = get_and_remove_pkce_verifier(&state).ok_or("No code verifier found. Please try logging in again.")?;
    let token_params = [
        ("client_id", MICROSOFT_CLIENT_ID),
        ("code", &code),
        ("redirect_uri", "wise0wl-oauth://callback"),
        ("grant_type", "authorization_code"),
        ("code_verifier", &code_verifier),
    ];
    println!("Exchanging code for token...");
    let client = reqwest::Client::new();
    let token_resp = client.post(token_url)
        .form(&token_params)
        .send()
        .await
        .map_err(|e| format!("Failed to exchange code for token: {}", e))?;
    println!("Token response status: {}", token_resp.status());
    let token_data: serde_json::Value = token_resp.json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;
    // Check for OAuth errors
    if let Some(error) = token_data["error"].as_str() {
        let error_description = token_data["error_description"]
            .as_str()
            .unwrap_or("Unknown error");
        return Err(format!("OAuth error: {} - {}", error, error_description));
    }
    let access_token = token_data["access_token"]
        .as_str()
        .ok_or("No access token in response")?;
    println!("Successfully obtained Microsoft access token");
    // Get Xbox Live token
    let xbox_auth_response = get_xbox_live_token(access_token).await?;
    // Get Minecraft token
    let minecraft_token = get_minecraft_token(&xbox_auth_response).await?;
    // Get user profile
    let profile = get_minecraft_profile(&minecraft_token).await?;
    let auth_token = AuthToken {
        access_token: minecraft_token,
        client_token: "wise0wl-launcher".to_string(),
        uuid: profile.id.clone(),
        name: profile.name.clone(),
        expires_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600, // 1 hour
    };
    // Store the token
    {
        let mut tokens = AUTH_TOKENS.lock().unwrap();
        tokens.insert(profile.id.clone(), auth_token.clone());
        // Save to persistent storage
        if let Err(e) = save_tokens_to_file(&tokens) {
            println!("Warning: Failed to save token to persistent storage: {}", e);
        }
    }
    println!("Successfully authenticated user: {}", profile.name);
    Ok(auth_token)
}

async fn get_xbox_live_token(access_token: &str) -> Result<XboxLiveAuthResponse, String> {
    println!("Getting Xbox Live token with access token: {}", &access_token[..std::cmp::min(20, access_token.len())]);
    
    let client = reqwest::Client::new();
    let xbox_resp = client.post("https://user.auth.xboxlive.com/user/authenticate")
        .json(&serde_json::json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={}", access_token)
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to get Xbox Live token: {}", e))?;
    
    println!("Xbox Live response status: {}", xbox_resp.status());
    
    let xbox_data: serde_json::Value = xbox_resp.json()
        .await
        .map_err(|e| format!("Failed to parse Xbox Live response: {}", e))?;
    
    println!("Xbox Live response data: {:?}", xbox_data);
    
    let xbox_token = xbox_data["Token"]
        .as_str()
        .ok_or("No Xbox Live token in response")?;
    
    // Get XSTS token
    let xsts_resp = client.post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .json(&serde_json::json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [xbox_token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to get XSTS token: {}", e))?;
    
    println!("XSTS response status: {}", xsts_resp.status());
    
    let xsts_data: serde_json::Value = xsts_resp.json()
        .await
        .map_err(|e| format!("Failed to parse XSTS response: {}", e))?;
    
    println!("XSTS response data: {:?}", xsts_data);
    
    let xsts_token = xsts_data["Token"]
        .as_str()
        .ok_or("No XSTS token in response")?;
        
    let user_hash = xsts_data["DisplayClaims"]["xui"][0]["uhs"]
        .as_str()
        .ok_or("No user hash in XSTS response")?;
    
    Ok(XboxLiveAuthResponse {
        token: xsts_token.to_string(),
        user_hash: user_hash.to_string(),
    })
}

async fn get_minecraft_token(xbox_auth: &XboxLiveAuthResponse) -> Result<String, String> {
    let client = reqwest::Client::new();
    let minecraft_resp = client.post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&serde_json::json!({
            "identityToken": format!("XBL3.0 x={};{}", xbox_auth.user_hash, xbox_auth.token)
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to get Minecraft token: {}", e))?;
    
    let minecraft_data: serde_json::Value = minecraft_resp.json()
        .await
        .map_err(|e| format!("Failed to parse Minecraft response: {}", e))?;
    
    let access_token = minecraft_data["access_token"]
        .as_str()
        .ok_or("No Minecraft access token in response")?;
    
    Ok(access_token.to_string())
}

#[derive(Deserialize)]
struct MinecraftProfile {
    id: String,
    name: String,
}

async fn get_minecraft_profile(access_token: &str) -> Result<MinecraftProfile, String> {
    let client = reqwest::Client::new();
    let profile_resp = client.get("https://api.minecraftservices.com/minecraft/profile")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("Failed to get Minecraft profile: {}", e))?;
    
    let profile: MinecraftProfile = profile_resp.json()
        .await
        .map_err(|e| format!("Failed to parse profile response: {}", e))?;
    
    Ok(profile)
}

#[tauri::command]
async fn get_auth_token(uuid: String) -> Result<Option<AuthToken>, String> {
    // First, get the token and check expiration
    let token = {
        let tokens = AUTH_TOKENS.lock().unwrap();
        if let Some(token) = tokens.get(&uuid) {
            // Check if token is expired
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if current_time >= token.expires_at {
                // Token is expired, remove it
                drop(tokens); // Release the lock
                let mut tokens = AUTH_TOKENS.lock().unwrap();
                tokens.remove(&uuid);
                // Save changes to persistent storage
                if let Err(e) = save_tokens_to_file(&tokens) {
                    println!("Warning: Failed to save token removal to persistent storage: {}", e);
                }
                return Ok(None);
            }
            
            Some(token.clone())
        } else {
            None
        }
    };
    
    // If we have a token, validate it with Minecraft services
    if let Some(token) = token {
        if let Err(_) = validate_minecraft_token(&token.access_token).await {
            // Token is invalid, remove it
            let mut tokens = AUTH_TOKENS.lock().unwrap();
            tokens.remove(&uuid);
            // Save changes to persistent storage
            if let Err(e) = save_tokens_to_file(&tokens) {
                println!("Warning: Failed to save invalid token removal to persistent storage: {}", e);
            }
            return Ok(None);
        }
        
        Ok(Some(token))
    } else {
        Ok(None)
    }
}

async fn validate_minecraft_token(access_token: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let profile_resp = client.get("https://api.minecraftservices.com/minecraft/profile")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("Failed to validate token: {}", e))?;
    
    if !profile_resp.status().is_success() {
        return Err("Token validation failed".to_string());
    }
    
    Ok(())
}

#[tauri::command]
async fn logout_user(uuid: String) -> Result<(), String> {
    let mut tokens = AUTH_TOKENS.lock().unwrap();
    tokens.remove(&uuid);
    // Save changes to persistent storage
    save_tokens_to_file(&tokens)?;
    println!("Logged out user with UUID: {}", uuid);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tokens from persistent storage
    initialize_tokens();
    
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            println!("New instance opened with args: {:?}, cwd: {}", argv, cwd);
            // We got a deep link, let's focus the main window
            if let Some(main_window) = app.get_webview_window("main") {
                main_window.set_focus().unwrap();
            }
        }));
    }

    builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                let urls = event.urls();
                println!("Deep link opened: {:?}", urls);
                if let Some(url) = urls.first() {
                     app_handle.emit("oauth_callback", vec![url.to_string()]).unwrap();
                }
            });

            // On Windows and Linux, we need to register the scheme for development
            #[cfg(any(windows, target_os = "linux"))]
            {
                 app.deep_link().register_all()?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_modpacks,
            get_settings,
            save_settings,
            launch_minecraft,
            ensure_java_installed_for_mc,
            downloader::download_modpack_with_groups,
            downloader::fetch_modpack_list,
            get_microsoft_auth_url,
            handle_microsoft_callback,
            get_auth_token,
            logout_user,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
