#!/bin/bash
set -euo pipefail

# Usage: ./llama-server-update.sh <installation_path>
# Example: ./llama-server-update.sh ~/AIAgent/llama.cpp/llama_cpp

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <installation_path>"
    echo "Example: $0 ~/AIAgent/llama.cpp/llama_cpp"
    exit 1
fi

INSTALL_PATH="$1"
GITHUB_REPO="ggml-org/llama.cpp"
DOWNLOAD_TMP="/tmp/llama-cpp-download"

# The argument check above already ensures we have a valid path.
# Remove the duplicated check and any stray numbered prefixes.
# (These prefixes were a copy‑paste artifact and caused syntax errors.)
if [[ ! -d "$INSTALL_PATH" ]]; then
    echo "ERROR: Installation path does not exist: $INSTALL_PATH"
    exit 1
fi

server_bin=$(find "$INSTALL_PATH" -maxdepth 1 -name "llama-server" -type f 2>/dev/null | head -1)
if [[ -z "$server_bin" ]]; then
    echo "llama-server not found in $INSTALL_PATH"
    exit 1
fi

# Get local version (build number from --version output)
raw_ver=$("$server_bin" --version 2>&1) || { echo "Could not get version"; exit 1; }
local_ver=$(echo "$raw_ver" | grep -oP 'build\s+\K[0-9]+' || echo "0")
echo "Local version (build): $local_ver"

# Get latest build tag (bXXXX format) that has pre-built Ubuntu vulkan binaries
tag=$(curl -sL --max-time 15 "https://api.github.com/repos/$GITHUB_REPO/releases" \
    | python3 -c "
import sys, json
data = json.load(sys.stdin)
for r in data:
    if r['tag_name'].startswith('b') and r['tag_name'][1:].isdigit():
        assets = [a['name'] for a in r.get('assets', [])]
        if any('ubuntu-vulkan-x64' in a for a in assets):
            print(r['tag_name'])
            break
" 2>/dev/null) || { echo "Could not fetch latest build version"; exit 1; }

latest_ver=$(echo "$tag" | grep -oP 'b\K[0-9]+') || { echo "Could not parse build number from $tag"; exit 1; }
echo "Latest version (build): $latest_ver ($tag)"

if [[ "$latest_ver" -le "$local_ver" ]]; then
    echo "Already up to date."
    exit 0
fi

echo "Update available: $local_ver -> $latest_ver"

gpu_backend="cpu"
if [[ -f "$INSTALL_PATH/libggml-vulkan.so" ]]; then
    gpu_backend="vulkan"
elif [[ -f "$INSTALL_PATH/libggml-rocm.so" ]]; then
    gpu_backend="rocm"
elif ldd "$server_bin" 2>/dev/null | grep -qi "cuda"; then
    gpu_backend="cuda"
fi
echo "GPU backend: $gpu_backend"

case "$gpu_backend" in
    vulkan) asset_name="llama-${tag}-bin-ubuntu-vulkan-x64.tar.gz" ;;
    rocm)   asset_name="llama-${tag}-bin-ubuntu-rocm-7.2-x64.tar.gz" ;;
    *)      asset_name="llama-${tag}-bin-ubuntu-x64.tar.gz" ;;
esac

download_url="https://github.com/$GITHUB_REPO/releases/download/$tag/$asset_name"
echo "Downloading: $asset_name"

mkdir -p "$DOWNLOAD_TMP"
if ! curl -sL --fail --max-time 600 -o "$DOWNLOAD_TMP/$asset_name" "$download_url"; then
    echo "ERROR: Asset not found at: $download_url"
    echo "The release may not have a pre-built binary for your platform."
    echo "Try: https://github.com/$GITHUB_REPO/releases/tag/$tag"
    rm -rf "$DOWNLOAD_TMP"
    exit 1
fi
echo "Downloaded: $(du -h "$DOWNLOAD_TMP/$asset_name" | cut -f1)"

if ! gzip -t "$DOWNLOAD_TMP/$asset_name" 2>/dev/null; then
    echo "ERROR: Downloaded file is corrupted or not a valid archive."
    rm -rf "$DOWNLOAD_TMP"
    exit 1
fi

backup_dir="$(dirname "$INSTALL_PATH")/backup/llama-b${local_ver}-backup-$(date +%Y%m%d%H%M%S)"
mkdir -p "$(dirname "$backup_dir")"
cp -a "$INSTALL_PATH" "$backup_dir"
echo "Backup saved: $backup_dir"

extract_dir="$DOWNLOAD_TMP/extracted"
rm -rf "$extract_dir"
mkdir -p "$extract_dir"
tar xzf "$DOWNLOAD_TMP/$asset_name" -C "$extract_dir"

rm -rf "$INSTALL_PATH"
cp -a "$extract_dir"/* "$INSTALL_PATH/"
rm -rf "$DOWNLOAD_TMP"

echo "Update complete!"
