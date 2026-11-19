//! LED 七段顯示器元件。

use leptos::prelude::*;
use minesweeper_core::led_digits;

/// 三位數 LED 計數器，數字由 CSS `data-value` 呈現。
#[component]
pub fn LedCounter(#[prop(into)] value: Signal<i32>) -> impl IntoView {
    let digit = move |i: usize| move || led_digits(value.get())[i].to_string();
    view! {
        <div class="led-counter">
            <span class="led-digit" data-value=digit(0)></span>
            <span class="led-digit" data-value=digit(1)></span>
            <span class="led-digit" data-value=digit(2)></span>
        </div>
    }
}
