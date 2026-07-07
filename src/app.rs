use crate::config;
use crate::model::{AppConfig, GpuMetrics, ModelSettings, scan_model_files, model_dir_from_common, GPU_HISTORY_LEN};
use crate::server_manager::{ServerEvent, ServerManager};
use std::collections::VecDeque;

use std::sync::mpsc;
use std::time::{Duration, Instant};
use tui_input::Input;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppTab {
    Server,
    Configure,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServerState {
    Idle,
    Running,
}

/// Which section is focused in the Configure tab.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigSection {
    Common,
    ModelList,
    ModelSettings,
}

/// Edit state for the Configure tab.
pub struct ConfigEdit {
    pub section: ConfigSection,
    pub common_idx: usize,
    pub model_list_idx: usize,
    pub model_field_idx: usize,
    pub editing: bool,
    pub input: Input,
    pub common_scroll: u16,
    pub model_scroll: u16,
}

const PANEL_VISIBLE_COMMON: u16 = 7;
const PANEL_VISIBLE_MODEL: u16 = 11;

impl ConfigEdit {
    pub fn new() -> Self {
        Self {
            section: ConfigSection::Common,
            common_idx: 0,
            model_list_idx: 0,
            model_field_idx: 0,
            editing: false,
            input: Input::default(),
            common_scroll: 0,
            model_scroll: 0,
        }
    }

    /// Adjust scroll offsets so the currently selected field is visible.
    pub fn follow_selection(&mut self) {
        match self.section {
            ConfigSection::Common => {
                let sel = self.common_idx as u16;
                if sel < self.common_scroll {
                    self.common_scroll = sel;
                } else if sel >= self.common_scroll + PANEL_VISIBLE_COMMON {
                    self.common_scroll = sel - PANEL_VISIBLE_COMMON + 1;
                }
            }
            ConfigSection::ModelSettings => {
                let sel = self.model_field_idx as u16;
                if sel < self.model_scroll {
                    self.model_scroll = sel;
                } else if sel >= self.model_scroll + PANEL_VISIBLE_MODEL {
                    self.model_scroll = sel - PANEL_VISIBLE_MODEL + 1;
                }
            }
            _ => {}
        }
    }
}

pub struct App {
    pub config: AppConfig,
    pub tab: AppTab,
    pub server_state: ServerState,
    #[allow(dead_code)]
    pub model_files: Vec<crate::model::ModelFileEntry>,
    pub selected_model_idx: usize,
    pub config_edit: ConfigEdit,
    #[allow(dead_code)]
    pub config_dirty: bool,
    pub server_manager: ServerManager,
    pub server_event_rx: Option<mpsc::Receiver<ServerEvent>>,
    pub log_lines: Vec<String>,
    pub max_log_lines: usize,
    pub log_scroll: u16,
    pub log_auto_scroll: bool,
    pub show_update_popup: bool,
    pub update_output: Vec<String>,
    pub should_quit: bool,
    /// NVML handle. `None` means no NVIDIA driver / NVML not available.
    nvml: Option<nvml_wrapper::Nvml>,
    pub gpu_metrics: Vec<GpuMetrics>,
    pub gpu_util_history: VecDeque<u32>,
    pub gpu_mem_history: VecDeque<u32>,
    last_gpu_poll: Instant,
    gpu_poll_interval: Duration,
    pub gpu_available: bool,
    pub graph_mode: bool,
    pub mouse_select_start: Option<(u16, u16)>,
    pub mouse_select_end: Option<(u16, u16)>,
}

impl App {
    pub fn new() -> Self {
        let mut config = config::load_config();

        // Resolve model_dir using the fallback if empty, then persist.
        {
            let resolved = model_dir_from_common(&config.common);
            if config.common.model_dir.is_empty() {
                config.common.model_dir = resolved.to_string_lossy().to_string();
            }
        }

        let common = config.common.clone();
        let model_files = scan_model_files(&model_dir_from_common(&common));
        config::sync_models(&mut config, &common);
        let _ = config::save_config(&config);

        // Try initialising NVML at startup (runtime-load of libnvidia-ml.so)
        let (nvml, gpu_available) = match nvml_wrapper::Nvml::init() {
            Ok(nvml) => (Some(nvml), true),
            Err(_) => (None, false),
        };

        let app = Self {
            config,
            tab: AppTab::Server,
            server_state: ServerState::Idle,
            model_files,
            selected_model_idx: 0,
            config_edit: ConfigEdit::new(),
            config_dirty: false,
            server_manager: ServerManager::new(),
            server_event_rx: None,
            log_lines: Vec::with_capacity(1000),
            max_log_lines: 1000,
            log_scroll: 0,
            log_auto_scroll: true,
            show_update_popup: false,
            update_output: Vec::new(),
            should_quit: false,
            nvml,
            gpu_metrics: Vec::new(),
            gpu_util_history: VecDeque::with_capacity(GPU_HISTORY_LEN),
            gpu_mem_history: VecDeque::with_capacity(GPU_HISTORY_LEN),
            last_gpu_poll: Instant::now(),
            gpu_poll_interval: Duration::from_secs(1),
            gpu_available,
            graph_mode: false,
            mouse_select_start: None,
            mouse_select_end: None,
        };

        app
    }

