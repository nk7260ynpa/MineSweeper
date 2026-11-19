#!/usr/bin/env bash
#
# 在 Docker 容器內執行核心邏輯單元測試。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR

cd "${SCRIPT_DIR}"

echo "正在建立測試用 Docker 映像..."
docker/build.sh

echo "正在執行單元測試..."
docker run --rm \
  -v "${SCRIPT_DIR}/logs:/app/logs" \
  minesweeper-test \
  cargo test --locked -p minesweeper-core
