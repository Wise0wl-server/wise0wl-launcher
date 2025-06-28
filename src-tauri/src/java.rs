use std::path::{Path, PathBuf};
use std::fs;
use std::io::Cursor;

use dirs::data_dir;
use reqwest::Client;

#[cfg(target_os = "windows")]
const PLATFORM: &str = "windows";
#[cfg(target_os = "linux")]
const PLATFORM: &str = "linux";
#[cfg(target_os = "macos")]
const PLATFORM: &str = "mac";

#[cfg(target_arch = "x86_64")]
const ARCH: &str = "x64";
#[cfg(target_arch = "aarch64")]
const ARCH: &str = "aarch64";

const JAVA_VENDOR: &str = "eclipse";
const IMAGE_TYPE: &str = "jre";

fn java_storage_dir(java_version: u32) -> PathBuf {
    data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".minecraft-wise0wl")
        .join("runtime")
        .join(format!("jre-{}", java_version))
}

fn java_bin_path(root: &Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        root.join("bin").join("java.exe")
    }
    #[cfg(not(target_os = "windows"))]
    {
        root.join("bin").join("java")
    }
}

fn find_existing_java(java_version: u32) -> Option<PathBuf> {
    let java_dir = java_storage_dir(java_version);
    if java_dir.exists() {
        let entries = fs::read_dir(&java_dir).ok()?;
        for entry in entries {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                let java_bin = java_bin_path(&path);
                if java_bin.exists() {
                    return Some(java_bin);
                }
            }
        }
    }
    None
}

pub async fn ensure_java_installed(java_version: u32) -> Result<PathBuf, String> {
    // 1. Check for existing Java in our managed dir
    if let Some(java_bin) = find_existing_java(java_version) {
        return Ok(java_bin);
    }

    // 2. Download Adoptium JRE
    let api_url = format!(
        "https://api.adoptium.net/v3/assets/feature_releases/{}/ga?architecture={}&image_type={}&os={}&vendor={}&heap_size=normal",
        java_version, ARCH, IMAGE_TYPE, PLATFORM, JAVA_VENDOR
    );
    let client = Client::new();
    let resp = client.get(&api_url).send().await.map_err(|e| format!("Failed to query Adoptium API: {}", e))?;
    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse Adoptium API response: {}", e))?;
    let assets = json.as_array().ok_or("Unexpected Adoptium API response")?;
    let asset = assets.get(0).ok_or("No Adoptium JRE found for this platform")?;
    let binaries = asset.get("binaries").and_then(|b| b.as_array()).ok_or("No binaries in Adoptium asset")?;
    let binary = binaries.get(0).ok_or("No binary found in Adoptium asset")?;
    let pkg = binary.get("package").ok_or("No package in Adoptium binary")?;
    let link = pkg.get("link").and_then(|l| l.as_str()).ok_or("No download link in Adoptium package")?;
    let filename = pkg.get("name").and_then(|n| n.as_str()).ok_or("No filename in Adoptium package")?;

    // 3. Download the archive
    let java_dir = java_storage_dir(java_version);
    fs::create_dir_all(&java_dir).map_err(|e| format!("Failed to create java dir: {}", e))?;
    let archive_path = java_dir.join(filename);
    let resp = client.get(link).send().await.map_err(|e| format!("Failed to download JRE: {}", e))?;
    let bytes = resp.bytes().await.map_err(|e| format!("Failed to read JRE bytes: {}", e))?;
    fs::write(&archive_path, &bytes).map_err(|e| format!("Failed to save JRE archive: {}", e))?;

    // 4. Extract the archive
    let extract_dir = java_dir.join(filename.replace(".zip", "").replace(".tar.gz", ""));
    if extract_dir.exists() {
        fs::remove_dir_all(&extract_dir).map_err(|e| format!("Failed to clean old java dir: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        let reader = Cursor::new(&bytes);
        let mut zip = zip::ZipArchive::new(reader).map_err(|e| format!("Failed to open zip: {}", e))?;
        zip.extract(&extract_dir).map_err(|e| format!("Failed to extract zip: {}", e))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        use flate2::read::GzDecoder;
        use tar::Archive;
        let reader = Cursor::new(&bytes);
        let gz = GzDecoder::new(reader);
        let mut archive = Archive::new(gz);
        archive.unpack(&extract_dir).map_err(|e| format!("Failed to extract tar.gz: {}", e))?;
    }

    // 5. Find the java binary in the extracted dir
    // Adoptium archives usually have a top-level dir, so search for it
    let mut java_bin = None;
    for entry in fs::read_dir(&extract_dir).map_err(|e| format!("Failed to read extract dir: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read extract entry: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            let candidate = java_bin_path(&path);
            if candidate.exists() {
                java_bin = Some(candidate);
                break;
            }
        }
    }
    let java_bin = java_bin.ok_or("Failed to find java binary after extraction")?;
    Ok(java_bin)
} 