//! LED 七段顯示器格式化。

/// 將數值轉為 LED 計數器的三個字元。
///
/// 數值限制在 `-99..=999`；負數以 `-` 佔第一位，其餘補零。
///
/// # 範例
///
/// ```
/// use minesweeper_core::led_digits;
/// assert_eq!(led_digits(7), ['0', '0', '7']);
/// assert_eq!(led_digits(-5), ['-', '0', '5']);
/// ```
pub fn led_digits(value: i32) -> [char; 3] {
    let value = value.clamp(-99, 999);
    let text = if value < 0 {
        format!("-{:02}", value.unsigned_abs())
    } else {
        format!("{value:03}")
    };
    let mut chars = text.chars();
    std::array::from_fn(|_| chars.next().unwrap_or('0'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pads_positive_values() {
        assert_eq!(led_digits(0), ['0', '0', '0']);
        assert_eq!(led_digits(42), ['0', '4', '2']);
        assert_eq!(led_digits(999), ['9', '9', '9']);
    }

    #[test]
    fn formats_negative_values() {
        assert_eq!(led_digits(-1), ['-', '0', '1']);
        assert_eq!(led_digits(-99), ['-', '9', '9']);
    }

    #[test]
    fn clamps_out_of_range() {
        assert_eq!(led_digits(1234), ['9', '9', '9']);
        assert_eq!(led_digits(-500), ['-', '9', '9']);
    }
}
