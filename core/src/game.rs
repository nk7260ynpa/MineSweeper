//! 遊戲狀態與規則。

use std::collections::VecDeque;

use crate::config::Config;
use crate::rng::SplitMix64;

/// 格子座標 `(列, 欄)`。
pub type Pos = (usize, usize);

/// 格子的玩家可見狀態。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CellState {
    /// 未揭露。
    #[default]
    Hidden,
    /// 已揭露。
    Revealed,
    /// 插旗。
    Flagged,
    /// 問號標記。
    Question,
}

/// 單一格子。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cell {
    /// 玩家可見狀態。
    pub state: CellState,
    /// 是否為地雷。
    pub mine: bool,
    /// 相鄰地雷數（0–8）。
    pub adjacent_mines: u8,
    /// 是否為踩爆的那顆地雷。
    pub exploded: bool,
    /// 遊戲結束時，此旗幟是否插錯（下方不是地雷）。
    pub wrong_flag: bool,
}

/// 格子的畫面呈現方式，前端直接對應到 CSS class。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellView {
    /// 未揭露。
    Hidden,
    /// 插旗。
    Flagged,
    /// 問號。
    Question,
    /// 已揭露的安全格，附相鄰地雷數。
    Revealed(u8),
    /// 遊戲結束時顯示的地雷。
    Mine,
    /// 踩爆的地雷。
    MineExploded,
    /// 插錯的旗幟。
    WrongFlag,
}

/// 一次操作的結果。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MoveResult {
    /// 狀態有變動的格子。
    pub cells: Vec<Pos>,
    /// 是否踩雷。
    pub exploded: bool,
    /// 是否勝利。
    pub won: bool,
}

/// 一局踩地雷遊戲。
///
/// 地雷在第一次揭露時才配置，並保證點擊位置的 3×3 範圍內沒有地雷。
#[derive(Clone, Debug)]
pub struct Game {
    rows: usize,
    cols: usize,
    mine_count: usize,
    cells: Vec<Cell>,
    started: bool,
    game_over: bool,
    won: bool,
    revealed_count: usize,
    flag_count: usize,
    mark_enabled: bool,
    rng: SplitMix64,
}

impl Game {
    /// 依設定建立新遊戲，`seed` 為地雷配置的亂數種子。
    pub fn new(config: Config, seed: u64) -> Self {
        Self {
            rows: config.rows,
            cols: config.cols,
            mine_count: config.mines,
            cells: vec![Cell::default(); config.rows * config.cols],
            started: false,
            game_over: false,
            won: false,
            revealed_count: 0,
            flag_count: 0,
            mark_enabled: true,
            rng: SplitMix64::new(seed),
        }
    }

    /// 以指定的地雷位置建立已開始的遊戲，用於測試或重播。
    pub fn with_mines(rows: usize, cols: usize, mines: &[Pos]) -> Self {
        let mut game = Self::new(
            Config {
                rows,
                cols,
                mines: mines.len(),
            },
            0,
        );
        game.started = true;
        for &(r, c) in mines {
            game.cell_mut(r, c).mine = true;
        }
        game.mine_count = game.cells.iter().filter(|cell| cell.mine).count();
        game.calc_adjacent();
        game
    }

    /// 列數。
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// 欄數。
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// 地雷總數。
    pub fn mine_count(&self) -> usize {
        self.mine_count
    }

    /// 是否已配置地雷（第一次揭露後）。
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// 遊戲是否結束（勝或負）。
    pub fn is_over(&self) -> bool {
        self.game_over
    }

    /// 是否勝利。
    pub fn is_won(&self) -> bool {
        self.won
    }

    /// 是否踩雷落敗。
    pub fn is_lost(&self) -> bool {
        self.game_over && !self.won
    }

    /// 已揭露的安全格數量。
    pub fn revealed_count(&self) -> usize {
        self.revealed_count
    }

    /// 目前插旗數量。
    pub fn flag_count(&self) -> usize {
        self.flag_count
    }

    /// 是否啟用問號標記。
    pub fn mark_enabled(&self) -> bool {
        self.mark_enabled
    }

    /// 設定是否啟用問號標記。
    pub fn set_mark_enabled(&mut self, enabled: bool) {
        self.mark_enabled = enabled;
    }

