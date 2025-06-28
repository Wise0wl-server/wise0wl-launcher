use std::path::{Path, PathBuf};
use tauri::command;
use dirs;
use std::fs;
use std::io::Write;
use std::process::Command;
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnlineModpack {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub minecraft_version: String,
    pub modloader: String,
    pub modloader_version: String,
    pub image: String,
    pub url: String,
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModFileEntry {
    pub filename: String,
    pub url: String,
    pub dir: String,
    pub hash: Option<String>,
    pub hashformat: Option<String>,
    pub scopes: Option<Vec<String>>,
}

/// Utility: check if any of the user's groups match the scopes (or if scopes is None, allow)
fn is_in_scope(user_groups: &[String], scopes: &Option<Vec<String>>) -> bool {
    match scopes {
        Some(scope_list) => user_groups.iter().any(|g| scope_list.contains(g)),
        None => true,
    }
}

/// Fetch and filter the modpack list by user_groups
#[command]
pub async fn fetch_modpack_list(list_url: &str, user_groups: Vec<String>) -> Result<Vec<OnlineModpack>, String> {
    let resp = reqwest::get(list_url)
        .await
        .map_err(|e| format!("Failed to fetch modpack list: {}", e))?;
    let modpacks: Vec<OnlineModpack> = resp.json()
        .await
        .map_err(|e| format!("Failed to parse modpack list: {}", e))?;
    let filtered = modpacks
        .into_iter()
        .filter(|mp| is_in_scope(&user_groups, &mp.scopes))
        .collect();
    Ok(filtered)
}

/// Fetch and filter the modpack file (mods/resources) by user_groups
pub async fn fetch_modpack_file(url: &str, user_groups: &[String]) -> Result<Vec<ModFileEntry>, String> {
    let resp = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to fetch modpack file: {}", e))?;
    let files: Vec<ModFileEntry> = resp.json()
        .await
        .map_err(|e| format!("Failed to parse modpack file: {}", e))?;
    let filtered = files
        .into_iter()
        .filter(|f| is_in_scope(user_groups, &f.scopes))
        .collect();
    Ok(filtered)
}

/// Download and verify a file
async fn download_and_verify(entry: &ModFileEntry, dest_dir: &Path) -> Result<(), String> {
    let resp = reqwest::get(&entry.url)
        .await
        .map_err(|e| format!("Failed to download {}: {}", entry.filename, e))?;
    let bytes = resp.bytes()
        .await
        .map_err(|e| format!("Failed to read bytes for {}: {}", entry.filename, e))?;
    let target_dir = dest_dir.join(&entry.dir);
    fs::create_dir_all(&target_dir).map_err(|e| format!("Failed to create dir {}: {}", target_dir.display(), e))?;
    let file_path = target_dir.join(&entry.filename);
    let mut file = fs::File::create(&file_path).map_err(|e| format!("Failed to create file {}: {}", file_path.display(), e))?;
    file.write_all(&bytes).map_err(|e| format!("Failed to write file {}: {}", file_path.display(), e))?;
    // TODO: Verify hash if provided
    Ok(())
}

/// Get the latest modloader version if not specified
async fn get_latest_modloader_version(modloader: &str, mc_version: &str) -> Result<String, String> {
    match modloader.to_lowercase().as_str() {
        "fabric" => {
            // Fabric meta API: https://meta.fabricmc.net/v2/versions/loader/{mc_version}
            let url = format!("https://meta.fabricmc.net/v2/versions/loader/{}", mc_version);
            let resp = reqwest::get(&url).await.map_err(|e| format!("Failed to fetch Fabric loader meta: {}", e))?;
            let arr: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse Fabric loader meta: {}", e))?;
            let loader_version = arr.as_array()
                .and_then(|a| a.first())
                .and_then(|v| v["loader"]["version"].as_str())
                .ok_or("No Fabric loader version found")?;
            Ok(loader_version.to_string())
        },
        "forge" => {
            // Forge meta API: https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json
            let url = "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
            let resp = reqwest::get(url).await.map_err(|e| format!("Failed to fetch Forge promotions: {}", e))?;
            let json: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse Forge promotions: {}", e))?;
            let key = format!("{}.latest", mc_version);
            let latest = json["promos"][&key].as_str().ok_or("No Forge latest version found for this Minecraft version")?;
            Ok(latest.to_string())
        },
        "neoforge" => {
            // NeoForge maven metadata: https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml
            // We'll parse the XML to get the latest version for the given mc_version
            let url = "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";
            let resp = reqwest::get(url).await.map_err(|e| format!("Failed to fetch NeoForge maven metadata: {}", e))?;
            let xml = resp.text().await.map_err(|e| format!("Failed to read NeoForge maven metadata: {}", e))?;
            // Find the latest version for the given mc_version prefix
            let mut latest: Option<String> = None;
            for line in xml.lines() {
                if let Some(start) = line.find("<version>") {
                    if let Some(end) = line.find("</version>") {
                        let version = &line[start + 9..end];
                        if version.starts_with(mc_version) {
                            latest = Some(version.to_string());
                        }
                    }
                }
            }
            latest.ok_or("No NeoForge latest version found for this Minecraft version".to_string())
        },
        _ => Err("Latest version lookup not implemented for this modloader".to_string()),
    }
}

/// Main function to download and set up a modpack instance with group scoping
#[command]
pub async fn download_modpack_with_groups(
    modpack: OnlineModpack,
    user_groups: Vec<String>
) -> Result<(), String> {
    let instance_dir = get_instance_dir(&modpack.id);
    fs::create_dir_all(&instance_dir).map_err(|e| format!("Failed to create instance dir: {}", e))?;

    // Step 1: Determine modloader version
    let modloader_version = if modpack.modloader_version.is_empty() {
        get_latest_modloader_version(&modpack.modloader, &modpack.minecraft_version).await?
    } else {
        modpack.modloader_version.clone()
    };

    // Step 2: Download Minecraft
    download_minecraft(&modpack.minecraft_version, &instance_dir).await?;

    // Step 3: Install mod loader
    match modpack.modloader.to_lowercase().as_str() {
        "forge" => install_forge(&modloader_version, &modpack.minecraft_version, &instance_dir).await?,
        "fabric" => install_fabric(&modloader_version, &modpack.minecraft_version, &instance_dir).await?,
        "neoforge" => install_neoforge(&modloader_version, &modpack.minecraft_version, &instance_dir).await?,
        _ => return Err("Unknown modloader".to_string()),
    }

    // Step 4: Fetch and filter modpack file
    let files = fetch_modpack_file(&modpack.url, &user_groups).await?;
    for entry in files.iter() {
        download_and_verify(entry, &instance_dir).await?;
    }

    Ok(())
}

/// Get the user's Minecraft directory (e.g., ~/.minecraft or %APPDATA%\.minecraft)
fn get_minecraft_dir() -> PathBuf {
    // On Windows, dirs::data_dir() returns %APPDATA%, on Linux ~/.local/share, on macOS ~/Library/Application Support
    // Minecraft uses %APPDATA%\.minecraft on Windows, ~/.minecraft on Linux/macOS
    if cfg!(target_os = "windows") {
        dirs::data_dir()
            .expect("Failed to get data dir")
            .join(".minecraft")
    } else {
        dirs::home_dir()
            .expect("Failed to get home dir")
            .join(".minecraft")
    }
}

/// Get the instance directory for a modpack
fn get_instance_dir(modpack_id: &str) -> PathBuf {
    get_minecraft_dir().join("instances").join(modpack_id)
}

/// Download the vanilla Minecraft jar for the given version
async fn download_minecraft(version: &str, dest_dir: &Path) -> Result<(), String> {
    // Mojang version manifest URL
    let manifest_url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    let manifest_resp = reqwest::get(manifest_url)
        .await
        .map_err(|e| format!("Failed to fetch version manifest: {}", e))?;
    let manifest_json: serde_json::Value = manifest_resp.json()
        .await
        .map_err(|e| format!("Failed to parse version manifest: {}", e))?;

    // Find the version object
    let versions = manifest_json["versions"].as_array().ok_or("Invalid manifest format")?;
    let version_obj = versions.iter().find(|v| v["id"] == version)
        .ok_or(format!("Version {} not found in manifest", version))?;
    let version_url = version_obj["url"].as_str().ok_or("Missing version URL")?;

    // Fetch the version details
    let version_resp = reqwest::get(version_url)
        .await
        .map_err(|e| format!("Failed to fetch version details: {}", e))?;
    let version_json: serde_json::Value = version_resp.json()
        .await
        .map_err(|e| format!("Failed to parse version details: {}", e))?;
    let client_url = version_json["downloads"]["client"]["url"].as_str().ok_or("Missing client jar URL")?;

    // Download the client jar
    let client_resp = reqwest::get(client_url)
        .await
        .map_err(|e| format!("Failed to download client jar: {}", e))?;
    let client_bytes = client_resp.bytes()
        .await
        .map_err(|e| format!("Failed to read client jar bytes: {}", e))?;

    // Save to {dest_dir}/versions/{version}/{version}.jar
    let version_dir = dest_dir.join("versions").join(version);
    fs::create_dir_all(&version_dir).map_err(|e| format!("Failed to create version dir: {}", e))?;
    let jar_path = version_dir.join(format!("{}.jar", version));
    let mut file = fs::File::create(&jar_path).map_err(|e| format!("Failed to create jar file: {}", e))?;
    file.write_all(&client_bytes).map_err(|e| format!("Failed to write jar file: {}", e))?;

    Ok(())
}

/// Download and install Forge for the given version
async fn install_forge(forge_version: &str, mc_version: &str, dest_dir: &Path) -> Result<(), String> {
    // Forge installer URL pattern (official):
    // https://maven.minecraftforge.net/net/minecraftforge/forge/{mc_version}-{forge_version}/forge-{mc_version}-{forge_version}-installer.jar
    let installer_url = format!(
        "https://maven.minecraftforge.net/net/minecraftforge/forge/{mc_version}-{forge_version}/forge-{mc_version}-{forge_version}-installer.jar"
    );
    let installer_resp = reqwest::get(&installer_url)
        .await
        .map_err(|e| format!("Failed to download Forge installer: {}", e))?;
    let installer_bytes = installer_resp.bytes()
        .await
        .map_err(|e| format!("Failed to read Forge installer bytes: {}", e))?;
    let installer_path = dest_dir.join(format!("forge-{mc_version}-{forge_version}-installer.jar"));
    let mut file = fs::File::create(&installer_path).map_err(|e| format!("Failed to create Forge installer file: {}", e))?;
    file.write_all(&installer_bytes).map_err(|e| format!("Failed to write Forge installer file: {}", e))?;

    // Run the installer with Java
    let status = Command::new("java")
        .arg("-jar")
        .arg(&installer_path)
        .arg("--installServer")
        .current_dir(dest_dir)
        .status()
        .map_err(|e| format!("Failed to run Forge installer: {}", e))?;
    if !status.success() {
        return Err("Forge installer failed".to_string());
    }
    Ok(())
}

/// Download and install Fabric for the given version
async fn install_fabric(fabric_version: &str, mc_version: &str, dest_dir: &Path) -> Result<(), String> {
    // Fabric installer URL (universal jar): https://maven.fabricmc.net/net/fabricmc/fabric-installer/0.11.2/fabric-installer-0.11.2.jar
    // We'll use the latest stable installer version (can be parameterized if needed)
    let fabric_installer_version = "0.11.2";
    let installer_url = format!(
        "https://maven.fabricmc.net/net/fabricmc/fabric-installer/{0}/fabric-installer-{0}.jar",
        fabric_installer_version
    );
    let installer_resp = reqwest::get(&installer_url)
        .await
        .map_err(|e| format!("Failed to download Fabric installer: {}", e))?;
    let installer_bytes = installer_resp.bytes()
        .await
        .map_err(|e| format!("Failed to read Fabric installer bytes: {}", e))?;
    let installer_path = dest_dir.join(format!("fabric-installer-{}.jar", fabric_installer_version));
    let mut file = fs::File::create(&installer_path).map_err(|e| format!("Failed to create Fabric installer file: {}", e))?;
    file.write_all(&installer_bytes).map_err(|e| format!("Failed to write Fabric installer file: {}", e))?;

    // Run the installer with Java
    let status = Command::new("java")
        .arg("-jar")
        .arg(&installer_path)
        .arg("client")
        .arg("-dir")
        .arg(dest_dir)
        .arg("-mcversion")
        .arg(mc_version)
        .arg("-loader")
        .arg(fabric_version)
        .status()
        .map_err(|e| format!("Failed to run Fabric installer: {}", e))?;
    if !status.success() {
        return Err("Fabric installer failed".to_string());
    }
    Ok(())
}

/// Download and install NeoForge for the given version
async fn install_neoforge(neoforge_version: &str, mc_version: &str, dest_dir: &Path) -> Result<(), String> {
    // NeoForge installer URL pattern:
    // https://maven.neoforged.net/releases/net/neoforged/neoforge/{mc_version}-{neoforge_version}/neoforge-{mc_version}-{neoforge_version}-installer.jar
    let installer_url = format!(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{mc_version}-{neoforge_version}/neoforge-{mc_version}-{neoforge_version}-installer.jar"
    );
    let installer_resp = reqwest::get(&installer_url)
        .await
        .map_err(|e| format!("Failed to download NeoForge installer: {}", e))?;
    let installer_bytes = installer_resp.bytes()
        .await
        .map_err(|e| format!("Failed to read NeoForge installer bytes: {}", e))?;
    let installer_path = dest_dir.join(format!("neoforge-{mc_version}-{neoforge_version}-installer.jar"));
    let mut file = fs::File::create(&installer_path).map_err(|e| format!("Failed to create NeoForge installer file: {}", e))?;
    file.write_all(&installer_bytes).map_err(|e| format!("Failed to write NeoForge installer file: {}", e))?;

    // Run the installer with Java
    let status = Command::new("java")
        .arg("-jar")
        .arg(&installer_path)
        .arg("--installServer")
        .current_dir(dest_dir)
        .status()
        .map_err(|e| format!("Failed to run NeoForge installer: {}", e))?;
    if !status.success() {
        return Err("NeoForge installer failed".to_string());
    }
    Ok(())
} 