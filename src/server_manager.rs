use crate::model::{
    CommonSettings, ModelSettings, cache_dir_for_model, model_dir_from_common, server_path_for_model,
};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Merge common + per-model settings into the full llama-server argument list.
/// Only checkbox-enabled per-model settings are passed as arguments.
fn build_args(common: &CommonSettings, model: &ModelSettings) -> Vec<String> {
    let model_path = model_dir_from_common(common).join(&model.file);
    let cache_dir = cache_dir_for_model(common, model);

    let mut args: Vec<String> = vec![
        "--log-colors".into(),
        "on".into(),
        "--port".into(),
        common.port.to_string(),
        "--host".into(),
        common.host.clone(),
        "--slot-save-path".into(),
        cache_dir.to_string_lossy().to_string(),
    ];

    if model.file_enabled {
        args.extend_from_slice(&["-m".into(), model_path.to_string_lossy().to_string()]);
    }
    if model.name_enabled {
        args.extend_from_slice(&["--alias".into(), model.name.clone()]);
    }
    if model.gpu_layers_enabled {
        args.extend_from_slice(&["--n-gpu-layers".into(), model.gpu_layers.clone()]);
    }
    if model.cpu_moe_enabled {
        args.extend_from_slice(&["--n-cpu-moe".into(), model.cpu_moe.to_string()]);
    }
    if model.ctx_size_enabled {
        args.extend_from_slice(&["--ctx-size".into(), model.ctx_size.to_string()]);
    }
    if model.kv_k_enabled {
        args.extend_from_slice(&["-ctk".into(), model.kv_k.clone()]);
    }
    if model.kv_v_enabled {
        args.extend_from_slice(&["-ctv".into(), model.kv_v.clone()]);
    }
    if model.temperature_enabled {
        args.extend_from_slice(&["--temp".into(), model.temperature.to_string()]);
    }
    if model.top_p_enabled {
        args.extend_from_slice(&["--top-p".into(), model.top_p.to_string()]);
    }
    if model.top_k_enabled {
        args.extend_from_slice(&["--top-k".into(), model.top_k.to_string()]);
    }
    if model.min_p_enabled {
        args.extend_from_slice(&["--min-p".into(), model.min_p.to_string()]);
    }
    if model.repeat_penalty_enabled {
        args.extend_from_slice(&["--repeat-penalty".into(), model.repeat_penalty.to_string()]);
    }
    if model.presence_penalty_enabled {
        args.extend_from_slice(&["--presence-penalty".into(), model.presence_penalty.to_string()]);
    }
    if model.no_mmap_enabled && model.no_mmap {
        args.push("--no-mmap".into());
    }
    if model.flash_attn_enabled {
        args.extend_from_slice(&["--flash-attn".into(), model.flash_attn.clone()]);
    }
    if model.spec_type_enabled {
        args.extend_from_slice(&["--spec-type".into(), model.spec_type.clone()]);
    }
    if model.spec_draft_n_max_enabled {
        args.extend_from_slice(&["--spec-draft-n-max".into(), model.spec_draft_n_max.to_string()]);
    }
    if model.model_draft_enabled && !model.model_draft.is_empty() {
        args.extend_from_slice(&["--model-draft".into(), model.model_draft.clone()]);
    }
    if model.cache_ram_enabled {
        args.extend_from_slice(&["--cache-ram".into(), model.cache_ram.to_string()]);
    }
    if model.load_mode_enabled && !model.load_mode.is_empty() {
        args.extend_from_slice(&["--load-mode".into(), model.load_mode.clone()]);
    }
    if model.parallel_enabled {
        args.extend_from_slice(&["--parallel".into(), model.parallel.to_string()]);
    }
    if model.threads_enabled {
        args.extend_from_slice(&["--threads".into(), model.threads.to_string()]);
    }
    if model.moe_expert_cache_size_enabled {
        args.extend_from_slice(&[
            "--moe-expert-cache-size".into(),
            model.moe_expert_cache_size.to_string(),
        ]);
    }
    if model.fit_enabled {
        args.extend_from_slice(&["--fit".into(), model.fit.clone()]);
    }
    if model.kv_offload_enabled {
        args.push("--kv-offload".into());
    }
    if model.jinja_enabled {
        args.push("--jinja".into());
    }
    if model.batch_size_enabled {
        args.extend_from_slice(&["--batch-size".into(), model.batch_size.to_string()]);
    }
    if model.ubatch_size_enabled {
        args.extend_from_slice(&["--ubatch-size".into(), model.ubatch_size.to_string()]);
    }
    if model.extra_args_enabled {
        let trimmed = model.extra_args.trim();
        if !trimmed.is_empty() {
            args.extend(trimmed.split_whitespace().map(String::from));
        }
    }

    let common_trimmed = common.extra_args.trim();
    if !common_trimmed.is_empty() {
        args.extend(common_trimmed.split_whitespace().map(String::from));
    }

    args
}

