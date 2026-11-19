//! 無外部相依的虛擬亂數產生器。
//!
//! 使用 SplitMix64：速度快、品質足以洗牌地雷位置，且在原生與 WASM
//! 環境行為一致；種子由呼叫端提供，測試時可重現結果。

/// SplitMix64 虛擬亂數產生器。
#[derive(Clone, Debug)]
pub(crate) struct SplitMix64(u64);

impl SplitMix64 {
    /// 以指定種子建立產生器。
    pub(crate) fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// 產生下一個 64 位元亂數。
    pub(crate) fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// 產生 `[0, n)` 範圍內的整數（Lemire 乘法對映）。
    ///
    /// `n` 必須大於 0。
    pub(crate) fn below(&mut self, n: usize) -> usize {
        debug_assert!(n > 0);
        ((u128::from(self.next_u64()) * n as u128) >> 64) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn below_stays_in_range() {
        let mut rng = SplitMix64::new(7);
        for n in 1..200 {
            assert!(rng.below(n) < n);
        }
    }
}
