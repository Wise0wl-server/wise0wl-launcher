use crate::modpack::{Mod, Modpack};
use crate::LaunchOptions;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::fs;
use std::io::Write;
use std::collections::HashMap;
use futures::stream::{FuturesUnordered, StreamExt};

#[derive(Debug, Serialize, Deserialize)]
struct MinecraftVersion {
    id: String,
    #[serde(rename = "type")]
    version_type: String,
    url: String,
    time: String,
    #[serde(rename = "releaseTime")]
    release_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct VersionManifest {
    latest: HashMap<String, String>,
    versions: Vec<MinecraftVersion>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VersionDetails {
    id: String,
    #[serde(rename = "type")]
    version_type: String,
    #[serde(rename = "mainClass")]
    main_class: String,
    #[serde(rename = "minecraftArguments")]
    minecraft_arguments: Option<String>,
    arguments: Option<Arguments>,
    libraries: Vec<Library>,
    downloads: Downloads,
    #[serde(rename = "assetIndex")]
    asset_index: AssetIndex,
    assets: String,
    #[serde(rename = "releaseTime")]
    release_time: String,
    time: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Arguments {
    game: Vec<Argument>,
    jvm: Vec<Argument>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Argument {
    String(String),
    Object {
        rules: Vec<Rule>,
        value: serde_json::Value,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct Rule {
    action: String,
    #[serde(rename = "os")]
    operating_system: Option<OsRule>,
    features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OsRule {
    name: Option<String>,
    version: Option<String>,
    arch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Library {
    name: String,
    downloads: Option<LibraryDownloads>,
    rules: Option<Vec<Rule>>,
    natives: Option<HashMap<String, String>>,
    extract: Option<Extract>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LibraryDownloads {
    artifact: Option<Artifact>,
    classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Artifact {
    #[serde(default)]
    path: Option<String>,
    url: String,
    size: u64,
    sha1: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Extract {
    exclude: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Downloads {
    client: Artifact,
    server: Option<Artifact>,
    #[serde(rename = "client_mappings")]
    client_mappings: Option<Artifact>,
    #[serde(rename = "server_mappings")]
    server_mappings: Option<Artifact>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AssetIndex {
    id: String,
    #[serde(rename = "totalSize")]
    total_size: u64,
    url: String,
}

pub struct MinecraftLauncher {
    minecraft_dir: PathBuf,
}

impl MinecraftLauncher {
    pub fn new() -> Self {
        MinecraftLauncher {
            minecraft_dir: Self::get_minecraft_dir(),
        }
    }

    pub async fn launch(&self, options: LaunchOptions) -> Result<(), String> {
        // Debug authentication data
        println!("Launch options - Username: {:?}, UUID: {:?}, Token: {:?}", 
            options.username, 
            options.uuid.as_ref().map(|u| &u[..std::cmp::min(8, u.len())]), 
            options.access_token.as_ref().map(|t| &t[..std::cmp::min(8, t.len())]));
        
        println!("Launching Minecraft with game directory: {}", options.game_dir.display());
        println!("Minecraft directory: {}", self.minecraft_dir.display());
        
        // 1. Verify Java installation
        self.verify_java(&options.java_path)?;

        // 2. Get modpack info
        let modpack = self.get_modpack(&options.modpack_id)?;

        // 3. Ensure Minecraft version is downloaded
        self.ensure_minecraft_version(&modpack.minecraft_version).await?;

        // 3.5. Ensure assets are downloaded
        self.ensure_assets(&modpack.minecraft_version).await?;

        // 4. Handle modloader installation if needed
        self.handle_modloader(&modpack).await?;

        // 5. Download/Update mods if needed
        if !modpack.mods.is_empty() {
            self.update_mods(&modpack, &options.game_dir).await?;
        }

        // 6. Ensure native libraries are extracted
        self.ensure_native_libraries(&modpack.minecraft_version, &options.game_dir).await?;

        // 7. Get version details and build launch command
        let version_details = self.get_version_details(&modpack.minecraft_version).await?;
        let command = self.build_launch_command(&options, &modpack, &version_details)?;

        // 8. Launch the game
        self.execute_command(command)
    }

    fn verify_java(&self, java_path: &Option<String>) -> Result<(), String> {
        let java = java_path.as_ref().map(String::as_str).unwrap_or("java");

        let output = Command::new(java)
            .arg("-version")
            .output()
            .map_err(|e| format!("Failed to execute Java: {}", e))?;

        if !output.status.success() {
            return Err("Java is not installed or not accessible".to_string());
        }

        Ok(())
    }

    fn get_modpack(&self, modpack_id: &str) -> Result<Modpack, String> {
        let modpacks = crate::modpack::get_all_modpacks()?;
        modpacks
            .into_iter()
            .find(|m| m.id == modpack_id)
            .ok_or_else(|| format!("Modpack '{}' not found", modpack_id))
    }

    async fn ensure_minecraft_version(&self, version: &str) -> Result<(), String> {
        let version_dir = self.minecraft_dir.join("versions").join(version);
        let jar_path = version_dir.join(format!("{}.jar", version));

        if !jar_path.exists() {
            self.download_minecraft_version(version).await?;
        }

        // Ensure libraries are downloaded
        self.ensure_libraries(version).await?;

        Ok(())
    }

    async fn download_minecraft_version(&self, version: &str) -> Result<(), String> {
        // Get version manifest
        let manifest_url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
        let manifest_resp = reqwest::get(manifest_url)
            .await
            .map_err(|e| format!("Failed to fetch version manifest: {}", e))?;
        
        let manifest: VersionManifest = manifest_resp.json()
            .await
            .map_err(|e| format!("Failed to parse version manifest: {}", e))?;

        // Find the version
        let version_info = manifest.versions
            .into_iter()
            .find(|v| v.id == version)
            .ok_or_else(|| format!("Version {} not found", version))?;

        // Get version details
        let version_resp = reqwest::get(&version_info.url)
            .await
            .map_err(|e| format!("Failed to fetch version details: {}", e))?;
        
        let version_details: VersionDetails = version_resp.json()
            .await
            .map_err(|e| format!("Failed to parse version details: {}", e))?;

        // Create version directory
        let version_dir = self.minecraft_dir.join("versions").join(version);
        fs::create_dir_all(&version_dir)
            .map_err(|e| format!("Failed to create version directory: {}", e))?;

        // Download client jar
        let jar_path = version_dir.join(format!("{}.jar", version));
        if !jar_path.exists() {
            let jar_resp = reqwest::get(&version_details.downloads.client.url)
                .await
                .map_err(|e| format!("Failed to download client jar: {}", e))?;
            
            let jar_bytes = jar_resp.bytes()
                .await
                .map_err(|e| format!("Failed to read jar bytes: {}", e))?;

            let mut file = fs::File::create(&jar_path)
                .map_err(|e| format!("Failed to create jar file: {}", e))?;
            file.write_all(&jar_bytes)
                .map_err(|e| format!("Failed to write jar file: {}", e))?;
        }

        // Save version details
        let version_json_path = version_dir.join(format!("{}.json", version));
        let version_json = serde_json::to_string_pretty(&version_details)
            .map_err(|e| format!("Failed to serialize version details: {}", e))?;
        
        fs::write(&version_json_path, version_json)
            .map_err(|e| format!("Failed to write version JSON: {}", e))?;

        Ok(())
    }

    async fn handle_modloader(&self, modpack: &Modpack) -> Result<(), String> {
        // Check if modloader is already installed
        let modloader_version = if let Some(forge_version) = &modpack.forge_version {
            format!("{}-forge-{}", modpack.minecraft_version, forge_version)
        } else if let Some(fabric_version) = &modpack.fabric_version {
            format!("{}-fabric-{}", modpack.minecraft_version, fabric_version)
        } else if let Some(neoforge_version) = &modpack.neoforge_version {
            format!("{}-neoforge-{}", modpack.minecraft_version, neoforge_version)
        } else {
            return Ok(()); // No modloader needed
        };

        let modloader_dir = self.minecraft_dir.join("versions").join(&modloader_version);
        let modloader_jar = modloader_dir.join(format!("{}.jar", modloader_version));

        if !modloader_jar.exists() {
            // Install modloader
            if modpack.forge_version.is_some() {
                self.install_forge(&modpack.minecraft_version, modpack.forge_version.as_ref().unwrap()).await?;
            } else if modpack.fabric_version.is_some() {
                self.install_fabric(&modpack.minecraft_version, modpack.fabric_version.as_ref().unwrap()).await?;
            } else if modpack.neoforge_version.is_some() {
                self.install_neoforge(&modpack.minecraft_version, modpack.neoforge_version.as_ref().unwrap()).await?;
            }
        }

        Ok(())
    }

    async fn install_forge(&self, mc_version: &str, forge_version: &str) -> Result<(), String> {
        let installer_url = format!(
            "https://maven.minecraftforge.net/net/minecraftforge/forge/{mc_version}-{forge_version}/forge-{mc_version}-{forge_version}-installer.jar"
        );
        
        let installer_resp = reqwest::get(&installer_url)
            .await
            .map_err(|e| format!("Failed to download Forge installer: {}", e))?;
        
        let installer_bytes = installer_resp.bytes()
            .await
            .map_err(|e| format!("Failed to read Forge installer: {}", e))?;

        let temp_dir = std::env::temp_dir().join("forge-installer");
        fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;

        let installer_path = temp_dir.join(format!("forge-{mc_version}-{forge_version}-installer.jar"));
        let mut file = fs::File::create(&installer_path)
            .map_err(|e| format!("Failed to create installer file: {}", e))?;
        file.write_all(&installer_bytes)
            .map_err(|e| format!("Failed to write installer: {}", e))?;

        // Run installer
        let status = Command::new("java")
            .arg("-jar")
            .arg(&installer_path)
            .arg("--installClient")
            .arg(&self.minecraft_dir)
            .status()
            .map_err(|e| format!("Failed to run Forge installer: {}", e))?;

        if !status.success() {
            return Err("Forge installer failed".to_string());
        }

        Ok(())
    }

    async fn install_fabric(&self, mc_version: &str, fabric_version: &str) -> Result<(), String> {
        let installer_url = "https://maven.fabricmc.net/net/fabricmc/fabric-installer/0.11.2/fabric-installer-0.11.2.jar";
        
        let installer_resp = reqwest::get(installer_url)
            .await
            .map_err(|e| format!("Failed to download Fabric installer: {}", e))?;
        
        let installer_bytes = installer_resp.bytes()
            .await
            .map_err(|e| format!("Failed to read Fabric installer: {}", e))?;

        let temp_dir = std::env::temp_dir().join("fabric-installer");
        fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;

        let installer_path = temp_dir.join("fabric-installer-0.11.2.jar");
        let mut file = fs::File::create(&installer_path)
            .map_err(|e| format!("Failed to create installer file: {}", e))?;
        file.write_all(&installer_bytes)
            .map_err(|e| format!("Failed to write installer: {}", e))?;

        // Run installer
        let status = Command::new("java")
            .arg("-jar")
            .arg(&installer_path)
            .arg("client")
            .arg("-dir")
            .arg(&self.minecraft_dir)
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

    async fn install_neoforge(&self, mc_version: &str, neoforge_version: &str) -> Result<(), String> {
        let installer_url = format!(
            "https://maven.neoforged.net/releases/net/neoforged/neoforge/{mc_version}-{neoforge_version}/neoforge-{mc_version}-{neoforge_version}-installer.jar"
        );
        
        let installer_resp = reqwest::get(&installer_url)
            .await
            .map_err(|e| format!("Failed to download NeoForge installer: {}", e))?;
        
        let installer_bytes = installer_resp.bytes()
            .await
            .map_err(|e| format!("Failed to read NeoForge installer: {}", e))?;

        let temp_dir = std::env::temp_dir().join("neoforge-installer");
        fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;

        let installer_path = temp_dir.join(format!("neoforge-{mc_version}-{neoforge_version}-installer.jar"));
        let mut file = fs::File::create(&installer_path)
            .map_err(|e| format!("Failed to create installer file: {}", e))?;
        file.write_all(&installer_bytes)
            .map_err(|e| format!("Failed to write installer: {}", e))?;

        // Run installer
        let status = Command::new("java")
            .arg("-jar")
            .arg(&installer_path)
            .arg("--installClient")
            .arg(&self.minecraft_dir)
            .status()
            .map_err(|e| format!("Failed to run NeoForge installer: {}", e))?;

        if !status.success() {
            return Err("NeoForge installer failed".to_string());
        }

        Ok(())
    }

    async fn get_version_details(&self, version: &str) -> Result<VersionDetails, String> {
        let version_dir = self.minecraft_dir.join("versions").join(version);
        let version_json_path = version_dir.join(format!("{}.json", version));

        if !version_json_path.exists() {
            self.download_minecraft_version(version).await?;
        }

        let version_json = fs::read_to_string(&version_json_path)
            .map_err(|e| format!("Failed to read version JSON: {}", e))?;
        
        serde_json::from_str(&version_json)
            .map_err(|e| format!("Failed to parse version JSON: {}", e))
    }

    async fn update_mods(&self, modpack: &Modpack, game_dir: &PathBuf) -> Result<(), String> {
        let mods_dir = game_dir.join("mods");
        fs::create_dir_all(&mods_dir)
            .map_err(|e| format!("Failed to create mods directory: {}", e))?;

        for mod_info in &modpack.mods {
            self.download_mod(mod_info, &mods_dir).await?;
        }

        Ok(())
    }

    async fn download_mod(&self, mod_info: &Mod, mods_dir: &PathBuf) -> Result<(), String> {
        let mod_path = mods_dir.join(&mod_info.name);
        
        if !mod_path.exists() {
            let resp = reqwest::get(&mod_info.download_url)
                .await
                .map_err(|e| format!("Failed to download mod {}: {}", mod_info.name, e))?;
            
            let bytes = resp.bytes()
                .await
                .map_err(|e| format!("Failed to read mod bytes: {}", e))?;

            let mut file = fs::File::create(&mod_path)
                .map_err(|e| format!("Failed to create mod file: {}", e))?;
            file.write_all(&bytes)
                .map_err(|e| format!("Failed to write mod file: {}", e))?;
        }

        Ok(())
    }

    fn build_launch_command(
        &self,
        options: &LaunchOptions,
        modpack: &Modpack,
        version_details: &VersionDetails,
    ) -> Result<Command, String> {
        let java_path = options
            .java_path
            .as_ref()
            .map(String::as_str)
            .ok_or("Java path is required")?;

        let mut command = Command::new(java_path);

        // JVM arguments
        self.add_jvm_arguments(&mut command, version_details, options)?;

        // Main class
        command.arg(&version_details.main_class);

        // Game arguments
        self.add_game_arguments(&mut command, version_details, options, modpack)?;

        Ok(command)
    }

    fn add_jvm_arguments(
        &self,
        command: &mut Command,
        version_details: &VersionDetails,
        options: &LaunchOptions,
    ) -> Result<(), String> {
        // Memory settings
        if let Some(max_mem) = options.max_memory {
            command.arg(format!("-Xmx{}M", max_mem));
        }
        if let Some(min_mem) = options.min_memory {
            command.arg(format!("-Xms{}M", min_mem));
        }

        // Add JVM arguments from version details
        if let Some(arguments) = &version_details.arguments {
            for arg in &arguments.jvm {
                match arg {
                    Argument::String(s) => {
                        // Filter out macOS-specific options on other platforms
                        if !self.should_skip_jvm_argument(s) {
                            let processed = self.process_jvm_argument(s, options);
                            command.arg(processed);
                        }
                    }
                    Argument::Object { rules, value } => {
                        if self.should_apply_rule(rules) {
                            if let Some(s) = value.as_str() {
                                if !self.should_skip_jvm_argument(s) {
                                    let processed = self.process_jvm_argument(s, options);
                                    command.arg(processed);
                                }
                            } else if let Some(arr) = value.as_array() {
                                for item in arr {
                                    if let Some(s) = item.as_str() {
                                        if !self.should_skip_jvm_argument(s) {
                                            let processed = self.process_jvm_argument(s, options);
                                            command.arg(processed);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Add library paths
        let libraries_path = self.minecraft_dir.join("libraries");
        command.arg(format!("-Djava.library.path={}", libraries_path.display()));

        // Add natives directory
        let natives_dir = options.game_dir.join("natives");
        command.arg(format!("-Djava.library.path={};{}", libraries_path.display(), natives_dir.display()));

        // Add classpath
        let classpath = self.build_classpath(version_details)?;
        command.arg(format!("-cp"));
        command.arg(classpath);

        Ok(())
    }

    fn process_jvm_argument(&self, arg: &str, options: &LaunchOptions) -> String {
        arg.replace("${natives_directory}", &options.game_dir.join("natives").to_string_lossy())
    }

    fn should_skip_jvm_argument(&self, arg: &str) -> bool {
        // Skip macOS-specific JVM arguments on other platforms
        if !cfg!(target_os = "macos") {
            match arg {
                "-XstartOnFirstThread" => true,
                _ => false,
            }
        } else {
            false
        }
    }

    fn add_game_arguments(
        &self,
        command: &mut Command,
        version_details: &VersionDetails,
        options: &LaunchOptions,
        modpack: &Modpack,
    ) -> Result<(), String> {
        if let Some(arguments) = &version_details.arguments {
            // New argument format
            for argument in &arguments.game {
                match argument {
                    Argument::String(s) => {
                        let processed = self.process_game_argument(s, options, modpack);
                        println!("Game argument: '{}' -> '{}'", s, processed);
                        command.arg(processed);
                    }
                    Argument::Object { rules, value } => {
                        println!("Processing argument with rules: {:?}, value: {:?}", rules, value);
                        if self.should_apply_rule(rules) {
                            match value {
                                serde_json::Value::String(s) => {
                                    let processed = self.process_game_argument(s, options, modpack);
                                    println!("Game argument (with rules): '{}' -> '{}'", s, processed);
                                    command.arg(processed);
                                }
                                serde_json::Value::Array(arr) => {
                                    for item in arr {
                                        if let Some(s) = item.as_str() {
                                            let processed = self.process_game_argument(s, options, modpack);
                                            println!("Game argument (array): '{}' -> '{}'", s, processed);
                                            command.arg(processed);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        } else {
                            println!("Skipping game argument due to rules: {:?}", value);
                        }
                    }
                }
            }
        } else if let Some(minecraft_args) = &version_details.minecraft_arguments {
            // Legacy argument format
            for arg in minecraft_args.split_whitespace() {
                let processed = self.process_game_argument(arg, options, modpack);
                println!("Legacy game argument: '{}' -> '{}'", arg, processed);
                command.arg(processed);
            }
        }

        Ok(())
    }

    fn process_game_argument(&self, arg: &str, options: &LaunchOptions, modpack: &Modpack) -> String {
        // Get the asset index name from version details
        let asset_index_name = {
            let version_json_path = self.minecraft_dir
                .join("versions")
                .join(&modpack.minecraft_version)
                .join(format!("{}.json", modpack.minecraft_version));
            
            if let Ok(version_json) = fs::read_to_string(&version_json_path) {
                if let Ok(version_details) = serde_json::from_str::<VersionDetails>(&version_json) {
                    version_details.asset_index.id
                } else {
                    modpack.minecraft_version.clone()
                }
            } else {
                modpack.minecraft_version.clone()
            }
        };

        let processed = arg.replace("${auth_player_name}", options.username.as_deref().unwrap_or("Player"))
            .replace("${version_name}", &modpack.minecraft_version)
            .replace("${game_directory}", &options.game_dir.to_string_lossy())
            .replace("${assets_root}", &self.minecraft_dir.join("assets").to_string_lossy())
            .replace("${assets_index_name}", &asset_index_name)
            .replace("${auth_uuid}", options.uuid.as_deref().unwrap_or("00000000-0000-0000-0000-000000000000"))
            .replace("${auth_access_token}", options.access_token.as_deref().unwrap_or("token"))
            .replace("${clientid}", "clientid")
            .replace("${auth_xuid}", "xuid")
            .replace("${user_type}", "msa")
            .replace("${version_type}", "release")
            .replace("${resolution_width}", &options.width.unwrap_or(1280).to_string())
            .replace("${resolution_height}", &options.height.unwrap_or(720).to_string())
            .replace("${natives_directory}", &options.game_dir.join("natives").to_string_lossy());
        
        // Debug asset paths
        if arg.contains("${assets_root}") || arg.contains("${assets_index_name}") {
            println!("Asset debug - Original: '{}', Processed: '{}'", arg, processed);
        }
        
        // Debug authentication data
        if arg.contains("${auth_player_name}") || arg.contains("${auth_uuid}") || arg.contains("${auth_access_token}") {
            println!("Auth debug - Username: {:?}, UUID: {:?}, Token: {:?}", 
                options.username, 
                options.uuid.as_ref().map(|u| &u[..std::cmp::min(8, u.len())]), 
                options.access_token.as_ref().map(|t| &t[..std::cmp::min(8, t.len())]));
        }
        
        processed
    }

    fn should_apply_rule(&self, rules: &[Rule]) -> bool {
        if rules.is_empty() {
            return true;
        }

        for rule in rules {
            let mut should_apply = true;

            if let Some(os_rule) = &rule.operating_system {
                if let Some(name) = &os_rule.name {
                    should_apply = should_apply && self.matches_os_name(name);
                }
                if let Some(arch) = &os_rule.arch {
                    should_apply = should_apply && self.matches_arch(arch);
                }
            }

            if let Some(features) = &rule.features {
                for (feature, required) in features {
                    match feature.as_str() {
                        "is_demo_user" => {
                            println!("is_demo_user rule: required={}, we have valid auth, so should_apply = should_apply && {}", required, *required == false);
                            should_apply = should_apply && *required == false;
                        }
                        "has_custom_resolution" => {
                            should_apply = should_apply && !required;
                        }
                        "has_quick_plays_support" => {
                            should_apply = should_apply && !required;
                        }
                        "is_quick_play_singleplayer" => {
                            should_apply = should_apply && !required;
                        }
                        "is_quick_play_multiplayer" => {
                            should_apply = should_apply && !required;
                        }
                        "is_quick_play_realms" => {
                            should_apply = should_apply && !required;
                        }
                        _ => {}
                    }
                }
            }

            if rule.action == "allow" && should_apply {
                return true;
            } else if rule.action == "disallow" && should_apply {
                return false;
            }
        }

        false // If no rules matched, do not apply the argument
    }

    fn matches_os_name(&self, name: &str) -> bool {
        match name {
            "windows" => cfg!(target_os = "windows"),
            "linux" => cfg!(target_os = "linux"),
            "osx" => cfg!(target_os = "macos"),
            _ => false,
        }
    }

    fn matches_arch(&self, arch: &str) -> bool {
        match arch {
            "x86" => cfg!(target_arch = "x86"),
            "x64" => cfg!(target_arch = "x86_64"),
            "arm64" => cfg!(target_arch = "aarch64"),
            _ => false,
        }
    }

    fn build_classpath(&self, version_details: &VersionDetails) -> Result<String, String> {
        let mut classpath_parts = Vec::new();

        // Add version jar
        let version_jar = self.minecraft_dir
            .join("versions")
            .join(&version_details.id)
            .join(format!("{}.jar", version_details.id));
        classpath_parts.push(version_jar.to_string_lossy().to_string());
        println!("Added version jar to classpath: {}", version_jar.display());

        // Add libraries
        let mut library_count = 0;
        for library in &version_details.libraries {
            if self.should_include_library(library) {
                if let Some(downloads) = &library.downloads {
                    if let Some(artifact) = &downloads.artifact {
                        if let Some(path) = &artifact.path {
                            let library_path = self.minecraft_dir
                                .join("libraries")
                                .join(path);
                            
                            if library_path.exists() {
                                classpath_parts.push(library_path.to_string_lossy().to_string());
                                library_count += 1;
                                if library_count <= 5 { // Only print first 5 for debugging
                                    println!("Added library to classpath: {}", path);
                                }
                            } else {
                                println!("Warning: Library not found: {}", library_path.display());
                            }
                        }
                    }
                }
            }
        }
        println!("Total libraries in classpath: {}", library_count);

        let separator = if cfg!(target_os = "windows") { ";" } else { ":" };
        let classpath = classpath_parts.join(separator);
        println!("Final classpath length: {} characters", classpath.len());
        
        Ok(classpath)
    }

    fn should_include_library(&self, library: &Library) -> bool {
        if let Some(rules) = &library.rules {
            self.should_apply_rule(rules)
        } else {
            true
        }
    }

    fn execute_command(&self, mut command: Command) -> Result<(), String> {
        // Debug: Print the command being executed
        println!("Executing command: {:?}", command);
        
        command
            .spawn()
            .map_err(|e| format!("Failed to launch Minecraft: {}", e))?;
        Ok(())
    }

    fn get_minecraft_dir() -> PathBuf {
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

    async fn ensure_libraries(&self, version: &str) -> Result<(), String> {
        // Load version details to get library list
        let version_json_path = self.minecraft_dir
            .join("versions")
            .join(version)
            .join(format!("{}.json", version));
        
        let version_json = fs::read_to_string(&version_json_path)
            .map_err(|e| format!("Failed to read version JSON: {}", e))?;
        
        let version_details: VersionDetails = serde_json::from_str(&version_json)
            .map_err(|e| format!("Failed to parse version JSON: {}", e))?;

        // Download libraries
        for library in &version_details.libraries {
            if self.should_include_library(library) {
                if let Some(downloads) = &library.downloads {
                    if let Some(artifact) = &downloads.artifact {
                        if let Some(path) = &artifact.path {
                            let library_path = self.minecraft_dir.join("libraries").join(path);
                            
                            // Create parent directory if it doesn't exist
                            if let Some(parent) = library_path.parent() {
                                fs::create_dir_all(parent)
                                    .map_err(|e| format!("Failed to create library directory: {}", e))?;
                            }

                            // Download if not exists
                            if !library_path.exists() {
                                println!("Downloading library: {}", path);
                                let library_resp = reqwest::get(&artifact.url)
                                    .await
                                    .map_err(|e| format!("Failed to download library {}: {}", path, e))?;
                                
                                let library_bytes = library_resp.bytes()
                                    .await
                                    .map_err(|e| format!("Failed to read library bytes for {}: {}", path, e))?;

                                let mut file = fs::File::create(&library_path)
                                    .map_err(|e| format!("Failed to create library file {}: {}", path, e))?;
                                file.write_all(&library_bytes)
                                    .map_err(|e| format!("Failed to write library file {}: {}", path, e))?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn ensure_native_libraries(&self, version: &str, game_dir: &PathBuf) -> Result<(), String> {
        println!("Ensuring native libraries for version {} in game directory: {}", version, game_dir.display());
        
        // Load version details to get library list
        let version_json_path = self.minecraft_dir
            .join("versions")
            .join(version)
            .join(format!("{}.json", version));
        
        let version_json = fs::read_to_string(&version_json_path)
            .map_err(|e| format!("Failed to read version JSON: {}", e))?;
        
        let version_details: VersionDetails = serde_json::from_str(&version_json)
            .map_err(|e| format!("Failed to parse version JSON: {}", e))?;

        // Create natives directory
        let natives_dir = game_dir.join("natives");
        println!("Creating natives directory: {}", natives_dir.display());
        fs::create_dir_all(&natives_dir)
            .map_err(|e| format!("Failed to create natives directory: {}", e))?;

        // Extract native libraries
        for library in &version_details.libraries {
            if self.should_include_library(library) {
                if let Some(natives) = &library.natives {
                    // Get the native library for the current platform
                    let platform_key = if cfg!(target_os = "windows") {
                        "natives-windows"
                    } else if cfg!(target_os = "linux") {
                        "natives-linux"
                    } else if cfg!(target_os = "macos") {
                        "natives-macos"
                    } else {
                        continue;
                    };

                    if let Some(native_path) = natives.get(platform_key) {
                        if let Some(downloads) = &library.downloads {
                            if let Some(classifiers) = &downloads.classifiers {
                                if let Some(_native_artifact) = classifiers.get(native_path) {
                                    let native_library_path = self.minecraft_dir.join("libraries").join(native_path);
                                    
                                    if native_library_path.exists() {
                                        println!("Extracting native library from: {}", native_library_path.display());
                                        // Extract native library
                                        self.extract_native_library(&native_library_path, &natives_dir).await?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn extract_native_library(&self, library_path: &PathBuf, natives_dir: &PathBuf) -> Result<(), String> {
        use std::io::Read;
        
        let file = fs::File::open(library_path)
            .map_err(|e| format!("Failed to open native library {}: {}", library_path.display(), e))?;
        
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Failed to open zip archive {}: {}", library_path.display(), e))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| format!("Failed to access file in archive: {}", e))?;
            
            let file_path = file.name();
            
            // Only extract native files (dll, so, dylib)
            if file_path.ends_with(".dll") || file_path.ends_with(".so") || file_path.ends_with(".dylib") {
                let output_path = natives_dir.join(file_path.split('/').last().unwrap_or(file_path));
                
                let mut output_file = fs::File::create(&output_path)
                    .map_err(|e| format!("Failed to create native file {}: {}", output_path.display(), e))?;
                
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)
                    .map_err(|e| format!("Failed to read native file from archive: {}", e))?;
                
                output_file.write_all(&buffer)
                    .map_err(|e| format!("Failed to write native file {}: {}", output_path.display(), e))?;
                
                println!("Extracted native library: {}", output_path.display());
            }
        }

        Ok(())
    }

    /// Download all assets referenced in the asset index for the given version.
    pub async fn ensure_assets(&self, version: &str) -> Result<(), String> {
        // 1. Load version details to get asset index
        let version_json_path = self.minecraft_dir
            .join("versions")
            .join(version)
            .join(format!("{}.json", version));
        let version_json = fs::read_to_string(&version_json_path)
            .map_err(|e| format!("Failed to read version JSON: {}", e))?;
        let version_details: VersionDetails = serde_json::from_str(&version_json)
            .map_err(|e| format!("Failed to parse version JSON: {}", e))?;

        println!("Asset index ID: {}", version_details.asset_index.id);
        println!("Asset index URL: {}", version_details.asset_index.url);

        // 2. Download asset index file
        let asset_index_path = self.minecraft_dir
            .join("assets")
            .join("indexes")
            .join(format!("{}.json", version_details.asset_index.id));
        
        println!("Asset index path: {}", asset_index_path.display());
        
        if !asset_index_path.exists() {
            println!("Downloading asset index for version {}", version);
            let resp = reqwest::get(&version_details.asset_index.url)
                .await
                .map_err(|e| format!("Failed to download asset index: {}", e))?;
            let bytes = resp.bytes()
                .await
                .map_err(|e| format!("Failed to read asset index bytes: {}", e))?;
            fs::create_dir_all(asset_index_path.parent().unwrap())
                .map_err(|e| format!("Failed to create asset index dir: {}", e))?;
            fs::write(&asset_index_path, &bytes)
                .map_err(|e| format!("Failed to write asset index: {}", e))?;
        }

        // 3. Parse asset index
        let asset_index_json = fs::read_to_string(&asset_index_path)
            .map_err(|e| format!("Failed to read asset index: {}", e))?;
        let asset_index: serde_json::Value = serde_json::from_str(&asset_index_json)
            .map_err(|e| format!("Failed to parse asset index: {}", e))?;

        let objects = asset_index["objects"].as_object().ok_or("Invalid asset index format")?;
        println!("Found {} assets in index", objects.len());

        // 4. Find missing assets
        let mut missing_assets = Vec::new();
        for (name, obj) in objects {
            let hash = obj["hash"].as_str().ok_or("Missing hash in asset object")?;
            let subdir = &hash[0..2];
            let asset_path = self.minecraft_dir
                .join("assets")
                .join("objects")
                .join(subdir)
                .join(hash);

            if !asset_path.exists() {
                missing_assets.push((name.clone(), hash.to_string(), asset_path));
            }
        }

        if missing_assets.is_empty() {
            println!("All assets are already downloaded for version {}", version);
            return Ok(());
        }

        println!("Downloading {} missing assets for version {}", missing_assets.len(), version);
        println!("Assets directory: {}", self.minecraft_dir.join("assets").display());

        // 5. Download missing assets with retries and rate limiting
        let mut downloaded = 0;
        let mut failed = Vec::new();
        
        // Process assets in smaller batches to avoid overwhelming the server
        let batch_size = 10;
        for chunk in missing_assets.chunks(batch_size) {
            let mut futures = FuturesUnordered::new();
            
            for (name, hash, asset_path) in chunk {
                let url = format!("https://resources.download.minecraft.net/{}/{}", &hash[0..2], hash);
                let asset_path = asset_path.clone();
                let name = name.clone();
                let _hash = hash.clone();
                
                futures.push(async move {
                    // Retry up to 3 times with exponential backoff
                    for attempt in 1..=3 {
                        match download_asset_with_retry(&url, &asset_path, &name, attempt).await {
                            Ok(()) => return Ok(()),
                            Err(e) => {
                                if attempt == 3 {
                                    return Err(e);
                                }
                                // Wait before retry (exponential backoff)
                                let delay = std::time::Duration::from_millis(100 * attempt as u64);
                                tokio::time::sleep(delay).await;
                            }
                        }
                    }
                    unreachable!()
                });
            }

            // Await batch completion
            while let Some(result) = futures.next().await {
                match result {
                    Ok(()) => {
                        downloaded += 1;
                        if downloaded % 50 == 0 {
                            println!("Downloaded {} assets...", downloaded);
                        }
                    }
                    Err(e) => failed.push(e),
                }
            }

            // Rate limiting between batches
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        println!("Asset download complete: {} downloaded, {} failed", downloaded, failed.len());

        if !failed.is_empty() {
            // Only report first 10 errors to avoid overwhelming output
            let error_count = failed.len();
            let reported_errors = failed.into_iter().take(10).collect::<Vec<_>>();
            let mut error_msg = format!("{} assets failed to download. First {} errors:", error_count, reported_errors.len());
            for error in reported_errors {
                error_msg.push_str(&format!("\n  {}", error));
            }
            if error_count > 10 {
                error_msg.push_str(&format!("\n  ... and {} more errors", error_count - 10));
            }
            return Err(error_msg);
        }

        Ok(())
    }
}

/// Helper function to download a single asset with retry logic
async fn download_asset_with_retry(url: &str, asset_path: &PathBuf, name: &str, attempt: u32) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let resp = client.get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("Failed to download asset {} (attempt {}): {}", name, attempt, e))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {} for asset {} (attempt {})", resp.status(), name, attempt));
    }

    let bytes = resp.bytes()
        .await
        .map_err(|e| format!("Failed to read asset bytes for {} (attempt {}): {}", name, attempt, e))?;

    fs::create_dir_all(asset_path.parent().unwrap())
        .map_err(|e| format!("Failed to create asset dir: {}", e))?;
    
    fs::write(asset_path, &bytes)
        .map_err(|e| format!("Failed to write asset file: {}", e))?;

    Ok(())
}
