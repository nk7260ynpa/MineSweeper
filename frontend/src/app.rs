//! 應用程式根元件：狀態列（地雷計數、笑臉、計時器）與遊戲面板。

use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use minesweeper_core::{Config, Difficulty, Game};
use serde::Serialize;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;

use crate::board::Board;
use crate::led::LedCounter;
use crate::tauri;
use crate::timer::Timer;

/// `fit_window` 指令參數：遊戲畫面與可視區域的尺寸差（邏輯像素）。
#[derive(Serialize)]
struct FitArgs {
    dw: f64,
    dh: f64,
}

/// 量測遊戲畫面與可視區域的差距，請後端把視窗調整到剛好容納遊戲畫面。
fn fit_window(container: &web_sys::HtmlDivElement) {
    let rect = container.get_bounding_client_rect();
    let win = window();
    let viewport = |v: Result<wasm_bindgen::JsValue, _>| v.ok().and_then(|v| v.as_f64());
    let (Some(width), Some(height)) = (viewport(win.inner_width()), viewport(win.inner_height()))
    else {
        return;
    };
    let args = FitArgs {
        dw: rect.width().ceil() - width,
        dh: rect.height().ceil() - height,
    };
    spawn_local(async move {
        if let Err(err) = tauri::invoke_with::<_, ()>("fit_window", &args).await {
            leptos::logging::warn!("無法調整視窗：{err}");
        }
    });
}

/// 以 `ResizeObserver` 監看遊戲畫面，尺寸一變就呼叫 [`fit_window`]。
///
/// 觀察器在實際排版完成後才觸發，不受響應式更新順序影響。
fn observe_resize(container: web_sys::HtmlDivElement) {
    let target = container.clone();
    let callback = Closure::<dyn FnMut(js_sys::Array)>::new(move |_entries| fit_window(&target));
    match web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()) {
        Ok(observer) => observer.observe(&container),
        Err(err) => leptos::logging::warn!("無法建立 ResizeObserver：{err:?}"),
    }
    // 觀察器在整個 App 生命週期內有效。
    callback.forget();
}

/// 產生地雷配置用的亂數種子。
fn random_seed() -> u64 {
    const RANGE: f64 = 4_294_967_296.0; // 2^32
    let high = (js_sys::Math::random() * RANGE) as u64;
    let low = (js_sys::Math::random() * RANGE) as u64;
    (high << 32) | low
}

/// 根元件。
#[component]
pub fn App() -> impl IntoView {
    let config = StoredValue::new(Difficulty::default().config());
    let mark_enabled = StoredValue::new(true);
    let game = RwSignal::new(Game::new(config.get_value(), random_seed()));
    let timer = Timer::new();
    let surprised = RwSignal::new(false);

    let new_game = move || {
        let mut next = Game::new(config.get_value(), random_seed());
        next.set_mark_enabled(mark_enabled.get_value());
        timer.reset();
        surprised.set(false);
        game.set(next);
    };

    // 向後端取得初始設定；不在 Tauri 環境時沿用預設值。
    spawn_local(async move {
        if let Ok(initial) = tauri::invoke::<Config>("get_config").await {
            config.set_value(initial);
        }
        if let Ok(enabled) = tauri::invoke::<bool>("get_mark_enabled").await {
            mark_enabled.set_value(enabled);
        }
        new_game();
    });

    tauri::listen("new-game", move |()| new_game());
    tauri::listen("set-config", move |next: Config| {
        config.set_value(next);
        new_game();
    });
    tauri::listen("toggle-mark", move |enabled: bool| {
        mark_enabled.set_value(enabled);
        game.update_untracked(|g| g.set_mark_enabled(enabled));
    });

    // 遊戲畫面尺寸改變（含啟動、切換難度）時調整視窗。
    let container = NodeRef::<html::Div>::new();
    container.on_load(observe_resize);

    let mines_left =
        Signal::derive(move || game.with(|g| if g.is_lost() { 0 } else { g.remaining_mines() }));
    let smiley_class = move || {
        let face = game.with(|g| {
            if g.is_lost() {
                "dead"
            } else if g.is_won() {
                "win"
            } else if surprised.get() {
                "surprised"
            } else {
                "normal"
            }
        });
        format!("smiley-btn smiley-{face}")
    };

    view! {
        <div class="game-container" node_ref=container>
            <div class="status-bar">
                <LedCounter value=mines_left />
                <button class=smiley_class aria-label="新遊戲" on:click=move |_| new_game()></button>
                <LedCounter value=timer.seconds() />
            </div>
            <Board game timer surprised />
        </div>
    }
}
