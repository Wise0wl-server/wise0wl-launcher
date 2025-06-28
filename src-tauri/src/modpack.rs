use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use dirs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mod {
    pub id: String,
    pub name: String,
    pub version: String,
    pub required: bool,
    #[serde(rename = "downloadUrl")]
    pub download_url: String,
    pub hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Modpack {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    #[serde(rename = "minecraftVersion")]
    pub minecraft_version: String,
    #[serde(rename = "forgeVersion")]
    pub forge_version: Option<String>,
    #[serde(rename = "fabricVersion")]
    pub fabric_version: Option<String>,
    #[serde(rename = "neoforgeVersion")]
    pub neoforge_version: Option<String>,
    pub image: String,
    pub mods: Vec<Mod>,
    #[serde(rename = "lastUpdated")]
    pub last_updated: String,
    pub changelog: Option<String>,
}

fn get_modpacks_path() -> PathBuf {
    dirs::config_dir()
        .expect("Failed to get config directory")
        .join("wise0wl-launcher")
        .join("modpacks.json")
}

pub fn get_all_modpacks() -> Result<Vec<Modpack>, String> {
    let modpacks_path = get_modpacks_path();

    if !modpacks_path.exists() {
        return Ok(get_default_modpacks());
    }

    fs::read_to_string(&modpacks_path)
        .map_err(|e| format!("Failed to read modpacks file: {}", e))
        .and_then(|content| {
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse modpacks: {}", e))
        })
}

fn get_default_modpacks() -> Vec<Modpack> {
    vec![
        Modpack {
            id: "vanilla".to_string(),
            name: "Vanilla".to_string(),
            description: "Minecraft Vanilla, no mods.".to_string(),
            version: "1.20.1".to_string(),
            minecraft_version: "1.20.1".to_string(),
            forge_version: None,
            fabric_version: None,
            neoforge_version: None,
            image: "/images/WOLOGO.png".to_string(),
            mods: vec![],
            last_updated: "2024-03-20".to_string(),
            changelog: None,
        },
        Modpack {
            id: "opti".to_string(),
            name: "Vanilla+".to_string(),
            description: "Enhanced Vanilla experience with OptiFine and QoL mods.".to_string(),
            version: "1.20.1".to_string(),
            minecraft_version: "1.20.1".to_string(),
            forge_version: Some("47.2.0".to_string()),
            fabric_version: None,
            neoforge_version: None,
            image: "/images/WOLOGO.png".to_string(),
            mods: vec![Mod {
                id: "optifine".to_string(),
                name: "OptiFine".to_string(),
                version: "HD_U_I7".to_string(),
                required: true,
                download_url: "https://optifine.net/download?f=OptiFine_1.20.1_HD_U_I7.jar"
                    .to_string(),
                hash: None,
            }],
            last_updated: "2024-03-20".to_string(),
            changelog: None,
        },
    ]
}
