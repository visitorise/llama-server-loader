# llama-server Loader

llama-server 실행을 관리하는 TUI(터미널 UI) 런처입니다.
기존 6개의 중복된 셸 스크립트를 하나의 설정-driven 바이너리로 대체합니다.

## Features

- **TUI 인터페이스** — ratatui 기반, 키보드로 모델 선택 및 서버 시작/중지
- **설정 파일 기반** — `~/.config/llama-server-loader/config.json` 에 모든 설정 저장
- **모델 자동 스캔** — 서버 디렉토리에서 `.gguf` 파일 자동 탐색
- **로그 실시간 출력** — 서버 stdout/stderr 실시간 표시, 색상 구분
- **자동 업데이트** — GitHub Release에서 llama.cpp 바이너리 다운로드 및 적용
- **Graceful shutdown** — SIGTERM → 20초 대기 → SIGKILL

## Requirements

- Rust 2021 edition (빌드 시)
- `llama-server` 바이너리 (llama.cpp 프로젝트)
- Linux (Unix signal/process API 사용)

## Build

```bash
cd llama-server-loader
cargo build --release
```

빌드된 바이너리는 `target/release/llama-server-loader` 에 위치합니다.

## Configuration

설정 파일: `~/.config/llama-server-loader/config.json`

처음 실행 시 기본 설정 파일이 자동 생성됩니다.
`Configure` 탭에서 설정을 편집할 수 있습니다.

### Common Settings (공통 설정)

| 항목 | 기본값 | 설명 |
|------|--------|------|
| `llama_server_path` | `~/AIAgent/llama.cpp/llama_cpp` | llama-server 실행 파일이 위치한 디렉토리 |
| `host` | `127.0.0.1` | 서버 바인딩 IP |
| `port` | 8888 | 서버 포트 |
| `n_gpu_layers` | 50 | GPU 오프로드 레이어 수 (-1 = 전체) |
| `ctx_size` | 4096 | 컨텍스트 크기 (token) |
| `mid_pane_height` | 3 | 중간 패널 높이 (nvtop 등 예약) |
| `extra_args` | `""` | llama-server에 추가로 전달할 인자 |

### Model Settings (모델별 설정)

| 항목 | 설명 |
|------|------|
| `name` | 모델 표시 이름 |
| `model_path` | `.gguf` 파일 경로 |
| `prompt_cache` | 프롬프트 캐시 파일 경로 (선택) |
| `extra_args` | 모델별 추가 인자 (선택) |

## Usage

```bash
./llama-server-loader
```

### Key Bindings

**Server 탭:**

| 키 | 동작 |
|---|------|
| `↑` / `k` | 모델 선택 위로 |
| `↓` / `j` | 모델 선택 아래로 |
| `Enter` / `r` / `R` | 선택한 모델로 서버 시작 |
| `s` / `S` | 실행 중인 서버 중지 |
| `Tab` | Configure 탭으로 전환 |
| `q` / `Q` / `Esc` | 종료 |

**Configure 탭:**

| 키 | 동작 |
|---|------|
| `↑` / `k` | 모델 목록 위로 |
| `↓` / `j` | 모델 목록 아래로 |
| `c` / `C` | 업데이트 확인 (GitHub Release) |
| `Tab` | Server 탭으로 전환 |

## Update

`Configure` 탭에서 `c` 키를 누르면 `llama-server-update.sh` 스크립트가 실행되어 최신 버전의 llama.cpp 바이너리를 다운로드합니다.

수동 실행:
```bash
./llama-server-update.sh
```

업데이트 과정:
1. 현재 버전 확인
2. GitHub API로 최신 릴리스 태그 조회
3. GPU 백엔드 감지 (vulkan/rocm/cpu)
4. 적합한 아카이브 다운로드
5. 기존 파일 백업 (`~/AIAgent/llama.cpp/backup/`)
6. 새 바이너리로 교체

## Project Structure

```
llama-server-loader/
├── Cargo.toml
├── llama-server-update.sh   # 업데이트 스크립트
├── src/
│   ├── main.rs              # TUI event loop, keyboard dispatch
│   ├── app.rs               # App state machine (Idle/Running)
│   ├── model.rs             # 데이터 타입, .gguf 스캐너
│   ├── config.rs            # JSON 설정 load/save/sync
│   ├── server_manager.rs    # 프로세스 spawn/kill, mpsc 이벤트
│   ├── ui_log.rs            # 로그 표시 패널
│   ├── ui_mid.rs            # 중간 패널 (nvtop placeholder)
│   ├── ui_server_tab.rs     # Server 탭 (모델 리스트 + 버튼)
│   ├── ui_config_tab.rs     # Configure 탭 (설정 편집)
│   └── ui_update_popup.rs   # 업데이트 진행 팝업
└── README.md
```

## License

MIT
