use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
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
            mid_pane_height: 19,
            update_script_path: default_update_script(),

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
    #[serde(default = "default_true")]
    pub name_enabled: bool,
    pub name: String,
    #[serde(default = "default_true")]
    pub file_enabled: bool,
    pub file: String,
    #[serde(default = "default_true")]
    pub gpu_layers_enabled: bool,
    #[serde(default = "default_gpu_layers")]
    pub gpu_layers: String,
    #[serde(default = "default_true")]
    pub ctx_size_enabled: bool,
    #[serde(default = "default_ctx_size")]
    pub ctx_size: u32,
    #[serde(default = "default_true")]
    pub kv_k_enabled: bool,
    #[serde(default = "default_kv")]
    pub kv_k: String,
    #[serde(default = "default_true")]
    pub kv_v_enabled: bool,
    #[serde(default = "default_kv")]
    pub kv_v: String,
    #[serde(default = "default_true")]
    pub cpu_moe_enabled: bool,
    #[serde(default)]
    pub cpu_moe: u32,
    #[serde(default = "default_true")]
    pub temperature_enabled: bool,
    #[serde(default = "default_temp")]
    pub temperature: f32,
    #[serde(default = "default_true")]
    pub top_k_enabled: bool,
    #[serde(default = "default_top_k")]
    pub top_k: u32,
    #[serde(default = "default_true")]
    pub top_p_enabled: bool,
    #[serde(default = "default_top_p")]
    pub top_p: f32,
    #[serde(default = "default_true")]
    pub min_p_enabled: bool,
    #[serde(default)]
    pub min_p: f32,
    #[serde(default = "default_true")]
    pub repeat_penalty_enabled: bool,
    #[serde(default = "default_penalty")]
    pub repeat_penalty: f32,
    #[serde(default = "default_true")]
    pub presence_penalty_enabled: bool,
    #[serde(default)]
    pub presence_penalty: f32,
    #[serde(default = "default_true")]
    pub extra_args_enabled: bool,
    #[serde(default)]
    pub extra_args: String,

    // ── Moved from CommonSettings (checkbox-enabled) ──
    /// Whether to use per-model llama_server_path override. Empty = master.
    #[serde(default)]
    pub llama_server_path_enabled: bool,
    /// Per-model llama_server_path override. If empty, uses common llama_server_path.
    #[serde(default)]
    pub llama_server_path: String,
    /// Whether to use per-model cache_dir override.
    #[serde(default = "default_true")]
    pub cache_dir_enabled: bool,
    /// Per-model cache directory override.
    #[serde(default)]
    pub cache_dir: String,
    /// Whether to pass --no-mmap
    #[serde(default = "default_true")]
    pub no_mmap_enabled: bool,
    #[serde(default = "default_true")]
    pub no_mmap: bool,
    /// Whether to pass --flash-attn
    #[serde(default = "default_true")]
    pub flash_attn_enabled: bool,
    #[serde(default = "default_flash_attn")]
    pub flash_attn: String,
    /// Whether to pass --spec-type
    #[serde(default = "default_true")]
    pub spec_type_enabled: bool,
    #[serde(default = "default_spec_type")]
    pub spec_type: String,
    /// Whether to pass --spec-draft-n-max
    #[serde(default = "default_true")]
    pub spec_draft_n_max_enabled: bool,
    #[serde(default = "default_spec_draft")]
    pub spec_draft_n_max: u32,

    // ── New per-model settings (checkbox-enabled) ──
    /// Whether to pass --model-draft
    #[serde(default)]
    pub model_draft_enabled: bool,
    /// Path to draft model for speculative decoding.
    #[serde(default)]
    pub model_draft: String,
    /// Whether to pass --cache-ram
    #[serde(default)]
    pub cache_ram_enabled: bool,
    #[serde(default)]
    pub cache_ram: u64,
    /// Whether to pass --load-mode
    #[serde(default)]
    pub load_mode_enabled: bool,
    /// Value for --load-mode.
    #[serde(default)]
    pub load_mode: String,
    /// Whether to pass --parallel
    #[serde(default)]
    pub parallel_enabled: bool,
    #[serde(default = "default_parallel")]
    pub parallel: u32,
    /// Whether to pass --threads
    #[serde(default)]
    pub threads_enabled: bool,
    #[serde(default)]
    pub threads: u32,
    /// Whether to pass --moe-expert-cache-size
    #[serde(default)]
    pub moe_expert_cache_size_enabled: bool,
    #[serde(default)]
    pub moe_expert_cache_size: u32,
    /// Whether to pass --fit
    #[serde(default)]
    pub fit_enabled: bool,
    /// Value for --fit (on/off, default: on)
    #[serde(default = "default_fit")]
    pub fit: String,
    /// Whether to pass --kv-offload (pure flag, no value)
    #[serde(default = "default_true")]
    pub kv_offload_enabled: bool,
    /// Whether to pass --jinja (pure flag, no value)
    #[serde(default)]
    pub jinja_enabled: bool,
    /// Whether to pass --batch-size
    #[serde(default)]
    pub batch_size_enabled: bool,
    #[serde(default = "default_batch_size")]
    pub batch_size: u32,
    /// Whether to pass --ubatch-size
    #[serde(default)]
    pub ubatch_size_enabled: bool,
    #[serde(default = "default_ubatch_size")]
    pub ubatch_size: u32,
}

