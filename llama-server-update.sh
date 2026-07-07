#!/bin/bash
set -euo pipefail

GITHUB_REPO="ggml-org/llama.cpp"
LLAMA_SRV_PATH="$HOME/AIAgent/llama.cpp/llama_cpp"
DOWNLOAD_TMP="/tmp/llama-cpp-download"

server_bin=$(find "$LLAMA_SRV_PATH" -maxdepth 1 -name "llama-server" -type f 2>/dev/null | head -1)
if [[ -z "$server_bin" ]]; then
    echo "llama-server not found in $LLAMA_SRV_PATH"
    exit 1
fi

raw_ver=$("$server_bin" --version 2>&1) || { echo "Could not get version"; exit 1; }
local_ver=$(echo "$raw_ver" | grep -oP 'version:\s*\K[0-9]+' || echo "0")
echo "Local version: $local_ver"

tag=$(curl -sL --max-time 15 "https://api.github.com/repos/$GITHUB_REPO/releases/latest" \
    | python3 -c "import sys,json; print(json.load(sys.stdin).get('tag_name',''))" 2>/dev/null) || { echo "Could not fetch latest version"; exit 1; }
latest_ver=$(echo "$tag" | grep -oP '[0-9]+') || { echo "Could not parse version from $tag"; exit 1; }
echo "Latest version: $latest_ver ($tag)"

if [[ "$latest_ver" -le "$local_ver" ]]; then
    echo "Already up to date."
    exit 0
fi

echo "Update available: $local_ver -> $latest_ver"

gpu_backend="cpu"
if [[ -f "$LLAMA_SRV_PATH/libggml-vulkan.so" ]]; then
    gpu_backend="vulkan"
elif [[ -f "$LLAMA_SRV_PATH/libggml-rocm.so" ]]; then
    gpu_backend="rocm"
elif ldd "$server_bin" 2>/dev/null | grep -qi "cuda"; then
    gpu_backend="cuda"
fi
echo "GPU backend: $gpu_backend"

case "$gpu_backend" in
    vulkan) asset_name="llama-b${latest_ver}-bin-ubuntu-vulkan-x64.tar.gz" ;;
    rocm)   asset_name="llama-b${latest_ver}-bin-ubuntu-rocm-7.2-x64.tar.gz" ;;
    *)      asset_name="llama-b${latest_ver}-bin-ubuntu-x64.tar.gz" ;;
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

backup_dir="$HOME/AIAgent/llama.cpp/backup/llama-b${local_ver}-backup-$(date +%Y%m%d%H%M%S)"
cp -a "$LLAMA_SRV_PATH" "$backup_dir"
echo "Backup saved: $backup_dir"

extract_dir="$DOWNLOAD_TMP/extracted"
rm -rf "$extract_dir"
mkdir -p "$extract_dir"
tar xzf "$DOWNLOAD_TMP/$asset_name" -C "$extract_dir"

rm -rf "$LLAMA_SRV_PATH"
cp -a "$extract_dir"/* "$LLAMA_SRV_PATH/"
rm -rf "$DOWNLOAD_TMP"

echo "Update complete!"