    /// 剩餘地雷數（地雷總數 − 旗幟數），可能為負數。
    pub fn remaining_mines(&self) -> i32 {
        self.mine_count as i32 - self.flag_count as i32
    }

    /// 取得格子。
    ///
    /// # Panics
    ///
    /// 座標超出面板範圍時 panic。
    pub fn cell(&self, r: usize, c: usize) -> &Cell {
        &self.cells[self.index(r, c)]
    }

    /// 取得格子的畫面呈現方式。
    pub fn view(&self, r: usize, c: usize) -> CellView {
        let cell = self.cell(r, c);
        match cell.state {
            CellState::Hidden => CellView::Hidden,
            CellState::Question => CellView::Question,
            CellState::Flagged if cell.wrong_flag => CellView::WrongFlag,
            CellState::Flagged => CellView::Flagged,
            CellState::Revealed if cell.exploded => CellView::MineExploded,
            CellState::Revealed if cell.mine => CellView::Mine,
            CellState::Revealed => CellView::Revealed(cell.adjacent_mines),
        }
    }

    /// 左鍵揭露格子；第一次揭露時配置地雷。
    pub fn reveal(&mut self, r: usize, c: usize) -> MoveResult {
        if self.game_over {
            return MoveResult::default();
        }
        if matches!(
            self.cell(r, c).state,
            CellState::Flagged | CellState::Revealed
        ) {
            return MoveResult::default();
        }

        if !self.started {
            self.started = true;
            self.place_mines(r, c);
        }

        if self.cell(r, c).mine {
            return self.explode(r, c);
        }

        let cells = self.bfs_reveal(r, c);
        let won = self.check_win();
        MoveResult {
            cells,
            exploded: false,
            won,
        }
    }

    /// Chord Click：數字格周圍旗幟數等於數字時，揭露其餘未標記的鄰居。
    pub fn chord(&mut self, r: usize, c: usize) -> MoveResult {
        if self.game_over {
            return MoveResult::default();
        }
        let cell = *self.cell(r, c);
        if cell.state != CellState::Revealed || cell.adjacent_mines == 0 {
            return MoveResult::default();
        }

        let neighbors = self.neighbors(r, c);
        let flags = neighbors
            .iter()
            .filter(|&&(nr, nc)| self.cell(nr, nc).state == CellState::Flagged)
            .count();
        if flags != usize::from(cell.adjacent_mines) {
            return MoveResult::default();
        }

        let mut cells = Vec::new();
        for (nr, nc) in neighbors {
            // 狀態需即時檢查：前一個鄰居的 BFS 可能已揭露此格。
            if !matches!(
                self.cell(nr, nc).state,
                CellState::Hidden | CellState::Question
            ) {
                continue;
            }
            if self.cell(nr, nc).mine {
                let mut result = self.explode(nr, nc);
                cells.append(&mut result.cells);
                result.cells = cells;
                return result;
            }
            cells.extend(self.bfs_reveal(nr, nc));
        }

        let won = self.check_win();
        MoveResult {
            cells,
            exploded: false,
            won,
        }
    }

    /// 右鍵循環標記：未揭露 → 旗幟 → 問號（若啟用）→ 未揭露。
    ///
    /// 回傳新狀態；遊戲結束或格子已揭露時回傳 `None`。
    pub fn toggle_flag(&mut self, r: usize, c: usize) -> Option<CellState> {
        if self.game_over {
            return None;
        }
        let mark_enabled = self.mark_enabled;
        let state = match self.cell(r, c).state {
            CellState::Revealed => return None,
            CellState::Hidden => {
                self.flag_count += 1;
                CellState::Flagged
            }
            CellState::Flagged => {
                self.flag_count -= 1;
                if mark_enabled {
                    CellState::Question
                } else {
                    CellState::Hidden
                }
            }
            CellState::Question => CellState::Hidden,
        };
        self.cell_mut(r, c).state = state;
        Some(state)
    }

    /// 按住數字格時要顯示凹陷預覽的鄰居（未揭露或問號）。
    pub fn chord_targets(&self, r: usize, c: usize) -> Vec<Pos> {
        let cell = self.cell(r, c);
        if cell.state != CellState::Revealed || cell.adjacent_mines == 0 {
            return Vec::new();
        }
        self.neighbors(r, c)
            .into_iter()
            .filter(|&(nr, nc)| {
                matches!(
                    self.cell(nr, nc).state,
                    CellState::Hidden | CellState::Question
                )
            })
            .collect()
    }