    pub fn selected_model_settings(&self) -> Option<&ModelSettings> {
        if self.selected_model_idx < self.config.models.len() {
            Some(&self.config.models[self.selected_model_idx])
        } else {
            None
        }
    }

    pub fn select_model(&mut self, idx: usize) {
        if idx < self.config.models.len() {
            self.selected_model_idx = idx;
        }
    }

    pub fn start_server(&mut self) -> Result<(), String> {
        let model = self.selected_model_settings().ok_or("No model selected")?.clone();
        let rx = self.server_manager.spawn(&self.config.common, &model)?;
        self.server_event_rx = Some(rx);
        self.server_state = ServerState::Running;
        self.log_lines.push(format!(
            "[{}] llama-server started: {}",
            chrono_now(),
            model.name
        ));
        Ok(())
    }

    pub fn stop_server(&mut self) -> Result<(), String> {
        self.server_manager.stop()?;
        self.server_state = ServerState::Idle;
        self.server_event_rx = None;
        self.log_lines.push(format!("[{}] llama-server stopped", chrono_now()));
        Ok(())
    }

    pub fn poll_server_events(&mut self) {
        let rx = match self.server_event_rx.take() {
            Some(rx) => rx,
            None => return,
        };

        let mut disconnected = false;
        let mut exited = false;

        loop {
            match rx.try_recv() {
                Ok(ServerEvent::StdoutLine(line)) => {
                    self.push_log(line);
                }
                Ok(ServerEvent::StderrLine(line)) => {
                    self.push_log(line);
                }
                Ok(ServerEvent::Exited(code)) => {
                    self.push_log(format!(
                        "[{}] llama-server exited with code {code}",
                        chrono_now()
                    ));
                    exited = true;
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    disconnected = true;
                    break;
                }
            }
        }

        if exited || disconnected {
            self.server_state = ServerState::Idle;
        } else {
            self.server_event_rx = Some(rx);
        }
    }

    fn push_log(&mut self, line: String) {
        self.log_lines.push(line);
        if self.log_lines.len() > self.max_log_lines {
            self.log_lines.remove(0);
        }
        if self.log_auto_scroll {
            self.log_scroll = 0;
        }
    }

    /// Scroll the log view up by `n` logical lines and disable auto-scroll.
    pub fn scroll_up(&mut self, n: u16) {
        self.log_scroll = self.log_scroll.saturating_add(n);
        self.log_auto_scroll = false;
    }

    /// Scroll the log view down by `n` logical lines. If we reach the bottom,
    /// re-enable auto-scroll.
    pub fn scroll_down(&mut self, n: u16) {
        self.log_scroll = self.log_scroll.saturating_sub(n);
        if self.log_scroll == 0 {
            self.log_auto_scroll = true;
        }
    }

    #[allow(dead_code)]
    pub fn save_config(&mut self) -> Result<(), String> {
        config::save_config(&self.config)?;
        self.config_dirty = false;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn rescan_models(&mut self) {
        let common = self.config.common.clone();
        self.model_files = scan_model_files(&model_dir_from_common(&common));
        config::sync_models(&mut self.config, &common);
    }

    /// Poll all GPU metrics via NVML and update self.gpu_metrics + history.
    /// Called once per main-loop iteration (rate-limited internally).
    pub fn poll_gpu(&mut self) {
        let nvml = match self.nvml.as_ref() {
            Some(nvml) => nvml,
            None => return,
        };
        if self.last_gpu_poll.elapsed() < self.gpu_poll_interval {
            return;
        }
        self.last_gpu_poll = Instant::now();

        let count = match nvml.device_count() {
            Ok(c) => c,
            Err(_) => return,
        };

        let mut fresh: Vec<GpuMetrics> = Vec::with_capacity(count as usize);
        for i in 0..count {
            if let Ok(device) = nvml.device_by_index(i) {
                let m = GpuMetrics::from_device(&device, i, &mut self.gpu_util_history, &mut self.gpu_mem_history);
                fresh.push(m);
            }
        }
        self.gpu_metrics = fresh;
    }
}

fn chrono_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}
