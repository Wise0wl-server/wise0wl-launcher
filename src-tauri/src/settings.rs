use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use dirs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    #[serde(rename = "javaPath")]
    pub java_path: String,
    #[serde(rename = "maxMemory")]
    pub max_memory: u32,
    #[serde(rename = "minMemory")]
    pub min_memory: u32,
    #[serde(rename = "gameResolution")]
    pub game_resolution: GameResolution,
    #[serde(rename = "gameDirectory")]
    pub game_directory: PathBuf,
}

// Legacy settings struct for backward compatibility
#[derive(Debug, Serialize, Deserialize)]
struct LegacySettings {
    java_path: String,
    max_memory: u32,
    min_memory: u32,
    game_resolution: GameResolution,
    game_directory: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameResolution {
    pub width: u32,
    pub height: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            java_path: detect_java_path(),
            max_memory: 4096,
            min_memory: 2048,
            game_resolution: GameResolution {
                width: 1280,
                height: 720,
            },
            game_directory: default_game_directory(),
        }
    }
}

fn detect_java_path() -> String {
    // TODO: Implement proper Java detection
    #[cfg(target_os = "windows")]
    return "javaw.exe".to_string();
    #[cfg(not(target_os = "windows"))]
    return "java".to_string();
}

fn default_game_directory() -> PathBuf {
    dirs::config_dir()
        .expect("Failed to get config directory")
        .join(".minecraft-wise0wl")
}

fn get_settings_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        // On Windows: C:\Users\<User>\AppData\Local\Programs\Wise0wl\wise0wl-cmd\settings.json
        let appdata = std::env::var_os("LOCALAPPDATA")
            .expect("Failed to get LOCALAPPDATA environment variable");
        PathBuf::from(appdata)
            .join("Programs")
            .join("Wise0wl")
            .join("wise0wl-cml")
            .join("settings.json")
    }
    #[cfg(target_os = "macos")]
    {
        // On macOS: ~/Library/Application Support/Wise0wl/wise0wl-cmd/settings.json
        let home_dir = dirs::home_dir().expect("Failed to get home directory");
        home_dir
            .join("Library")
            .join("Application Support")
            .join("Wise0wl")
            .join("wise0wl-cml")
            .join("settings.json")
    }
    #[cfg(target_os = "linux")]
    {
        // On Linux: ~/.local/share/Wise0wl/wise0wl-cmd/settings.json
        let home_dir = dirs::home_dir().expect("Failed to get home directory");
        home_dir
            .join(".local")
            .join("share")
            .join("Wise0wl")
            .join("wise0wl-cml")
            .join("settings.json")
    }
}

pub fn load_settings() -> Result<Settings, String> {
    let settings_path = get_settings_path();

    if !settings_path.exists() {
        let default_settings = Settings::default();
        save_settings(&default_settings)?;
        return Ok(default_settings);
    }

    let content = fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    // Try to parse as new format first (camelCase)
    if let Ok(settings) = serde_json::from_str::<Settings>(&content) {
        return Ok(settings);
    }

    // If that fails, try to parse as legacy format (snake_case)
    if let Ok(legacy_settings) = serde_json::from_str::<LegacySettings>(&content) {
        // Convert legacy format to new format
        let settings = Settings {
            java_path: legacy_settings.java_path,
            max_memory: legacy_settings.max_memory,
            min_memory: legacy_settings.min_memory,
            game_resolution: legacy_settings.game_resolution,
            game_directory: legacy_settings.game_directory,
        };
        // Save in new format for next time
        save_settings(&settings)?;
        return Ok(settings);
    }

    Err("Failed to parse settings: invalid format".to_string())
}

pub fn save_settings(settings: &Settings) -> Result<(), String> {
    let settings_path = get_settings_path();

    // Create parent directories if they don't exist
    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create settings directory: {}", e))?;
    }

    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(&settings_path, content).map_err(|e| format!("Failed to write settings file: {}", e))
}
