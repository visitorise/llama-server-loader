# llama-server-loader

llama-server 실행을 관리하는 TUI(터미널 UI) 런처입니다. 기존 셸 스크립트들을 하나의 설정-driven 바이너리로 대체합니다.

![Version](https://img.shields.io/badge/version-0.2.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-Linux-lightgrey)

## Features

### 🖥️ TUI 인터페이스
- **ratatui** 기반 고성능 터미널 UI
- **마우스 지원**: 클릭, 드래그(텍스트 선택 → 클립보드 복사), 스크롤
- **실시간 로그**: 서버 stdout/stderr 색상 구분 표시
- **GPU 모니터링**: braille 그래프로 GPU Utilization/Memory 실시간 표시

### ⚙️ 설정 관리
- JSON 설정 파일 (`~/.config/llama-server-loader/config.json`)
- **공통 설정**: llama-server 경로, 호스트, 포트, GPU 오프로드, Flash Attention, speculative decoding 등
- **모델별 설정**: GPU 레이어, 컨텍스트 크기, KV 캐시 양자화, 샘플링 파라미터, 추가 인자
- TUI에서 직접 설정 편집 (Configure 탭)

### 🚀 서버 관리
- **원클릭 실행**: 모델 선택 후 Enter/r 키로 서버 시작
- **Graceful Shutdown**: SIGTERM → 20초 대기 → SIGKILL
- **로그 스트리밍**: 실시간 로그 표시, 자동 스크롤, 수동 스크롤 지원
- **LLama Args 팝업**: 실행 인자 미리보기

### 📊 GPU 모니터링 (nvtop 영감)
- NVIDIA GPU Utilization/Memory braille 그래프
- 실시간 GPU 메트릭 표시 (온도, 전력, 메모리 사용량)
- **nvtop 스타일** 시각화 — braille 문자 기반 그래프

### 🔄 자동 업데이트
- GitHub Release에서 llama.cpp 최신 바이너리 다운로드
- GPU 백엔드 자동 감지 (Vulkan/ROCm/CUDA)
- 자동 백업 및 교체

## Requirements

- Rust 2021 edition (빌드 시)
- `llama-server` 바이너리 (llama.cpp 프로젝트)
- Linux (Unix signal/process API 사용)
- NVIDIA GPU 사용 시: NVML 라이브러리 (nvidia-driver에 포함)

## Build

```bash
cd llama-server-loader
cargo build --release
```

빌드된 바이너리: `target/release/llama-server-loader`

## Usage

```bash
./llama-server-loader
```

### 화면 구성

- **상단 탭 바**: Server / Configure 탭 전환, 버전 표시
- **Server 탭**: 모델 목록 선택 + 서버 제어 버튼 (Run, Stop, Llama Args, Exit)
- **Configure 탭**: 공통 설정 및 모델별 설정 편집
- **GPU 모니터링 (중간)**: braille 그래프로 GPU Utilization/Memory 실시간 표시 (nvtop 스타일)
- **로그 패널 (하단)**: 서버 stdout/stderr 실시간 출력

### 키보드 단축키

**Server 탭:**

| 키 | 동작 |
|---|------|
| `↑` / `k` | 모델 선택 위로 |
| `↓` / `j` | 모델 선택 아래로 |
| `Enter` / `r` | 선택한 모델로 서버 시작 |
| `s` | 실행 중인 서버 중지 |
| `l` | Llama Args 팝업 표시 |
| `Tab` | Configure 탭으로 전환 |
| `q` / `Esc` | 종료 |

**Configure 탭:**

| 키 | 동작 |
|---|------|
| `↑` / `k` | 설정 항목 위로 |
| `↓` / `j` | 설정 항목 아래로 |
| `Enter` / `e` | 설정 편집 모드 토글 |
| `c` | 업데이트 확인 (GitHub Release) |
| `Tab` | Server 탭으로 전환 |

### 마우스 지원

| 동작 | 기능 |
|------|------|
| **클릭** | 버튼 실행, 탭 전환, 설정 항목 선택, 모델 선택 |
| **드래그** | 텍스트 선택 → 자동 클립보드 복사 (wl-copy) |
| **스크롤** | 로그/설정 영역 스크롤 |
| **팝업 중 클릭** | 차단 (클립보드 복사는 동작) |

## Configuration

설정 파일: `~/.config/llama-server-loader/config.json`

처음 실행 시 기본 설정 파일이 자동 생성됩니다.

> **⚠️ 초기 설정 후 재시작 필요**: 첫 실행에서 `llama_server_path`와 `model_dir`을 설정한 후, 앱을 재시작해야 모델 목록이 로드되어 세부 설정을 할 수 있습니다.

### Common Settings (공통 설정)

| 항목 | 기본값 | 설명 |
|------|--------|------|
| `llama_server_path` | `llama-server` | llama-server 실행 파일의 전체 경로 (명령어 포함). 예: `/home/user/AIAgent/llama.cpp/llama_cpp/llama-server` |
| `host` | `0.0.0.0` | 서버 바인딩 IP |
| `port` | `11400` | 서버 포트 |
| `model_dir` | `""` (자동 감지) | 모델 파일 디렉토리 |
| `no_mmap` | `true` | `--no-mmap` 플래그 사용 |
| `flash_attn` | `on` | Flash Attention 활성화 |
| `spec_type` | `none` | Speculative decoding 타입 |
| `spec_draft_n_max` | `2` | Speculative drafting 최대 수 |
| `extra_args` | `""` | llama-server 추가 인자 |
| `mid_pane_height` | `19` | 중간 패널(GPU 그래프) 높이 |

### Model Settings (모델별 설정)

| 항목 | 기본값 | 설명 |
|------|--------|------|
| `name` | 파일명 | 모델 표시 이름 |
| `file` | - | `.gguf` 파일명 |
| `gpu_layers` | `75` | GPU 오프로드 레이어 수 |
| `ctx_size` | `262144` | 컨텍스트 크기 (token) |
| `kv_k` | `q8_0` | KV Cache Key 양자화 |
| `kv_v` | `q8_0` | KV Cache Value 양자화 |
| `cpu_moe` | `0` | CPU MoE 레이어 수 |
| `temperature` | `1.0` | 샘플링 온도 |
| `top_k` | `40` | Top-K 샘플링 |
| `top_p` | `0.95` | Top-P (nucleus) 샘플링 |
| `min_p` | `0.0` | Min-P 샘플링 |
| `repeat_penalty` | `1.1` | 반복 패널티 |
| `presence_penalty` | `0.0` | Presence 패널티 |
| `extra_args` | `""` | 모델별 추가 인자 |

## Update

`Configure` 탭에서 `c` 키를 누르면 `llama-server-update.sh` 스크립트가 실행되어 최신 버전의 llama.cpp 바이너리를 다운로드합니다.

수동 실행:
```bash
./llama-server-update.sh
```

## Project Structure

```
llama-server-loader/
├── Cargo.toml
├── llama-server-update.sh      # 업데이트 스크립트
├── src/
│   ├── main.rs                 # TUI 이벤트 루프, 키보드/마우스 디스패치
│   ├── app.rs                  # App 상태 머신 (Idle/Running)
│   ├── model.rs                # 데이터 타입, .gguf 스캐너, GPU 메트릭
│   ├── config.rs               # JSON 설정 load/save/sync
│   ├── server_manager.rs       # 프로세스 spawn/kill, mpsc 이벤트
│   ├── ui_log.rs               # 로그 표시 패널
│   ├── ui_mid.rs               # GPU 모니터링 패널 (braille 그래프)
│   ├── ui_server_tab.rs        # Server 탭 (모델 리스트 + 버튼)
│   ├── ui_config_tab.rs        # Configure 탭 (설정 편집)
│   ├── ui_update_popup.rs      # 업데이트 진행 팝업
│   └── ui_llama_args_popup.rs  # Llama Args 미리보기 팝업
└── README.md
```

## License

MIT

### Third-Party Licenses

#### nvtop (GPU 모니터링 영감)

本 프로젝트의 GPU 모니터링 braille 그래프는 [nvtop](https://github.com/Syllo/nvtop)의 시각화 방식에서 영감을 받았습니다.

nvtop은 GPU & Accelerator 프로세스 모니터링 도구로, AMD, Apple, Huawei, Intel, NVIDIA, Qualcomm GPU를 지원합니다.

- **원본 저장소**: https://github.com/Syllo/nvtop
- **라이선스**: GNU General Public License v3.0 or later (GPL-3.0-or-later)

nvtop의 라이선스 조항에 따라,本 프로젝트에서 사용한 braille 그래프 시각화 방식은 GPLv3 조건을 준수합니다. GPLv3의 requirements에 따라:

1. **소스 코드 공개**: 본 프로젝트의 전체 소스 코드는 MIT 라이선스로 공개되어 있습니다
2. **변경 통보**: 본 프로젝트에서 nvtop의 시각화 방식을 참고했음을 이 README에 명시합니다
3. **라이선스 유지**: GPLv3의 copyleft 요구사항을 준수하기 위해, 본 프로젝트의 라이선스는 MIT이지만, nvtop에서 영감을 받은 GPU 모니터링 관련 코드는 GPLv3를 따릅니다

자세한 내용은 GNU GPLv3 라이선스 전문을 참조하세요: https://www.gnu.org/licenses/gpl-3.0.html

---

**참고**:本 프로젝트는 nvtop의 소스 코드를 직접 복사하지 않았으며, 시각화 컨셉(braille 문자 기반 그래프)만을 참고하여 독립적으로 구현하였습니다.
