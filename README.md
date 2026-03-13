# 踩地雷 MineSweeper

Windows 7 風格踩地雷桌面應用程式，使用 Electron + 7.css 建立，可在 macOS 上安裝執行。

## 功能特色

- Windows 7 經典視覺風格（3D 格子邊框、LED 七段顯示器、笑臉按鈕）
- 三種難度：初級 (9x9/10雷)、中級 (16x16/40雷)、高級 (30x16/99雷)
- 左鍵揭露格子、右鍵插旗/問號標記
- Chord Click（左右鍵同時按）快速揭露
- 首次點擊保證 3x3 安全區域
- 計時器與地雷計數器
- 可打包為 macOS .dmg 安裝檔

## 操作說明

| 操作 | 功能 |
|------|------|
| 左鍵點擊 | 揭露格子 |
| 右鍵點擊 | 插旗 → 問號 → 取消 |
| 左右鍵同時按 | Chord Click（快速揭露已標記數字格周圍） |
| F2 | 新遊戲 |
| 點擊笑臉 | 新遊戲 |

## 安裝與執行

### 開發模式

```bash
# 安裝依賴
npm install

# 啟動應用
npm start

# 或使用啟動腳本
./run.sh
```

### 打包 .dmg 安裝檔

```bash
npm run dist
```

產生的 .dmg 檔案位於 `dist/` 目錄。

### 執行測試

```bash
npm test
```

## 專案架構

```
MineSweeper/
├── package.json                    # 專案設定與依賴
├── .gitignore
├── README.md
├── run.sh                          # 啟動腳本
├── docker/
│   ├── build.sh                    # Docker 映像建立腳本（CI 測試用）
│   ├── Dockerfile                  # Docker 映像定義（CI 測試用）
│   └── docker-compose.yaml         # Docker Compose 設定（CI 測試用）
├── logs/
│   └── .gitkeep
├── src/
│   ├── main/
│   │   ├── main.js                 # Electron 主程序（視窗管理、IPC）
│   │   ├── menu.js                 # 選單列（遊戲/說明）
│   │   └── preload.js              # contextBridge API
│   └── renderer/
│       ├── index.html              # 主頁面
│       ├── styles/
│       │   ├── main.css            # 全域樣式（匯入 7.css）
│       │   ├── board.css           # 格子 3D 效果、數字顏色
│       │   ├── led.css             # LED 七段顯示器
│       │   └── smiley.css          # 笑臉按鈕
│       └── js/
│           ├── game.js             # 核心邏輯（地雷配置、BFS、勝負判定）
│           ├── board.js            # 面板 DOM 操作與事件處理
│           ├── timer.js            # 計時器
│           ├── counter.js          # LED 計數器
│           └── app.js              # 入口，串接所有模組
└── test/
    └── game.test.js                # 核心邏輯單元測試（26 項）
```

## 技術棧

| 項目 | 選擇 |
|------|------|
| 桌面框架 | Electron |
| Win7 樣式 | 7.css + 自訂 CSS |
| 打包工具 | electron-builder |
| 測試 | Node.js 內建 node:test |
