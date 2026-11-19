//! MineSweeper 前端入口（Leptos CSR，編譯為 WASM）。

mod app;
mod board;
mod led;
mod tauri;
mod timer;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::App);
}
