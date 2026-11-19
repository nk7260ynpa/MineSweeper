//! 應用程式選單：遊戲（新遊戲、難度、問號標記、結束）與說明。

use log::{error, info};
use minesweeper_core::Difficulty;
use tauri::menu::{
    AboutMetadata, CheckMenuItem, CheckMenuItemBuilder, Menu, MenuBuilder, MenuEvent,
    MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder,
};
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::{EVENT_NEW_GAME, Settings};

/// 選單項目 ID：新遊戲。
const ID_NEW_GAME: &str = "new_game";
/// 選單項目 ID：問號標記。
const ID_MARK: &str = "mark";

/// 需要在執行期同步勾選狀態的選單項目。
struct MenuHandles {
    difficulties: Vec<(Difficulty, CheckMenuItem<Wry>)>,
    mark: CheckMenuItem<Wry>,
}

/// 「關於」對話框。
fn about_item(app: &AppHandle) -> tauri::Result<PredefinedMenuItem<Wry>> {
    let metadata = AboutMetadata {
        name: Some("MineSweeper".into()),
        version: Some(app.package_info().version.to_string()),
        comments: Some("Windows 7 風格 MineSweeper\n使用 Rust（Tauri + Leptos）建立".into()),
        ..Default::default()
    };
    PredefinedMenuItem::about(app, Some("關於 MineSweeper"), Some(metadata))
}

/// 建立選單，並把需要同步的項目交給 Tauri 管理。
pub(crate) fn build(app: &AppHandle, settings: &Settings) -> tauri::Result<Menu<Wry>> {
    let difficulties = Difficulty::ALL
        .into_iter()
        .map(|d| {
            CheckMenuItemBuilder::with_id(d.id(), d.label())
                .checked(d == settings.difficulty)
                .build(app)
                .map(|item| (d, item))
        })
        .collect::<tauri::Result<Vec<_>>>()?;
    let mark = CheckMenuItemBuilder::with_id(ID_MARK, "標記 (?)")
        .checked(settings.mark_enabled)
        .build(app)?;

    let mut game = SubmenuBuilder::new(app, "遊戲")
        .item(
            &MenuItemBuilder::with_id(ID_NEW_GAME, "新遊戲")
                .accelerator("F2")
                .build(app)?,
        )
        .separator();
    for (_, item) in &difficulties {
        game = game.item(item);
    }
    let game = game
        .separator()
        .item(&mark)
        .separator()
        .item(&PredefinedMenuItem::quit(app, Some("結束"))?)
        .build()?;
    let help = SubmenuBuilder::new(app, "說明")
        .item(&about_item(app)?)
        .build()?;

    let mut menu = MenuBuilder::new(app);
    // macOS 的第一個子選單會成為應用程式選單（顯示 App 名稱）。
    if cfg!(target_os = "macos") {
        let app_menu = SubmenuBuilder::new(app, "MineSweeper")
            .item(&about_item(app)?)
            .separator()
            .hide()
            .hide_others()
            .show_all()
            .separator()
            .quit()
            .build()?;
        menu = menu.item(&app_menu);
    }
    let menu = menu.item(&game).item(&help).build()?;

    app.manage(MenuHandles { difficulties, mark });
    Ok(menu)
}

/// 處理選單點擊。
pub(crate) fn handle_event(app: &AppHandle, event: MenuEvent) {
    let id = event.id().as_ref();
    let result = match id {
        ID_NEW_GAME => {
            info!("新遊戲");
            app.emit(EVENT_NEW_GAME, ())
        }
        ID_MARK => crate::toggle_mark(app).and_then(|enabled| sync_mark(app, enabled)),
        _ => match Difficulty::from_id(id) {
            Some(difficulty) => crate::change_difficulty(app, difficulty)
                .and_then(|()| sync_difficulty(app, difficulty)),
            None => Ok(()),
        },
    };
    if let Err(err) = result {
        error!("處理選單項目 {id} 失敗：{err}");
    }
}

/// 讓難度項目呈現單選效果（點擊時系統會自動切換勾選，需要改回正確狀態）。
fn sync_difficulty(app: &AppHandle, selected: Difficulty) -> tauri::Result<()> {
    let handles = app.state::<MenuHandles>();
    for (difficulty, item) in &handles.difficulties {
        item.set_checked(*difficulty == selected)?;
    }
    Ok(())
}

/// 同步問號標記的勾選狀態。
fn sync_mark(app: &AppHandle, enabled: bool) -> tauri::Result<()> {
    app.state::<MenuHandles>().mark.set_checked(enabled)
}
