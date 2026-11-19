//! MineSweeper 核心遊戲邏輯。
//!
//! 本 crate 不依賴任何 GUI 或平台 API，同時供 Tauri 後端（原生）與
//! Leptos 前端（WASM）使用：
//!
//! - [`config`]：難度預設值。
//! - [`game`]：地雷配置、BFS 展開、Chord Click、勝負判定。
//! - [`led`]：LED 七段顯示器的三位數格式化。

pub mod config;
pub mod game;
pub mod led;
mod rng;

pub use config::{Config, Difficulty};
pub use game::{Cell, CellState, CellView, Game, MoveResult, Pos};
pub use led::led_digits;
