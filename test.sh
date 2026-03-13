#!/usr/bin/env bash
#
# 在 Docker 容器內執行單元測試

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly IMAGE_NAME="minesweeper-test"

cd "${SCRIPT_DIR}"

echo "正在建立測試用 Docker 映像..."
docker build -t "${IMAGE_NAME}" -f docker/Dockerfile .

echo "正在執行單元測試..."
docker run --rm \
  -v "${SCRIPT_DIR}/logs:/app/logs" \
  "${IMAGE_NAME}" \
  npm test
