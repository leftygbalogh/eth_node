#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_PATH="$ROOT_DIR/target/release/rust_terminal_snake_game"

is_wsl() {
  grep -qiE "(microsoft|wsl)" /proc/version 2>/dev/null
}

is_msys() {
  [[ "${OSTYPE:-}" == msys* || "${OSTYPE:-}" == cygwin* || -n "${MSYSTEM:-}" ]]
}

if [[ ! -x "$BIN_PATH" ]]; then
  echo "Building release binary..."
  cargo build --release
fi

if is_msys; then
  if command -v winpty >/dev/null 2>&1; then
    exec winpty "$BIN_PATH"
  fi
  exec "$BIN_PATH"
fi

if is_wsl; then
  exec "$BIN_PATH"
fi

exec "$BIN_PATH"
