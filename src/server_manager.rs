use crate::model::{CommonSettings, ModelSettings, cache_dir_from_settings, model_dir_from_server_path};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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

        let server_path = &common.llama_server_path;
        let model_dir = model_dir_from_server_path(server_path);
        let model_path = model_dir.join(&model.file);
        let cache_dir = cache_dir_from_settings(common);

        // Build args
        let mut args: Vec<String> = vec![
            "--port".into(),
            common.port.to_string(),
            "--host".into(),
            common.host.clone(),
            "--n-gpu-layers".into(),
            model.gpu_layers.to_string(),
            "--n-cpu-moe".into(),
            model.cpu_moe.to_string(),
            "--ctx-size".into(),
            model.ctx_size.to_string(),
            "--slot-save-path".into(),
            cache_dir.to_string_lossy().to_string(),
            "--no-mmap".into(),
            "--flash-attn".into(),
            "on".into(),
            "--spec-type".into(),
            "none".into(),
            "--spec-draft-n-max".into(),
            "2".into(),
            "-m".into(),
            model_path.to_string_lossy().to_string(),
            "--alias".into(),
            model.name.clone(),
            "--temp".into(),
            model.temperature.to_string(),
            "--top-p".into(),
            model.top_p.to_string(),
            "--top-k".into(),
            model.top_k.to_string(),
            "--min-p".into(),
            model.min_p.to_string(),
            "--repeat-penalty".into(),
            model.repeat_penalty.to_string(),
            "--presence-penalty".into(),
            model.presence_penalty.to_string(),
            "-ctk".into(),
            model.kv_k.clone(),
            "-ctv".into(),
            model.kv_v.clone(),
        ];

        // Append extra args if present
        let trimmed = common.extra_args.trim();
        if !trimmed.is_empty() {
            args.extend(trimmed.split_whitespace().map(String::from));
        }

        // Determine working directory (parent of server binary)
        let server_path_obj = std::path::Path::new(server_path);
        let work_dir = server_path_obj
            .parent()
            .unwrap_or(std::path::Path::new("."));

        let mut child = Command::new(server_path)
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
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}
