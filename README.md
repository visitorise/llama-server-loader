# llama-server-loader

A TUI (Terminal UI) launcher for managing llama-server execution. Replaces multiple shell scripts with a single configuration-driven binary.

![Version](https://img.shields.io/badge/version-0.3.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-Linux-lightgrey)
![한국어](https://github.com/visitorise/llama-server-loader/blob/master/README.ko.md)

## Features

### 🖥️ TUI Interface
- High-performance terminal UI built with **ratatui**
- **Mouse support**: Click, drag (text selection → clipboard copy), scroll
- **Real-time logs**: Color-distinguished server stdout/stderr display
- **GPU monitoring**: Braille graphs for real-time GPU Utilization/Memory display

### ⚙️ Configuration Management
- JSON configuration file (`~/.config/llama-server-loader/config.json`)
- **Common settings**: llama-server path, host, port, model directory, extra arguments, etc.
- **Per-model settings with checkboxes**: Every parameter (GPU layers, context size, KV cache quantization, sampling, server path, etc.) has a `[x]` checkbox to enable/disable it per model. Press `Space` to toggle.
- **Per-model `llama_server_path` override**: Set a custom server binary per model. When unchecked, the common path is used (shown as a fallback in the editor).
- **GPU layers as free-form text**: Supports `auto`, `all`, or a number (e.g. `75`).
- **Model directory rescan**: Editing the common model path automatically rescans the model list.
- Direct configuration editing in TUI (Configure tab)

### 🚀 Server Management
- **One-click execution**: Select model and press Enter/r to start server
- **Graceful Shutdown**: SIGTERM → 20s wait → SIGKILL
- **Log streaming**: Real-time log display with auto-scroll and manual scroll support
- **LLama Args popup**: Preview execution arguments — shows the resolved server binary path and one parameter per line, matching exactly what gets spawned

### 📊 GPU Monitoring (inspired by nvtop)
- NVIDIA GPU Utilization/Memory braille graphs
- Real-time GPU metrics display (temperature, power, memory usage)
- **nvtop-style** visualization — braille character-based graphs

### 🔄 Auto Update
- Download latest llama.cpp binaries from GitHub Release
- Automatic GPU backend detection (Vulkan/ROCm/CUDA)
- Automatic backup and replacement

## Requirements

- Rust 2021 edition (for building)
- `llama-server` binary (from llama.cpp project)
- Linux (uses Unix signal/process API)
- NVIDIA GPU: NVML library (included with nvidia-driver)

## Build

```bash
cd llama-server-loader
cargo build --release
```

Built binary: `target/release/llama-server-loader`

## Usage

```bash
./llama-server-loader
```

### Screen Layout

- **Top tab bar**: Switch between Server / Configure tabs, version display
- **Server tab**: Model list selection + server control buttons (Run, Stop, Llama Args, Exit)
- **Configure tab**: Common settings and per-model settings editor (each per-model parameter has a `[x]` checkbox; `Space` to enable/disable, `Enter` to edit the value)
- **GPU Monitoring (middle)**: Real-time GPU Utilization/Memory braille graphs (nvtop-style)
- **Log panel (bottom)**: Server stdout/stderr real-time output

### Keyboard Shortcuts

**Server Tab:**

| Key | Action |
|---|------|
| `↑` / `k` | Move model selection up |
| `↓` / `j` | Move model selection down |
| `Enter` / `r` | Start server with selected model |
| `s` | Stop running server |
| `l` | Show Llama Args popup |
| `Tab` | Switch to Configure tab |
| `q` / `Esc` | Quit |

**Configure Tab:**

| Key | Action |
|---|------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Enter` / `e` | Toggle edit mode |
| `Space` | Toggle checkbox (enable/disable a per-model parameter) |
| `c` | Check for updates (GitHub Release) |
| `Tab` | Switch to Server tab |

### Mouse Support

| Action | Function |
|------|------|
| **Click** | Execute buttons, switch tabs, select config items, select models |
| **Drag** | Select text → auto clipboard copy (wl-copy) |
| **Scroll** | Scroll log/config areas |
| **Click during popup** | Blocked (clipboard copy still works) |

## Configuration

Config file: `~/.config/llama-server-loader/config.json`

Default config is created automatically on first run.

> **⚠️ Restart required after initial setup**: After configuring `llama_server_path` and `model_dir` on first run, you must restart the app for the model list to load and enable detailed configuration.

### Common Settings

| Item | Default | Description |
|------|--------|------|
| `llama_server_path` | `llama-server` | Full path to llama-server executable (including command). Example: `/home/user/AIAgent/llama.cpp/llama_cpp/llama-server` |
| `host` | `0.0.0.0` | Server binding IP |
| `port` | `11400` | Server port |
| `model_dir` | `""` (auto-detect) | Model files directory. Editing this rescans the model list automatically |
| `cache_dir` | `""` (auto-detect) | Server cache directory (used for `--slot-save-path`) |
| `extra_args` | `""` | Additional llama-server arguments applied to every model |
| `mid_pane_height` | `19` | Middle panel (GPU graph) height |
| `update_script_path` | auto | Path to `llama-server-update.sh` |

> **Note**: `no_mmap`, `flash_attn`, `spec_type`, and `spec_draft_n_max` were moved from common settings to **per-model** settings (checkbox-enabled). Each model can now control these independently.

### Model Settings

Each parameter has a `[x]` checkbox. **Only checkbox-enabled parameters are passed to llama-server.** Press `Space` to toggle.

| Item | Default | CLI Flag | Description |
|------|--------|------|------|
| `llama_server_path` | `""` (common) | (binary path) | Per-model server binary override. Unchecked → uses common path |
| `cache_dir` | `""` (common) | `--slot-save-path` | Per-model cache directory override |
| `model` | - | `-m` | `.gguf` filename (auto-filled from the model list) |
| `model_draft` | `""` | `--model-draft` | Draft model path for speculative decoding |
| `alias` | filename | `--alias` | Model display name |
| `spec_type` | `none` | `--spec-type` | Speculative decoding type |
| `spec_draft_n_max` | `2` | `--spec-draft-n-max` | Max speculative drafting count |
| `gpu_layers` | `75` | `--n-gpu-layers` | GPU offload layer count — accepts `auto`, `all`, or a number |
| `cpu_moe` | `0` | `--n-cpu-moe` | CPU MoE expert layer count |
| `threads` | `0` (auto) | `--threads` | Number of threads |
| `moe_expert_cache_size` | `0` (auto) | `--moe-expert-cache-size` | MoE expert cache size |
| `ctx_size` | `262144` | `--ctx-size` | Context size (tokens) |
| `kv_k` | `q8_0` | `-ctk` | KV Cache Key quantization |
| `kv_v` | `q8_0` | `-ctv` | KV Cache Value quantization |
| `temperature` | `1.0` | `--temp` | Sampling temperature |
| `top_k` | `40` | `--top-k` | Top-K sampling |
| `top_p` | `0.95` | `--top-p` | Top-P (nucleus) sampling |
| `min_p` | `0.0` | `--min-p` | Min-P sampling |
| `repeat_penalty` | `1.0` | `--repeat-penalty` | Repeat penalty |
| `presence_penalty` | `0.0` | `--presence-penalty` | Presence penalty |
| `batch_size` | `2048` | `--batch-size` | Batch size |
| `ubatch_size` | `512` | `--ubatch-size` | Micro-batch size |
| `no_mmap` | `true` | `--no-mmap` | Disable memory mapping |
| `flash_attn` | `on` | `--flash-attn` | Flash Attention setting |
| `cache_ram` | `0` | `--cache-ram` | RAM cache size (MB) |
| `load_mode` | `""` | `--load-mode` | Model loading mode (lazy/mmap/mlock) |
| `parallel` | `1` | `--parallel` | Number of parallel sequences |
| `fit` | `on` | `--fit` | Fit model to available memory (`on`/`off`) |
| `kv_offload` | checked | `--kv-offload` | Offload KV cache to GPU (pure flag, no value) |
| `jinja` | unchecked | `--jinja` | Use Jinja templates for chat formatting (pure flag) |
| `extra_args` | `""` | - | Additional model-specific arguments |

## Update

Press `c` in the Configure tab to run `llama-server-update.sh` script, which downloads the latest llama.cpp binary.

Manual execution:
```bash
./llama-server-update.sh
```

## Project Structure

```
llama-server-loader/
├── Cargo.toml
├── llama-server-update.sh      # Update script
├── src/
│   ├── main.rs                 # TUI event loop, keyboard/mouse dispatch
│   ├── app.rs                  # App state machine (Idle/Running)
│   ├── model.rs                # Data types, .gguf scanner, GPU metrics
│   ├── config.rs               # JSON config load/save/sync
│   ├── server_manager.rs       # Process spawn/kill, mpsc events
│   ├── ui_log.rs               # Log display panel
│   ├── ui_mid.rs               # GPU monitoring panel (braille graphs)
│   ├── ui_server_tab.rs        # Server tab (model list + buttons)
│   ├── ui_config_tab.rs        # Configure tab (settings editor)
│   ├── ui_update_popup.rs      # Update progress popup
│   └── ui_llama_args_popup.rs  # Llama Args preview popup
└── README.md
```

## License

MIT

### Third-Party Licenses

#### nvtop (GPU Monitoring Inspiration)

The GPU monitoring braille graphs in this project were inspired by [nvtop](https://github.com/Syllo/nvtop)'s visualization approach.

nvtop is a GPU & Accelerator process monitoring tool that supports AMD, Apple, Huawei, Intel, NVIDIA, and Qualcomm GPUs.

- **Original Repository**: https://github.com/Syllo/nvtop
- **License**: GNU General Public License v3.0 or later (GPL-3.0-or-later)

In accordance with nvtop's license terms, the braille graph visualization used in this project complies with GPLv3 conditions. Under GPLv3 requirements:

1. **Source Code Disclosure**: The entire source code of this project is released under the MIT license
2. **Change Notification**: This README explicitly states that this project references nvtop's visualization approach
3. **License Maintenance**: To comply with GPLv3's copyleft requirements, while this project's license is MIT, the GPU monitoring code inspired by nvtop follows GPLv3

For details, see the GNU GPLv3 license text: https://www.gnu.org/licenses/gpl-3.0.html

---

**Note**: This project does not directly copy nvtop's source code. The visualization concept (braille character-based graphs) was referenced and implemented independently.