/// Messages from the server thread to the UI.
#[derive(Debug, Clone)]
pub enum ServerEvent {
    StdoutLine(String),
    StderrLine(String),
    Exited(i32),
}

/// Manages the llama-server child process.
pub struct ServerManager {
    pid: Option<u32>,
    running: Arc<AtomicBool>,
}

impl ServerManager {
    pub fn new() -> Self {
        Self {
            pid: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Spawn llama-server with merged common + model settings.
    /// Returns a receiver that streams ServerEvent.
    pub fn spawn(
        &mut self,
        common: &CommonSettings,
        model: &ModelSettings,
    ) -> Result<mpsc::Receiver<ServerEvent>, String> {
        if self.running.load(Ordering::SeqCst) {
            return Err("Server is already running".to_string());
        }

        let server_path = server_path_for_model(common, model);
        let args = build_args(common, model);

        // Determine working directory (parent of server binary)
        let server_path_obj = std::path::Path::new(&server_path);
        let work_dir = server_path_obj
            .parent()
            .unwrap_or(std::path::Path::new("."));

        let mut child = Command::new(&server_path)
            .args(&args)
            .current_dir(work_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn llama-server: {e}"))?;

        let pid = child.id();
        self.pid = Some(pid);
        self.running.store(true, Ordering::SeqCst);

        let (tx, rx) = mpsc::channel();

        // stdout reader thread
        let tx_out = tx.clone();
        let stdout = child.stdout.take().expect("stdout capture");
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().flatten() {
                if tx_out.send(ServerEvent::StdoutLine(line)).is_err() {
                    break;
                }
            }
        });

        // stderr reader thread
        let tx_err = tx.clone();
        let stderr = child.stderr.take().expect("stderr capture");
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().flatten() {
                if tx_err.send(ServerEvent::StderrLine(line)).is_err() {
                    break;
                }
            }
        });

        // wait thread (detects exit) — takes ownership of child
        let tx_exit = tx.clone();
        let running = self.running.clone();
        thread::spawn(move || {
            match child.wait() {
                Ok(status) => {
                    let code = status.code().unwrap_or(-1);
                    let _ = tx_exit.send(ServerEvent::Exited(code));
                }
                Err(_) => {
                    let _ = tx_exit.send(ServerEvent::Exited(-1));
                }
            }
            running.store(false, Ordering::SeqCst);
        });

        Ok(rx)
    }

    pub fn stop(&mut self) -> Result<(), String> {
        let pid = self.pid.ok_or("No server running")?;
        self.pid = None;

        let nix_pid = nix::unistd::Pid::from_raw(pid as i32);
        let _ = nix::sys::signal::kill(nix_pid, nix::sys::signal::Signal::SIGTERM);

        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        loop {
            match nix::sys::signal::kill(nix_pid, nix::sys::signal::Signal::SIGTERM) {
                Err(nix::errno::Errno::ESRCH) => {
                    self.running.store(false, Ordering::SeqCst);
                    return Ok(());
                }
                _ => {}
            }
            if std::time::Instant::now() >= deadline {
                let _ = nix::sys::signal::kill(nix_pid, nix::sys::signal::Signal::SIGKILL);
                self.running.store(false, Ordering::SeqCst);
                return Ok(());
            }
            thread::sleep(Duration::from_millis(200));
        }
    }

    /// Check if server is running.
    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

/// Build the full llama-server argument list for display.
pub fn build_args_display(common: &CommonSettings, model: &ModelSettings) -> Vec<String> {
    build_args(common, model)
}
