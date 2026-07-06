use crate::config;
use crate::model::{AppConfig, ModelSettings, scan_model_files, model_dir_from_server_path};
use crate::server_manager::{ServerEvent, ServerManager};
use std::sync::mpsc;

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

pub struct App {
    pub config: AppConfig,
    pub tab: AppTab,
    pub server_state: ServerState,
    pub model_files: Vec<crate::model::ModelFileEntry>,
    pub selected_model_idx: usize,
    pub config_selected_idx: usize,
    pub config_dirty: bool,
    pub server_manager: ServerManager,
    pub server_event_rx: Option<mpsc::Receiver<ServerEvent>>,
    pub log_lines: Vec<String>,
    pub max_log_lines: usize,
    pub show_update_popup: bool,
    pub update_output: Vec<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let mut config = config::load_config();
        let server_path = config.common.llama_server_path.clone();
        let model_files = scan_model_files(&model_dir_from_server_path(&server_path));
        config::sync_models(&mut config, &server_path);

        Self {
            config,
            tab: AppTab::Server,
            server_state: ServerState::Idle,
            model_files,
            selected_model_idx: 0,
            config_selected_idx: 0,
            config_dirty: false,
            server_manager: ServerManager::new(),
            server_event_rx: None,
            log_lines: Vec::with_capacity(1000),
            max_log_lines: 1000,
            show_update_popup: false,
            update_output: Vec::new(),
            should_quit: false,
        }
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
                    self.push_log(format!("[stderr] {line}"));
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
    }

    pub fn save_config(&mut self) -> Result<(), String> {
        config::save_config(&self.config)?;
        self.config_dirty = false;
        Ok(())
    }

    pub fn rescan_models(&mut self) {
        let server_path = self.config.common.llama_server_path.clone();
        self.model_files = scan_model_files(&model_dir_from_server_path(&server_path));
        config::sync_models(&mut self.config, &server_path);
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
