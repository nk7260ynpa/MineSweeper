#!/usr/bin/env bash
#
# 啟動 MineSweeper。
#
# 用法：
#   ./run.sh         開發模式（trunk 熱重載 + Tauri 視窗），log 寫入 logs/
#   ./run.sh build   打包 macOS .app 與 .dmg 至 target/release/bundle/

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR

#######################################
# 確認必要工具已安裝，缺少時提示安裝方式。
# Arguments:
#   無
#######################################
check_tools() {
  if [[ -f "${HOME}/.cargo/env" ]]; then
    # shellcheck source=/dev/null
    source "${HOME}/.cargo/env"
  fi

  if ! command -v cargo &> /dev/null; then
    echo "錯誤：請先安裝 Rust（https://rustup.rs）" >&2
    exit 1
  fi
  if ! rustup target list --installed | grep -q '^wasm32-unknown-unknown$'; then
    echo "正在安裝 wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
  fi
  if ! command -v trunk &> /dev/null; then
    echo "錯誤：請先安裝 trunk（cargo install --locked trunk）" >&2
    exit 1
  fi
  if ! cargo tauri --version &> /dev/null; then
    echo "錯誤：請先安裝 tauri-cli（cargo install --locked tauri-cli）" >&2
    exit 1
  fi
}

main() {
  cd "${SCRIPT_DIR}"
  check_tools
  mkdir -p logs

  case "${1:-dev}" in
    dev)
      echo "正在啟動 MineSweeper（開發模式）..."
      cargo tauri dev
      ;;
    build)
      echo "正在打包 MineSweeper..."
      cargo tauri build
      ;;
    *)
      echo "用法：$0 [dev|build]" >&2
      exit 1
      ;;
  esac
}

main "$@"
