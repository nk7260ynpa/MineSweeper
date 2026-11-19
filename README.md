# MineSweeper

Windows 7 風格 MineSweeper 桌面應用程式，以 Rust 撰寫：Tauri 2 負責視窗與選單，
Leptos 前端編譯為 WebAssembly，遊戲邏輯集中在共用的 `minesweeper-core` crate。

## 功能特色

- Windows 7 經典視覺風格（3D 格子邊框、LED 七段顯示器、笑臉按鈕）
- 三種難度：初級 (9×9/10 雷)、中級 (16×16/40 雷)、高級 (30×16/99 雷，預設)
- 左鍵揭露格子、右鍵插旗／問號標記（可於選單關閉問號）
- Chord Click（左右鍵同時按、中鍵，或左鍵點已揭露的數字格）快速揭露
- 首次點擊保證 3×3 安全區域
- 計時器與地雷計數器
- 可打包為 macOS .app／.dmg 安裝檔

## 操作說明

| 操作 | 功能 |
|------|------|
| 左鍵點擊 | 揭露格子 |
| 右鍵點擊 | 插旗 → 問號 → 取消 |
| 左右鍵同時按／中鍵 | Chord Click（快速揭露已標記數字格周圍） |
| F2／點擊笑臉 | 新遊戲 |

## 安裝與執行

### 需求

```bash
# Rust 工具鏈與 WASM target
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# 前端建置工具與 Tauri CLI
cargo install --locked trunk tauri-cli
```

### 開發模式

```bash
./run.sh          # 等同 cargo tauri dev，前端修改會熱重載
```

開發模式的 log 寫入 `logs/minesweeper.log`；打包版寫入
`~/Library/Logs/com.minesweeper.app/`。

### 打包 .dmg

```bash
./run.sh build    # 等同 cargo tauri build
```

產物位於 `target/release/bundle/macos/`（.app）與 `target/release/bundle/dmg/`。

### 執行測試

```bash
./test.sh                          # 在 Docker 容器內執行（建議）
cargo test -p minesweeper-core     # 本機直接執行
```

## 專案架構

```text
MineSweeper/
├── Cargo.toml              # Cargo workspace（core / frontend / src-tauri）
├── run.sh                  # 啟動（dev）或打包（build）
├── test.sh                 # 在 Docker 內執行單元測試
├── docker/                 # 單元測試用映像（GUI 無法在容器內執行）
│   ├── build.sh
│   ├── Dockerfile
│   └── docker-compose.yaml
├── logs/                   # 開發模式 log
├── core/                   # minesweeper-core：純邏輯，無平台相依
│   └── src/
│       ├── config.rs       # 難度預設值
│       ├── game.rs         # 地雷配置、BFS 展開、Chord、勝負判定（含單元測試）
│       ├── led.rs          # LED 三位數格式化
│       └── rng.rs          # SplitMix64 亂數（原生／WASM 行為一致）
├── frontend/               # Leptos CSR 前端，由 trunk 編譯為 WASM
│   ├── index.html          # trunk 入口
│   ├── Trunk.toml          # dev server 埠號 1420
│   ├── styles/             # Win7 風格 CSS（面板、LED、笑臉）
│   └── src/
│       ├── app.rs          # 根元件：狀態列、開新局、訂閱事件、視窗量測
│       ├── board.rs        # 面板呈現與滑鼠互動
│       ├── led.rs          # LED 計數器元件
│       ├── timer.rs        # 計時器
│       └── tauri.rs        # invoke／listen 的 wasm-bindgen 綁定
└── src-tauri/              # Tauri 2 後端
    ├── tauri.conf.json     # 視窗、CSP、打包設定
    ├── capabilities/       # 前端可用權限
    ├── icons/              # App 圖示（由 icon.svg 產生）
    └── src/
        ├── lib.rs          # 設定狀態、指令（含 fit_window）、log
        └── menu.rs         # 遊戲／說明選單
```

### 資料流

1. 前端啟動時以 `invoke("get_config")`、`invoke("get_mark_enabled")` 取得設定。
2. 滑鼠操作直接呼叫 WASM 內的 `minesweeper_core::Game`，不經過 IPC。
3. 選單操作由後端處理，透過 `set-config`、`new-game`、`toggle-mark` 事件通知前端。
4. 前端以 `ResizeObserver` 量測遊戲畫面，與可視區域的差值交給 `fit_window`
   調整視窗，不寫死標題列或邊框尺寸。

## 技術棧

| 項目 | 選擇 |
|------|------|
| 桌面框架 | Tauri 2 |
| 前端 | Leptos 0.8（CSR）+ trunk |
| 遊戲邏輯 | Rust（`minesweeper-core`） |
| 測試 | `cargo test`（Docker：`rust:1-slim`） |
