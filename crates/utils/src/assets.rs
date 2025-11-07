use directories::ProjectDirs;
use rust_embed::RustEmbed;

/// Find the workspace root by walking up from the manifest directory
/// and collecting ALL directories with [workspace], then return the outermost one
/// This handles nested workspaces (forge-app contains upstream submodule workspace)
fn find_workspace_root() -> std::path::PathBuf {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let mut workspace_roots: Vec<std::path::PathBuf> = manifest_dir
        .ancestors()
        .filter_map(|path| {
            let cargo_toml = path.join("Cargo.toml");
            if !cargo_toml.exists() {
                return None;
            }

            // Check if this Cargo.toml contains [workspace]
            std::fs::read_to_string(&cargo_toml)
                .ok()
                .and_then(|content| {
                    if content.contains("[workspace]") {
                        Some(path.to_path_buf())
                    } else {
                        None
                    }
                })
        })
        .collect();

    // Return the outermost workspace root (last in the list since ancestors() goes from inner to outer)
    workspace_roots
        .pop()
        .expect("Could not find any workspace root with [workspace] in Cargo.toml")
}

pub fn asset_dir() -> std::path::PathBuf {
    let path = if cfg!(debug_assertions) {
        // In debug mode, use workspace_root/dev_assets
        // This correctly handles the forge-app structure where upstream is a submodule
        find_workspace_root().join("dev_assets")
    } else {
        ProjectDirs::from("ai", "namastex", "automagik-forge")
            .expect("OS didn't give us a home directory")
            .data_dir()
            .to_path_buf()
    };

    // Ensure the directory exists
    if !path.exists() {
        std::fs::create_dir_all(&path).expect("Failed to create asset directory");
    }

    path
    // ✔ macOS → ~/Library/Application Support/MyApp
    // ✔ Linux → ~/.local/share/myapp   (respects XDG_DATA_HOME)
    // ✔ Windows → %APPDATA%\Example\MyApp
}

pub fn config_path() -> std::path::PathBuf {
    asset_dir().join("config.json")
}

pub fn profiles_path() -> std::path::PathBuf {
    asset_dir().join("profiles.json")
}

#[derive(RustEmbed)]
#[folder = "../../assets/sounds"]
pub struct SoundAssets;

#[derive(RustEmbed)]
#[folder = "../../assets/scripts"]
pub struct ScriptAssets;