    fn index(&self, r: usize, c: usize) -> usize {
        assert!(
            r < self.rows && c < self.cols,
            "座標 ({r}, {c}) 超出面板範圍"
        );
        r * self.cols + c
    }

    fn cell_mut(&mut self, r: usize, c: usize) -> &mut Cell {
        let index = self.index(r, c);
        &mut self.cells[index]
    }

    /// 相鄰（最多 8 格）座標。
    fn neighbors(&self, r: usize, c: usize) -> Vec<Pos> {
        let rows = r.saturating_sub(1)..=(r + 1).min(self.rows - 1);
        rows.flat_map(|nr| {
            (c.saturating_sub(1)..=(c + 1).min(self.cols - 1)).map(move |nc| (nr, nc))
        })
        .filter(|&pos| pos != (r, c))
        .collect()
    }

    /// 以 Fisher-Yates 洗牌配置地雷，排除 `(safe_r, safe_c)` 周圍 3×3。
    fn place_mines(&mut self, safe_r: usize, safe_c: usize) {
        let mut candidates: Vec<Pos> = (0..self.rows)
            .flat_map(|r| (0..self.cols).map(move |c| (r, c)))
            .filter(|&(r, c)| r.abs_diff(safe_r) > 1 || c.abs_diff(safe_c) > 1)
            .collect();

        for i in (1..candidates.len()).rev() {
            let j = self.rng.below(i + 1);
            candidates.swap(i, j);
        }

        let count = self.mine_count.min(candidates.len());
        for &(r, c) in &candidates[..count] {
            self.cell_mut(r, c).mine = true;
        }
        // 面板太小放不下時，以實際放置數為準，勝利判定才會正確。
        self.mine_count = count;
        self.calc_adjacent();
    }

    fn calc_adjacent(&mut self) {
        for r in 0..self.rows {
            for c in 0..self.cols {
                let count = self
                    .neighbors(r, c)
                    .into_iter()
                    .filter(|&(nr, nc)| self.cell(nr, nc).mine)
                    .count();
                self.cell_mut(r, c).adjacent_mines = count as u8;
            }
        }
    }

    /// BFS 揭露從 `(start_r, start_c)` 起相連的空白區域。
    fn bfs_reveal(&mut self, start_r: usize, start_c: usize) -> Vec<Pos> {
        let mut revealed = Vec::new();
        let mut visited = vec![false; self.cells.len()];
        let mut queue = VecDeque::from([(start_r, start_c)]);
        visited[self.index(start_r, start_c)] = true;

        while let Some((r, c)) = queue.pop_front() {
            let cell = self.cell_mut(r, c);
            if cell.state == CellState::Flagged {
                continue;
            }
            if cell.state != CellState::Revealed {
                cell.state = CellState::Revealed;
                self.revealed_count += 1;
                revealed.push((r, c));
            }
            if self.cell(r, c).adjacent_mines > 0 {
                continue;
            }
            for (nr, nc) in self.neighbors(r, c) {
                let index = self.index(nr, nc);
                if visited[index] {
                    continue;
                }
                visited[index] = true;
                let neighbor = self.cells[index];
                if neighbor.state != CellState::Revealed && !neighbor.mine {
                    queue.push_back((nr, nc));
                }
            }
        }
        revealed
    }

    /// 踩到 `(r, c)` 的地雷：結束遊戲並揭露全部地雷。
    fn explode(&mut self, r: usize, c: usize) -> MoveResult {
        self.game_over = true;
        let cell = self.cell_mut(r, c);
        cell.state = CellState::Revealed;
        cell.exploded = true;
        MoveResult {
            cells: self.reveal_all_mines(),
            exploded: true,
            won: false,
        }
    }

    /// 所有安全格都揭露即勝利，並自動為剩餘地雷插旗。
    fn check_win(&mut self) -> bool {
        if self.revealed_count != self.cells.len() - self.mine_count {
            return false;
        }
        self.game_over = true;
        self.won = true;
        for cell in &mut self.cells {
            if cell.mine && cell.state != CellState::Flagged {
                cell.state = CellState::Flagged;
                self.flag_count += 1;
            }
        }
        true
    }

