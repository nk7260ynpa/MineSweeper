//! 遊戲面板：格子呈現與滑鼠互動（揭露、插旗、Chord Click）。

use leptos::ev::MouseEvent;
use leptos::prelude::*;
use minesweeper_core::{CellView, Game, Pos};

use crate::timer::Timer;

/// `MouseEvent.button`：左鍵。
const BUTTON_LEFT: i16 = 0;
/// `MouseEvent.button`：中鍵。
const BUTTON_MIDDLE: i16 = 1;
/// `MouseEvent.button`：右鍵。
const BUTTON_RIGHT: i16 = 2;
/// `MouseEvent.buttons`：只按住左鍵。
const BUTTONS_LEFT: u16 = 1;
/// `MouseEvent.buttons`：左右鍵同時按住。
const BUTTONS_BOTH: u16 = 3;

/// 格子的 CSS class。
fn cell_class(view: Option<CellView>, pressed: bool) -> String {
    let base = match view {
        None | Some(CellView::Hidden) => "cell hidden".to_owned(),
        Some(CellView::Flagged) => "cell flagged".to_owned(),
        Some(CellView::Question) => "cell question".to_owned(),
        Some(CellView::Revealed(0)) => "cell revealed".to_owned(),
        Some(CellView::Revealed(n)) => format!("cell revealed n{n}"),
        Some(CellView::Mine) => "cell mine".to_owned(),
        Some(CellView::MineExploded) => "cell mine-exploded".to_owned(),
        Some(CellView::WrongFlag) => "cell mine-wrong".to_owned(),
    };
    if pressed { base + " pressed" } else { base }
}

/// 取得格子呈現；面板尺寸切換的瞬間，舊格子可能超出新面板範圍。
fn view_at(game: &Game, (r, c): Pos) -> Option<CellView> {
    (r < game.rows() && c < game.cols()).then(|| game.view(r, c))
}

/// 左鍵放開：數字格觸發 Chord Click，其餘揭露。
fn reveal_or_chord(game: &mut Game, (r, c): Pos) {
    if matches!(game.view(r, c), CellView::Revealed(n) if n > 0) {
        game.chord(r, c);
    } else {
        game.reveal(r, c);
    }
}

/// 遊戲面板元件。
#[component]
pub fn Board(game: RwSignal<Game>, timer: Timer, surprised: RwSignal<bool>) -> impl IntoView {
    // 目前呈現凹陷效果的格子。
    let pressed = RwSignal::new(Vec::<Pos>::new());
    let mouse_down = StoredValue::new(false);
    // 本次按壓是否為 Chord（左右鍵、中鍵，或左鍵按住數字格）。
    let chord_active = StoredValue::new(false);
    let dims = Memo::new(move |_| game.with(|g| (g.rows(), g.cols())));
    let is_over = move || game.with_untracked(Game::is_over);

    let clear = move || {
        pressed.set(Vec::new());
        chord_active.set_value(false);
    };
    let show_chord = move |pos: Pos| {
        pressed.set(game.with_untracked(|g| g.chord_targets(pos.0, pos.1)));
        chord_active.set_value(true);
    };
    // 左鍵按住：數字格顯示 Chord 預覽，未揭露／問號格凹陷；回傳是否有按壓效果。
    let press_left = move |pos: Pos| match game.with_untracked(|g| g.view(pos.0, pos.1)) {
        CellView::Revealed(n) if n > 0 => {
            show_chord(pos);
            true
        }
        CellView::Hidden | CellView::Question => {
            pressed.set(vec![pos]);
            true
        }
        _ => false,
    };

    let on_down = move |ev: MouseEvent, pos: Pos| {
        if is_over() {
            return;
        }
        ev.prevent_default();
        mouse_down.set_value(true);
        if ev.buttons() == BUTTONS_BOTH || ev.button() == BUTTON_MIDDLE {
            show_chord(pos);
            surprised.set(true);
        } else if ev.button() == BUTTON_LEFT && press_left(pos) {
            surprised.set(true);
        }
    };

    let on_over = move |ev: MouseEvent, pos: Pos| {
        if !mouse_down.get_value() || is_over() {
            return;
        }
        clear();
        match ev.buttons() {
            BUTTONS_BOTH => show_chord(pos),
            BUTTONS_LEFT => {
                press_left(pos);
            }
            _ => {}
        }
    };

    let on_up = move |ev: MouseEvent, pos: Pos| {
        if !mouse_down.get_value() {
            return;
        }
        mouse_down.set_value(false);
        let was_chord = chord_active.get_value();
        clear();
        surprised.set(false);
        if is_over() {
            return;
        }

        game.update(|g| match ev.button() {
            _ if was_chord => {
                g.chord(pos.0, pos.1);
            }
            BUTTON_MIDDLE => {
                g.chord(pos.0, pos.1);
            }
            BUTTON_LEFT => reveal_or_chord(g, pos),
            BUTTON_RIGHT => {
                g.toggle_flag(pos.0, pos.1);
            }
            _ => {}
        });

        let (started, over) = game.with_untracked(|g| (g.is_started(), g.is_over()));
        if over {
            timer.stop();
        } else if started {
            timer.start();
        }
    };

    let on_leave = move |_: MouseEvent| {
        mouse_down.set_value(false);
        clear();
        surprised.set(false);
    };

    let cells = move || {
        let (rows, cols) = dims.get();
        (0..rows)
            .flat_map(move |r| (0..cols).map(move |c| (r, c)))
            .map(|pos| {
                let class = move || {
                    let is_pressed = pressed.with(|p| p.contains(&pos));
                    cell_class(game.with(|g| view_at(g, pos)), is_pressed)
                };
                let text = move || match game.with(|g| view_at(g, pos)) {
                    Some(CellView::Revealed(n)) if n > 0 => n.to_string(),
                    _ => String::new(),
                };
                view! {
                    <div
                        class=class
                        on:mousedown=move |ev| on_down(ev, pos)
                        on:mouseover=move |ev| on_over(ev, pos)
                        on:mouseup=move |ev| on_up(ev, pos)
                    >
                        {text}
                    </div>
                }
            })
            .collect_view()
    };

    view! {
        <div
            class="board"
            style:grid-template-columns=move || format!("repeat({}, 28px)", dims.get().1)
            style:grid-template-rows=move || format!("repeat({}, 28px)", dims.get().0)
            on:mouseleave=on_leave
            on:contextmenu=|ev| ev.prevent_default()
        >
            {cells}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_class_maps_views() {
        assert_eq!(cell_class(Some(CellView::Hidden), false), "cell hidden");
        assert_eq!(
            cell_class(Some(CellView::Revealed(0)), false),
            "cell revealed"
        );
        assert_eq!(
            cell_class(Some(CellView::Revealed(3)), false),
            "cell revealed n3"
        );
        assert_eq!(
            cell_class(Some(CellView::WrongFlag), false),
            "cell mine-wrong"
        );
        assert_eq!(
            cell_class(Some(CellView::Question), true),
            "cell question pressed"
        );
        assert_eq!(cell_class(None, false), "cell hidden");
    }
}
