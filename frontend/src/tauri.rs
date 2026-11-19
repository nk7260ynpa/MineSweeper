//! 與 Tauri 後端溝通的薄封裝（使用 `withGlobalTauri` 注入的 `window.__TAURI__`）。

use leptos::task::spawn_local;
use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke, catch)]
    async fn tauri_invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"], js_name = listen, catch)]
    async fn tauri_listen(event: &str, handler: &js_sys::Function) -> Result<JsValue, JsValue>;
}

/// 呼叫無參數的後端指令並反序列化回傳值。
///
/// 不在 Tauri 環境（例如直接用瀏覽器開啟）時回傳 `Err`。
pub async fn invoke<T: DeserializeOwned>(cmd: &str) -> Result<T, String> {
    // 傳 undefined 讓 JS 端套用預設參數 `{}`。
    let value = tauri_invoke(cmd, JsValue::UNDEFINED)
        .await
        .map_err(|err| format!("{err:?}"))?;
    serde_wasm_bindgen::from_value(value).map_err(|err| err.to_string())
}

/// 呼叫帶參數的後端指令；`args` 序列化為 JS 物件，欄位名即指令參數名。
pub async fn invoke_with<A, T>(cmd: &str, args: &A) -> Result<T, String>
where
    A: Serialize,
    T: DeserializeOwned,
{
    let args = serde_wasm_bindgen::to_value(args).map_err(|err| err.to_string())?;
    let value = tauri_invoke(cmd, args)
        .await
        .map_err(|err| format!("{err:?}"))?;
    serde_wasm_bindgen::from_value(value).map_err(|err| err.to_string())
}

/// 訂閱後端事件，`handler` 收到反序列化後的 payload。
///
/// 訂閱在整個 App 生命週期內有效。
pub fn listen<T, F>(event: &'static str, mut handler: F)
where
    T: DeserializeOwned + 'static,
    F: FnMut(T) + 'static,
{
    let closure = Closure::<dyn FnMut(JsValue)>::new(move |event_obj: JsValue| {
        let payload = js_sys::Reflect::get(&event_obj, &"payload".into()).unwrap_or(JsValue::NULL);
        match serde_wasm_bindgen::from_value(payload) {
            Ok(value) => handler(value),
            Err(err) => web_sys_log(&format!("事件 {event} payload 解析失敗：{err}")),
        }
    });
    spawn_local(async move {
        if let Err(err) = tauri_listen(event, closure.as_ref().unchecked_ref()).await {
            web_sys_log(&format!("無法訂閱事件 {event}：{err:?}"));
        }
        // 事件訂閱需在整個 App 生命週期內存活。
        closure.forget();
    });
}

/// 輸出到瀏覽器主控台。
fn web_sys_log(message: &str) {
    leptos::logging::warn!("{message}");
}
