use crate::model::{AppConfig, CommonSettings, ModelSettings, scan_model_files, model_dir_from_common};
use directories::ProjectDirs;
use std::collections::HashMap;
use std::path::PathBuf;

const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "llamaloader";
const APPLICATION: &str = "llama-server-loader";

/// Get config file path (~/.config/llama-server-loader/config.json).
pub fn config_file_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
        let config_dir = proj_dirs.config_dir().to_path_buf();
        std::fs::create_dir_all(&config_dir).ok();
        config_dir.join("config.json")
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let p = PathBuf::from(home).join(".config").join(APPLICATION);
        std::fs::create_dir_all(&p).ok();
        p.join("config.json")
    }
}

/// Load config from file. If file doesn't exist, return default.
/// Always saves after loading so the file stays up-to-date with new fields.
pub fn load_config() -> AppConfig {
    let config = inner_load();
    // Re-save to persist any new default fields that serde filled in
    let _ = save_config(&config);
    config
}

fn inner_load() -> AppConfig {
    let path = config_file_path();
    if !path.exists() {
        return AppConfig::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            serde_json::from_str(&content).unwrap_or_else(|e| {
                eprintln!("Warning: config parse error ({e}), using defaults");
                AppConfig::default()
            })
        }
        Err(e) => {
            eprintln!("Warning: could not read config ({e}), using defaults");
            AppConfig::default()
        }
    }
}

/// Save config to file.
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_file_path();
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(())
}

/// Sync models: merge saved config models with files on disk.
/// Matches by `file` field. New files get `ModelSettings::default()` with name/file set.
pub fn sync_models(config: &mut AppConfig, common: &CommonSettings) {
    let model_dir = model_dir_from_common(common);
    let disk_files = scan_model_files(&model_dir);

    let mut model_map: HashMap<String, ModelSettings> = config
        .models
        .drain(..)
        .map(|m| (m.file.clone(), m))
        .collect();

    let mut merged: Vec<ModelSettings> = Vec::new();
    for entry in &disk_files {
        if let Some(existing) = model_map.remove(&entry.file_name) {
            merged.push(existing);
        } else {
            let name = entry
                .file_name
                .strip_suffix(".gguf")
                .unwrap_or(&entry.file_name)
                .to_string();
            merged.push(ModelSettings {
                name,
                file: entry.file_name.clone(),
                ..Default::default()
            });
        }
    }
    config.models = merged;
}
