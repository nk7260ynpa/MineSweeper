#!/usr/bin/env bash
#
# 啟動 MineSweeper 應用程式
# 自動安裝依賴並啟動 Electron 應用

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "${SCRIPT_DIR}"

# 檢查 Node.js 是否安裝
if ! command -v node &> /dev/null; then
  echo "錯誤：請先安裝 Node.js（建議 v18 以上）"
  exit 1
fi

# 安裝依賴
if [[ ! -d "node_modules" ]]; then
  echo "正在安裝依賴..."
  npm install
fi

# 啟動應用
echo "正在啟動 MineSweeper..."
npm start
