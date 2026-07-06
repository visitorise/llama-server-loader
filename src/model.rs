use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ── Common settings ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonSettings {
    pub llama_server_path: String,
    pub host: String,
    pub port: u16,
    pub cache_dir: String,
    pub mid_pane_height: u16,
    /// Path to llama-server-update.sh
    #[serde(default = "default_update_script")]
    pub update_script_path: String,
    /// Launch nvtop in a separate terminal when server starts
    #[serde(default)]
    pub nvtop_enabled: bool,
    /// nvtop command
    #[serde(default = "default_nvtop_cmd")]
    pub nvtop_cmd: String,
    /// Terminal emulator for nvtop
    #[serde(default = "default_terminal_cmd")]
    pub terminal_cmd: String,
    /// Pass --no-mmap to llama-server
    #[serde(default = "default_true")]
    pub no_mmap: bool,
    /// Value for --flash-attn (on/off)
    #[serde(default = "default_flash_attn")]
    pub flash_attn: String,
    /// Value for --spec-type (none/default)
    #[serde(default = "default_spec_type")]
    pub spec_type: String,
    /// Value for --spec-draft-n-max
    #[serde(default = "default_spec_draft")]
    pub spec_draft_n_max: u32,
    #[serde(default)]
    pub extra_args: String,
    /// Explicit model directory. If empty, derived from llama_server_path.
    #[serde(default)]
    pub model_dir: String,
}

fn default_update_script() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{home}/Develop/llamacpp_loader/llama-server-loader/llama-server-update.sh")
}
fn default_nvtop_cmd() -> String {
    "nvtop".to_string()
}
fn default_terminal_cmd() -> String {
    "x-terminal-emulator".to_string()
}
fn default_true() -> bool {
    true
}
fn default_flash_attn() -> String {
    "on".to_string()
}
fn default_spec_type() -> String {
    "none".to_string()
}
fn default_spec_draft() -> u32 {
    2
}

impl Default for CommonSettings {
    fn default() -> Self {
        Self {
            llama_server_path: "llama-server".to_string(),
            host: "0.0.0.0".to_string(),
            port: 11400,
            cache_dir: String::new(),
            mid_pane_height: 21,
            update_script_path: default_update_script(),
            nvtop_enabled: false,
            nvtop_cmd: default_nvtop_cmd(),
            terminal_cmd: default_terminal_cmd(),
            no_mmap: true,
            flash_attn: default_flash_attn(),
            spec_type: default_spec_type(),
            spec_draft_n_max: 2,
            extra_args: String::new(),
            model_dir: String::new(),
        }
    }
}

// ── Per-model settings ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSettings {
    pub name: String,
    pub file: String,
    #[serde(default = "default_gpu_layers")]
    pub gpu_layers: u32,
    #[serde(default = "default_ctx_size")]
    pub ctx_size: u32,
    #[serde(default = "default_kv")]
    pub kv_k: String,
    #[serde(default = "default_kv")]
    pub kv_v: String,
    #[serde(default)]
    pub cpu_moe: u32,
    #[serde(default = "default_temp")]
    pub temperature: f32,
    #[serde(default = "default_top_k")]
    pub top_k: u32,
    #[serde(default = "default_top_p")]
    pub top_p: f32,
    #[serde(default)]
    pub min_p: f32,
    #[serde(default = "default_penalty")]
    pub repeat_penalty: f32,
    #[serde(default)]
    pub presence_penalty: f32,
}

fn default_gpu_layers() -> u32 {
    75
}
fn default_ctx_size() -> u32 {
    262144
}
fn default_kv() -> String {
    "q8_0".to_string()
}
fn default_temp() -> f32 {
    1.0
}
fn default_top_k() -> u32 {
    40
}
fn default_top_p() -> f32 {
    0.95
}
fn default_penalty() -> f32 {
    1.0
}

impl Default for ModelSettings {
    fn default() -> Self {
        Self {
            name: String::new(),
            file: String::new(),
            gpu_layers: default_gpu_layers(),
            ctx_size: default_ctx_size(),
            kv_k: default_kv(),
            kv_v: default_kv(),
            cpu_moe: 0,
            temperature: default_temp(),
            top_k: default_top_k(),
            top_p: default_top_p(),
            min_p: 0.0,
            repeat_penalty: default_penalty(),
            presence_penalty: 0.0,
        }
    }
}

// ── Top-level config ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub common: CommonSettings,
    #[serde(default)]
    pub models: Vec<ModelSettings>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            common: CommonSettings::default(),
            models: Vec::new(),
        }
    }
}

// ── Model file scanner ──

#[derive(Debug, Clone)]
pub struct ModelFileEntry {
    pub file_name: String,
    pub file_path: PathBuf,
}

/// Scan a directory for .gguf files, return sorted list.
pub fn scan_model_files(model_dir: &Path) -> Vec<ModelFileEntry> {
    let mut entries = Vec::new();
    if !model_dir.is_dir() {
        return entries;
    }
    if let Ok(read_dir) = std::fs::read_dir(model_dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "gguf") {
                let file_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                entries.push(ModelFileEntry {
                    file_name,
                    file_path: path,
                });
            }
        }
    }
    entries.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    entries
}

/// Derive model directory: use explicit model_dir if set, else derive from server path.
pub fn model_dir_from_common(common: &CommonSettings) -> PathBuf {
    if !common.model_dir.is_empty() {
        let p = Path::new(&common.model_dir);
        if p.is_absolute() {
            return p.to_path_buf();
        }
    }
    model_dir_from_server_path(&common.llama_server_path)
}

/// Derive model directory from llama_server_path alone (fallback).
pub fn model_dir_from_server_path(server_path: &str) -> PathBuf {
    let p = Path::new(server_path);
    let parent = p.parent().unwrap_or(Path::new("."));
    parent.join("..").join("model")
}

/// Derive cache directory from common settings.
pub fn cache_dir_from_settings(common: &CommonSettings) -> PathBuf {
    if !common.cache_dir.is_empty() {
        let p = Path::new(&common.cache_dir);
        if p.is_absolute() {
            return p.to_path_buf();
        }
    }
    let server = Path::new(&common.llama_server_path);
    let parent = server.parent().unwrap_or(Path::new("."));
    parent.join("..").join("cache")
}
