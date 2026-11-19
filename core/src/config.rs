//! 難度設定。

use serde::{Deserialize, Serialize};

/// 遊戲面板設定。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// 列數。
    pub rows: usize,
    /// 欄數。
    pub cols: usize,
    /// 地雷數。
    pub mines: usize,
}

/// 預設難度。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Difficulty {
    /// 初級：9×9，10 顆雷。
    Beginner,
    /// 中級：16×16，40 顆雷。
    Intermediate,
    /// 高級：30×16，99 顆雷（預設）。
    #[default]
    Expert,
}

impl Difficulty {
    /// 所有難度，依選單順序排列。
    pub const ALL: [Difficulty; 3] = [Self::Beginner, Self::Intermediate, Self::Expert];

    /// 取得此難度的面板設定。
    pub const fn config(self) -> Config {
        match self {
            Self::Beginner => Config {
                rows: 9,
                cols: 9,
                mines: 10,
            },
            Self::Intermediate => Config {
                rows: 16,
                cols: 16,
                mines: 40,
            },
            Self::Expert => Config {
                rows: 16,
                cols: 30,
                mines: 99,
            },
        }
    }

    /// 選單顯示名稱。
    pub const fn label(self) -> &'static str {
        match self {
            Self::Beginner => "初級",
            Self::Intermediate => "中級",
            Self::Expert => "高級",
        }
    }

    /// 選單項目 ID。
    pub const fn id(self) -> &'static str {
        match self {
            Self::Beginner => "beginner",
            Self::Intermediate => "intermediate",
            Self::Expert => "expert",
        }
    }

    /// 由選單項目 ID 反查難度。
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|d| d.id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_match_classic_values() {
        assert_eq!(
            Difficulty::Beginner.config(),
            Config {
                rows: 9,
                cols: 9,
                mines: 10
            }
        );
        assert_eq!(
            Difficulty::Intermediate.config(),
            Config {
                rows: 16,
                cols: 16,
                mines: 40
            }
        );
        assert_eq!(
            Difficulty::Expert.config(),
            Config {
                rows: 16,
                cols: 30,
                mines: 99
            }
        );
    }

    #[test]
    fn default_is_expert() {
        assert_eq!(Difficulty::default(), Difficulty::Expert);
    }

    #[test]
    fn id_round_trip() {
        for d in Difficulty::ALL {
            assert_eq!(Difficulty::from_id(d.id()), Some(d));
        }
        assert_eq!(Difficulty::from_id("custom"), None);
    }
}
