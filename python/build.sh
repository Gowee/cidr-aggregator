#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PKG_DIR="$SCRIPT_DIR/cidr_aggregator"
BIN_DIR="$PKG_DIR/_bin"

# Parse args
TARGET=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --target) TARGET="$2"; shift 2 ;;
    *) echo "Unknown arg: $1"; exit 1 ;;
  esac
done

# Determine platform dir name
if [[ -z "$TARGET" ]]; then
  case "$(uname -s)-$(uname -m)" in
    Linux-x86_64)   PLATFORM="linux-x64" ;;
    Linux-aarch64)  PLATFORM="linux-arm64" ;;
    Darwin-x86_64)  PLATFORM="macos-x64" ;;
    Darwin-arm64)   PLATFORM="macos-arm64" ;;
    MINGW*-x86_64)  PLATFORM="windows-x64" ;;
    *) echo "Unknown platform: $(uname -s)-$(uname -m)"; exit 1 ;;
  esac
else
  case "$TARGET" in
    x86_64-unknown-linux-*|x86_64-unknown-linux-gnu)  PLATFORM="linux-x64" ;;
    aarch64-unknown-linux-*|aarch64-unknown-linux-gnu) PLATFORM="linux-arm64" ;;
    x86_64-apple-darwin)          PLATFORM="macos-x64" ;;
    aarch64-apple-darwin)         PLATFORM="macos-arm64" ;;
    x86_64-pc-windows-msvc)       PLATFORM="windows-x64" ;;
    *) echo "Unknown target: $TARGET"; exit 1 ;;
  esac
fi

CARGO_FLAGS=(--release --features cli)
[[ -n "$TARGET" ]] && CARGO_FLAGS+=(--target "$TARGET")

EXT=""
[[ "$PLATFORM" == windows-* ]] && EXT=".exe"

echo "==> Building Rust binary (${PLATFORM})..."
cargo build "${CARGO_FLAGS[@]}" -p cidr-aggregator

# Find the binary
if [[ -n "$TARGET" ]]; then
  BIN_PATH="$REPO_DIR/target/$TARGET/release/cidr-aggregator$EXT"
else
  BIN_PATH="$REPO_DIR/target/release/cidr-aggregator$EXT"
fi

echo "==> Copying binary to $BIN_DIR/$PLATFORM/"
mkdir -p "$BIN_DIR/$PLATFORM"
cp "$BIN_PATH" "$BIN_DIR/$PLATFORM/"

echo "==> Building Python wheel..."
cd "$SCRIPT_DIR"
python -m build --wheel

echo "==> Done: $(ls dist/*.whl)"