    /// 落敗時揭露所有地雷，並標出插錯的旗幟。
    fn reveal_all_mines(&mut self) -> Vec<Pos> {
        let cols = self.cols;
        let mut changed = Vec::new();
        for (index, cell) in self.cells.iter_mut().enumerate() {
            let pos = (index / cols, index % cols);
            if cell.mine {
                if cell.exploded || cell.state == CellState::Flagged {
                    changed.push(pos);
                } else if cell.state != CellState::Revealed {
                    cell.state = CellState::Revealed;
                    changed.push(pos);
                }
            } else if cell.state == CellState::Flagged {
                cell.wrong_flag = true;
                changed.push(pos);
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Difficulty;

    fn count_mines(game: &Game) -> usize {
        game.cells.iter().filter(|cell| cell.mine).count()
    }

    /// 5×5 面板，地雷在 (0,0) 與 (0,1)，(1,1) 已揭露（數字 2）。
    fn chord_board() -> Game {
        let mut game = Game::with_mines(5, 5, &[(0, 0), (0, 1)]);
        game.cell_mut(1, 1).state = CellState::Revealed;
        game.revealed_count = 1;
        game
    }

    // --- 建構 ---

    #[test]
    fn new_game_has_correct_size() {
        let game = Game::new(
            Config {
                rows: 9,
                cols: 9,
                mines: 10,
            },
            1,
        );
        assert_eq!((game.rows(), game.cols(), game.mine_count()), (9, 9, 10));
        assert_eq!(game.cells.len(), 81);
    }

    #[test]
    fn new_game_cells_are_hidden_without_mines() {
        let game = Game::new(Difficulty::Beginner.config(), 1);
        assert!(
            game.cells
                .iter()
                .all(|c| c.state == CellState::Hidden && !c.mine)
        );
    }

    #[test]
    fn new_game_is_not_started() {
        let game = Game::new(Difficulty::Beginner.config(), 1);
        assert!(!game.is_started());
        assert!(!game.is_over());
        assert!(!game.is_won());
        assert_eq!(game.revealed_count(), 0);
        assert_eq!(game.flag_count(), 0);
    }

    // --- 地雷配置 ---

    #[test]
    fn first_reveal_places_exact_mine_count() {
        for seed in 0..20 {
            let mut game = Game::new(Difficulty::Beginner.config(), seed);
            game.reveal(4, 4);
            assert_eq!(count_mines(&game), 10);
        }
    }

    #[test]
    fn first_reveal_keeps_3x3_safe() {
        for seed in 0..50 {
            let mut game = Game::new(Difficulty::Beginner.config(), seed);
            game.reveal(4, 4);
            for r in 3..=5 {
                for c in 3..=5 {
                    assert!(
                        !game.cell(r, c).mine,
                        "seed {seed} 位置 ({r},{c}) 應為安全區域"
                    );
                }
            }
        }
    }

    #[test]
    fn corner_reveal_keeps_2x2_safe() {
        for seed in 0..50 {
            let mut game = Game::new(Difficulty::Beginner.config(), seed);
            game.reveal(0, 0);
            for r in 0..=1 {
                for c in 0..=1 {
                    assert!(!game.cell(r, c).mine);
                }
            }
        }
    }

    #[test]
    fn adjacent_counts_are_correct() {
        let mut game = Game::new(Difficulty::Beginner.config(), 3);
        game.reveal(4, 4);
        for r in 0..9usize {
            for c in 0..9usize {
                if game.cell(r, c).mine {
                    continue;
                }
                let mut expected = 0;
                for nr in r.saturating_sub(1)..=(r + 1).min(8) {
                    for nc in c.saturating_sub(1)..=(c + 1).min(8) {
                        if (nr, nc) != (r, c) && game.cell(nr, nc).mine {
                            expected += 1;
                        }
                    }
                }
                assert_eq!(game.cell(r, c).adjacent_mines, expected, "位置 ({r},{c})");
            }
        }
    }

    #[test]
    fn same_seed_same_layout() {
        let mut a = Game::new(Difficulty::Expert.config(), 99);
        let mut b = Game::new(Difficulty::Expert.config(), 99);
        a.reveal(8, 15);
        b.reveal(8, 15);
        assert_eq!(a.cells, b.cells);
    }

    #[test]
    fn presets_place_expected_mines() {
        for d in Difficulty::ALL {
            let mut game = Game::new(d.config(), 5);
            game.reveal(0, 0);
            assert_eq!(count_mines(&game), d.config().mines, "{d:?}");
        }
    }

    #[test]
    fn too_many_mines_are_capped() {
        let mut game = Game::new(
            Config {
                rows: 3,
                cols: 3,
                mines: 20,
            },
            1,
        );
        let result = game.reveal(1, 1);
        assert_eq!(game.mine_count(), 0);
        assert!(result.won);
    }

    // --- 揭露 ---

    #[test]
    fn reveal_safe_cell() {
        let mut game = Game::new(Difficulty::Beginner.config(), 1);
        let result = game.reveal(4, 4);
        assert!(!result.exploded);
        assert!(!result.cells.is_empty());
        assert!(game.is_started());
    }

    #[test]
    fn reveal_flagged_cell_does_nothing() {
        let mut game = Game::with_mines(3, 3, &[(0, 0)]);
        game.toggle_flag(2, 2);
        assert_eq!(game.reveal(2, 2), MoveResult::default());
        assert_eq!(game.cell(2, 2).state, CellState::Flagged);
    }

    #[test]
    fn reveal_question_cell_works() {
        let mut game = Game::with_mines(3, 3, &[(0, 0)]);
        game.toggle_flag(2, 2);
        game.toggle_flag(2, 2);
        assert_eq!(game.cell(2, 2).state, CellState::Question);
        assert!(!game.reveal(2, 2).cells.is_empty());
    }

    #[test]
    fn bfs_expands_empty_area() {
        let mut game = Game::new(
            Config {
                rows: 9,
                cols: 9,
                mines: 1,
            },
            1,
        );
        assert!(game.reveal(4, 4).cells.len() > 1);
    }

    #[test]
    fn bfs_stops_at_flags() {
        let mut game = Game::with_mines(1, 5, &[(0, 4)]);
        game.toggle_flag(0, 1);
        // (0,2) 為空白格，向右展開到 (0,3)；向左被 (0,1) 的旗幟擋住。
        let result = game.reveal(0, 2);
        assert_eq!(result.cells, vec![(0, 2), (0, 3)]);
        assert_eq!(game.cell(0, 1).state, CellState::Flagged);
        assert_eq!(game.cell(0, 0).state, CellState::Hidden);
    }

    #[test]
    fn reveal_after_game_over_does_nothing() {
        let mut game = Game::with_mines(3, 3, &[(0, 0)]);
        assert!(game.reveal(0, 0).exploded);
        assert_eq!(game.reveal(2, 2), MoveResult::default());
    }

    #[test]
    fn reveal_mine_ends_game_and_shows_mines() {
        let mut game = Game::with_mines(3, 3, &[(0, 0), (2, 2)]);
        game.toggle_flag(1, 1); // 插錯
        let result = game.reveal(0, 0);
        assert!(result.exploded);
        assert!(game.is_lost());
        assert_eq!(game.view(0, 0), CellView::MineExploded);
        assert_eq!(game.view(2, 2), CellView::Mine);
        assert_eq!(game.view(1, 1), CellView::WrongFlag);
    }

    // --- 右鍵標記 ---

    #[test]
    fn flag_cycle_with_marks() {
        let mut game = Game::new(Difficulty::Beginner.config(), 1);
        assert_eq!(game.toggle_flag(0, 0), Some(CellState::Flagged));
        assert_eq!(game.flag_count(), 1);
        assert_eq!(game.toggle_flag(0, 0), Some(CellState::Question));
        assert_eq!(game.flag_count(), 0);
        assert_eq!(game.toggle_flag(0, 0), Some(CellState::Hidden));
    }

    #[test]
    fn flag_cycle_without_marks_skips_question() {
        let mut game = Game::new(Difficulty::Beginner.config(), 1);
        game.set_mark_enabled(false);
        assert_eq!(game.toggle_flag(0, 0), Some(CellState::Flagged));
        assert_eq!(game.toggle_flag(0, 0), Some(CellState::Hidden));
    }

    #[test]
    fn revealed_cell_cannot_be_flagged() {
        let mut game = Game::new(Difficulty::Beginner.config(), 1);
        game.reveal(4, 4);
        assert_eq!(game.toggle_flag(4, 4), None);
    }

    #[test]
    fn remaining_mines_counts_flags() {
        let mut game = Game::new(Difficulty::Beginner.config(), 1);
        assert_eq!(game.remaining_mines(), 10);
        game.toggle_flag(0, 0);
        assert_eq!(game.remaining_mines(), 9);
        game.toggle_flag(0, 1);
        assert_eq!(game.remaining_mines(), 8);
    }

    // --- Chord Click ---

    #[test]
    fn chord_reveals_when_flags_match() {
        let mut game = chord_board();
        assert_eq!(game.cell(1, 1).adjacent_mines, 2);
        game.toggle_flag(0, 0);
        game.toggle_flag(0, 1);
        let result = game.chord(1, 1);
        assert!(!result.exploded);
        assert!(!result.cells.is_empty());
    }

    #[test]
    fn chord_ignored_when_flags_mismatch() {
        let mut game = chord_board();
        game.toggle_flag(0, 0);
        assert!(game.chord(1, 1).cells.is_empty());
    }

    #[test]
    fn chord_ignored_on_hidden_cell() {
        let mut game = Game::with_mines(5, 5, &[(0, 0), (0, 1)]);
        assert!(game.chord(1, 1).cells.is_empty());
    }

    #[test]
    fn chord_with_wrong_flag_explodes() {
        let mut game = chord_board();
        game.toggle_flag(0, 0); // 正確
        game.toggle_flag(1, 0); // 錯誤：(1,0) 不是地雷
        let result = game.chord(1, 1);
        assert!(result.exploded);
        assert!(game.is_over());
        assert_eq!(game.view(0, 1), CellView::MineExploded);
    }

    // --- 勝利判定 ---

    #[test]
    fn revealing_all_safe_cells_wins_and_flags_mines() {
        let mut game = Game::with_mines(3, 3, &[(0, 0)]);
        let result = game.reveal(2, 2);
        assert!(result.won);
        assert!(game.is_won());
        assert!(game.is_over());
        assert_eq!(game.cell(0, 0).state, CellState::Flagged);
        assert_eq!(game.remaining_mines(), 0);
    }

    #[test]
    fn chord_can_win() {
        // 2×2：地雷在 (0,0)，揭露 (1,1) 後插旗再 chord，揭露其餘兩格即勝。
        let mut game = Game::with_mines(2, 2, &[(0, 0)]);
        assert_eq!(game.reveal(1, 1).cells, vec![(1, 1)]);
        game.toggle_flag(0, 0);
        let result = game.chord(1, 1);
        assert!(result.won);
        assert_eq!(result.cells.len(), 2);
    }

    // --- Chord 預覽 ---

    #[test]
    fn chord_targets_only_hidden_or_question() {
        let mut game = Game::with_mines(3, 3, &[(0, 0)]);
        game.cell_mut(1, 1).state = CellState::Revealed;
        game.toggle_flag(0, 1);
        game.toggle_flag(2, 2);
        game.toggle_flag(2, 2); // 問號
        let targets = game.chord_targets(1, 1);
        assert_eq!(targets.len(), 7);
        assert!(!targets.contains(&(0, 1)));
        assert!(targets.contains(&(2, 2)));
    }

    #[test]
    fn chord_targets_empty_for_hidden_cell() {
        let game = Game::with_mines(3, 3, &[(0, 0)]);
        assert!(game.chord_targets(1, 1).is_empty());
    }

    // --- 畫面呈現 ---

    #[test]
    fn view_maps_states() {
        let mut game = Game::with_mines(3, 3, &[(0, 0)]);
        assert_eq!(game.view(2, 2), CellView::Hidden);
        game.toggle_flag(0, 0);
        assert_eq!(game.view(0, 0), CellView::Flagged);
        game.toggle_flag(0, 0);
        assert_eq!(game.view(0, 0), CellView::Question);
        game.reveal(1, 1);
        assert_eq!(game.view(1, 1), CellView::Revealed(1));
    }
}
