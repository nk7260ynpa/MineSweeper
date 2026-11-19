#!/usr/bin/env bash
#
# 建立單元測試用 Docker image。
# Tauri GUI 應用需在 macOS 本機編譯執行，Docker 僅用於執行核心邏輯單元測試。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
PROJECT_DIR="$(dirname "${SCRIPT_DIR}")"
readonly PROJECT_DIR

cd "${PROJECT_DIR}"

docker build -t minesweeper-test -f docker/Dockerfile .