fn default_gpu_layers() -> String {
    "75".to_string()
}
fn default_ctx_size() -> u32 {
    262144
}
fn default_kv() -> String {
    "q8_0".to_string()
}
fn default_fit() -> String {
    "on".to_string()
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
fn default_parallel() -> u32 {
    1
}
fn default_batch_size() -> u32 {
    2048
}
fn default_ubatch_size() -> u32 {
    512
}

impl Default for ModelSettings {
    fn default() -> Self {
        Self {
            name_enabled: true,
            name: String::new(),
            file_enabled: true,
            file: String::new(),
            gpu_layers_enabled: true,
            gpu_layers: default_gpu_layers(),
            ctx_size_enabled: true,
            ctx_size: default_ctx_size(),
            kv_k_enabled: true,
            kv_k: default_kv(),
            kv_v_enabled: true,
            kv_v: default_kv(),
            cpu_moe_enabled: true,
            cpu_moe: 0,
            temperature_enabled: true,
            temperature: default_temp(),
            top_k_enabled: true,
            top_k: default_top_k(),
            top_p_enabled: true,
            top_p: default_top_p(),
            min_p_enabled: true,
            min_p: 0.0,
            repeat_penalty_enabled: true,
            repeat_penalty: default_penalty(),
            presence_penalty_enabled: true,
            presence_penalty: 0.0,
            extra_args_enabled: true,
            extra_args: String::new(),

            llama_server_path_enabled: false,
            llama_server_path: String::new(),
            cache_dir_enabled: true,
            cache_dir: String::new(),
            no_mmap_enabled: true,
            no_mmap: true,
            flash_attn_enabled: true,
            flash_attn: default_flash_attn(),
            spec_type_enabled: true,
            spec_type: default_spec_type(),
            spec_draft_n_max_enabled: true,
            spec_draft_n_max: default_spec_draft(),

            model_draft_enabled: false,
            model_draft: String::new(),
            cache_ram_enabled: false,
            cache_ram: 0,
            load_mode_enabled: false,
            load_mode: String::new(),
            parallel_enabled: false,
            parallel: default_parallel(),
            threads_enabled: false,
            threads: 0,
            moe_expert_cache_size_enabled: false,
            moe_expert_cache_size: 0,
            fit_enabled: false,
            fit: default_fit(),
            kv_offload_enabled: true,
            jinja_enabled: false,
            batch_size_enabled: false,
            batch_size: default_batch_size(),
            ubatch_size_enabled: false,
            ubatch_size: default_ubatch_size(),
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
    #[allow(dead_code)]
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

/// Derive model directory: use per-model override if set, else common, else derive from server path.
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

/// Derive cache directory for a specific model (per-model override takes priority).
pub fn cache_dir_for_model(common: &CommonSettings, model: &ModelSettings) -> PathBuf {
    if model.cache_dir_enabled && !model.cache_dir.is_empty() {
        let p = Path::new(&model.cache_dir);
        if p.is_absolute() {
            return p.to_path_buf();
        }
    }
    cache_dir_from_settings(common)
}

/// Resolve effective llama_server_path (per-model override or common).
pub fn server_path_for_model(common: &CommonSettings, model: &ModelSettings) -> String {
    if model.llama_server_path_enabled && !model.llama_server_path.is_empty() {
        model.llama_server_path.clone()
    } else {
        common.llama_server_path.clone()
    }
}

// ── GPU metrics (from NVML polling) ──

#[derive(Debug, Clone, Default)]
pub struct GpuMetrics {
    pub index: u32,
    pub name: String,
    pub gpu_util: u32,
    pub mem_used_mb: f64,
    pub mem_total_mb: f64,
    pub mem_util: u32,
    pub temp: u32,
    pub power_draw: f64,
    pub power_limit: f64,
    pub gpu_clock: u32,
    pub mem_clock: u32,
    pub fan_speed: u32,
    pub pstate: String,
    #[allow(dead_code)]
    pub encoder_util: u32,
    #[allow(dead_code)]
    pub decoder_util: u32,
    #[allow(dead_code)]
    pub util_history: VecDeque<u32>,
    pub pcie_link_gen: u32,
    pub pcie_link_width: u32,
    pub pcie_rx_kbps: u32,
    pub pcie_tx_kbps: u32,
}

pub const GPU_HISTORY_LEN: usize = 60;

impl GpuMetrics {
    pub fn from_device(
        device: &nvml_wrapper::Device,
        index: u32,
        history: &mut VecDeque<u32>,
        mem_history: &mut VecDeque<u32>,
    ) -> Self {
        use nvml_wrapper::enum_wrappers::device::{Clock, PcieUtilCounter, TemperatureSensor};

        let name = device.name().unwrap_or_default();

        let util = device.utilization_rates().unwrap_or(
            nvml_wrapper::struct_wrappers::device::Utilization { gpu: 0, memory: 0 },
        );
        let mem = device.memory_info().unwrap_or(
            nvml_wrapper::struct_wrappers::device::MemoryInfo {
                total: 0,
                free: 0,
                used: 0,
                reserved: 0,
                version: 0,
            },
        );
        let temp = device
            .temperature(TemperatureSensor::Gpu)
            .unwrap_or(0);
        let power = device.power_usage().unwrap_or(0);
        let gpu_clock = device.clock_info(Clock::Graphics).unwrap_or(0);
        let mem_clock = device.clock_info(Clock::Memory).unwrap_or(0);
        let fan = device.fan_speed(0).unwrap_or(0);
        let power_limit = device.enforced_power_limit().unwrap_or(0);

        let enc = device
            .encoder_utilization()
            .map(|u| u.utilization)
            .unwrap_or(0);
        let dec = device
            .decoder_utilization()
            .map(|u| u.utilization)
            .unwrap_or(0);

        let pstate = device
            .performance_state()
            .map(|ps| format!("P{}", ps as u32))
            .unwrap_or_default();

        let pcie_gen = device.current_pcie_link_gen().unwrap_or(0);
        let pcie_width = device.current_pcie_link_width().unwrap_or(0);
        let pcie_rx = device.pcie_throughput(PcieUtilCounter::Receive).unwrap_or(0);
        let pcie_tx = device.pcie_throughput(PcieUtilCounter::Send).unwrap_or(0);

        if history.len() >= GPU_HISTORY_LEN {
            history.pop_front();
        }
        history.push_back(util.gpu);

        if mem_history.len() >= GPU_HISTORY_LEN {
            mem_history.pop_front();
        }
        let mem_used_pct = if mem.total > 0 {
            ((mem.used + mem.reserved) * 100 / mem.total) as u32
        } else {
            0
        };
        mem_history.push_back(mem_used_pct);

        Self {
            index,
            name,
            gpu_util: util.gpu,
            mem_used_mb: (mem.used + mem.reserved) as f64 / 1_048_576.0,
            mem_total_mb: mem.total as f64 / 1_048_576.0,
            mem_util: mem_used_pct,
            temp,
            power_draw: power as f64 / 1000.0,
            power_limit: power_limit as f64 / 1000.0,
            gpu_clock,
            mem_clock,
            fan_speed: fan,
            pstate,
            encoder_util: enc,
            decoder_util: dec,
            util_history: history.clone(),
            pcie_link_gen: pcie_gen,
            pcie_link_width: pcie_width,
            pcie_rx_kbps: pcie_rx,
            pcie_tx_kbps: pcie_tx,
        }
    }
}
