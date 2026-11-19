//! MineSweeper Tauri 後端：視窗管理、選單與前端溝通。
//!
//! 遊戲邏輯在前端（Leptos/WASM）透過 `minesweeper-core` 執行；後端只負責
//! 保存「難度」與「問號標記」設定，並在選單操作時通知前端。

mod menu;

use std::sync::Mutex;

use log::{error, info};
use minesweeper_core::{Config, Difficulty};
use tauri::{AppHandle, Emitter, LogicalSize, Manager, State, WebviewWindow};

/// 前端事件：開新遊戲。
pub(crate) const EVENT_NEW_GAME: &str = "new-game";
/// 前端事件：變更面板設定（附 [`Config`]）。
pub(crate) const EVENT_SET_CONFIG: &str = "set-config";
/// 前端事件：切換問號標記（附 `bool`）。
pub(crate) const EVENT_TOGGLE_MARK: &str = "toggle-mark";

/// 使用者設定。
#[derive(Debug)]
pub(crate) struct Settings {
    /// 目前難度。
    pub difficulty: Difficulty,
    /// 是否啟用問號標記。
    pub mark_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::default(),
            mark_enabled: true,
        }
    }
}

/// 取得目前面板設定。
#[tauri::command]
fn get_config(settings: State<'_, Mutex<Settings>>) -> Config {
    lock(&settings).difficulty.config()
}

/// 取得是否啟用問號標記。
#[tauri::command]
fn get_mark_enabled(settings: State<'_, Mutex<Settings>>) -> bool {
    lock(&settings).mark_enabled
}

/// 取得設定鎖；鎖中毒時沿用內部資料（設定只是兩個純值，不會不一致）。
pub(crate) fn lock<'a>(settings: &'a Mutex<Settings>) -> std::sync::MutexGuard<'a, Settings> {
    settings
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// 依前端量測的差值調整視窗大小，讓遊戲畫面剛好填滿內容區。
///
/// 前端在面板尺寸改變後量測「遊戲畫面尺寸 − 可視區域尺寸」並傳入。由實際 DOM
/// 計算可同時涵蓋標題列、WebView 內距與字型度量等平台差異，不必寫死常數。
///
/// # Arguments
///
/// * `window` - 呼叫此指令的視窗。
/// * `dw` - 寬度差（邏輯像素），正值代表需要加寬。
/// * `dh` - 高度差（邏輯像素），正值代表需要加高。
#[tauri::command]
fn fit_window(window: WebviewWindow, dw: f64, dh: f64) -> Result<(), String> {
    if dw.abs() < 0.5 && dh.abs() < 0.5 {
        return Ok(());
    }
    let resize = || -> tauri::Result<()> {
        let scale = window.scale_factor()?;
        let current = window.inner_size()?.to_logical::<f64>(scale);
        let target = LogicalSize::new(current.width + dw, current.height + dh);
        info!("調整視窗：{current:?} → {target:?}");
        window.set_size(target)
    };
    resize().map_err(|err| {
        error!("調整視窗失敗：{err}");
        err.to_string()
    })
}

/// 切換難度：更新設定並通知前端重新開局（視窗大小由前端量測後調整）。
pub(crate) fn change_difficulty(app: &AppHandle, difficulty: Difficulty) -> tauri::Result<()> {
    let state = app.state::<Mutex<Settings>>();
    lock(&state).difficulty = difficulty;
    let config = difficulty.config();
    info!("切換難度：{} {:?}", difficulty.label(), config);
    app.emit(EVENT_SET_CONFIG, config)
}

/// 切換問號標記並通知前端，回傳新的狀態。
pub(crate) fn toggle_mark(app: &AppHandle) -> tauri::Result<bool> {
    let state = app.state::<Mutex<Settings>>();
    let enabled = {
        let mut settings = lock(&state);
        settings.mark_enabled = !settings.mark_enabled;
        settings.mark_enabled
    };
    info!("問號標記：{}", if enabled { "啟用" } else { "停用" });
    app.emit(EVENT_TOGGLE_MARK, enabled)?;
    Ok(enabled)
}

/// 建立 log 外掛：debug 版寫入專案 `logs/`，release 版寫入系統 log 目錄。
fn log_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    use tauri_plugin_log::{Target, TargetKind};

    let file_target = if cfg!(debug_assertions) {
        TargetKind::Folder {
            path: std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../logs"),
            file_name: Some("minesweeper".into()),
        }
    } else {
        TargetKind::LogDir {
            file_name: Some("minesweeper".into()),
        }
    };

    tauri_plugin_log::Builder::new()
        .targets([Target::new(TargetKind::Stdout), Target::new(file_target)])
        .level(log::LevelFilter::Info)
        .build()
}

/// 啟動應用程式。
///
/// # Panics
///
/// Tauri 無法建立應用程式（例如設定檔錯誤）時 panic。
pub fn run() {
    tauri::Builder::default()
        .plugin(log_plugin())
        .manage(Mutex::new(Settings::default()))
        .setup(|app| {
            let handle = app.handle();
            let settings = Settings::default();
            app.set_menu(menu::build(handle, &settings)?)?;
            info!("MineSweeper 啟動，難度：{}", settings.difficulty.label());
            Ok(())
        })
        .on_menu_event(menu::handle_event)
        .invoke_handler(tauri::generate_handler![
            get_config,
            get_mark_enabled,
            fit_window
        ])
        .run(tauri::generate_context!())
        .expect("無法啟動 MineSweeper");
}
