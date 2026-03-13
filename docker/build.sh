#!/usr/bin/env bash
#
# 建立 Docker image（CI 測試用）
# Electron GUI 應用不適合在容器中執行，Docker 僅用於執行單元測試

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_DIR="$(dirname "${SCRIPT_DIR}")"

cd "${PROJECT_DIR}"

docker build -t minesweeper-test -f docker/Dockerfile .
