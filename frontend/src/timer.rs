//! 遊戲計時器：首次揭露後開始，上限 999 秒。

use std::time::Duration;

use leptos::prelude::*;

/// 秒數上限。
const MAX_SECONDS: i32 = 999;

/// 計時器（可複製的訊號句柄）。
#[derive(Clone, Copy)]
pub struct Timer {
    seconds: RwSignal<i32>,
    handle: StoredValue<Option<IntervalHandle>>,
}

impl Timer {
    /// 建立歸零的計時器。
    pub fn new() -> Self {
        Self {
            seconds: RwSignal::new(0),
            handle: StoredValue::new(None),
        }
    }

    /// 目前秒數。
    pub fn seconds(self) -> Signal<i32> {
        self.seconds.into()
    }

    /// 開始計時；已在計時則不動作。
    pub fn start(self) {
        if self.handle.with_value(Option::is_some) {
            return;
        }
        let seconds = self.seconds;
        let tick = move || {
            seconds.update(|s| {
                if *s < MAX_SECONDS {
                    *s += 1;
                }
            });
        };
        match set_interval_with_handle(tick, Duration::from_secs(1)) {
            Ok(handle) => self.handle.set_value(Some(handle)),
            Err(err) => leptos::logging::warn!("無法啟動計時器：{err:?}"),
        }
    }

    /// 停止計時。
    pub fn stop(self) {
        if let Some(handle) = self.handle.try_update_value(Option::take).flatten() {
            handle.clear();
        }
    }

    /// 停止並歸零。
    pub fn reset(self) {
        self.stop();
        self.seconds.set(0);
    }
}
