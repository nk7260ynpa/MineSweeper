/**
 * 踩地雷核心邏輯
 *
 * 狀態定義：
 * - state: 'hidden' | 'revealed' | 'flagged' | 'question'
 * - mine: boolean
 * - adjacentMines: number (0-8)
 */

// 用於 Node.js 測試環境匯出
const isNode = typeof window === 'undefined';

class Game {
  /**
   * 建立遊戲實例
   * @param {number} rows - 列數
   * @param {number} cols - 欄數
   * @param {number} mineCount - 地雷數量
   */
  constructor(rows, cols, mineCount) {
    this.rows = rows;
    this.cols = cols;
    this.mineCount = mineCount;
    this.board = [];
    this.started = false;
    this.gameOver = false;
    this.won = false;
    this.revealedCount = 0;
    this.flagCount = 0;
    this.markEnabled = true;

    this._initBoard();
  }

  /** 初始化空白面板 */
  _initBoard() {
    this.board = [];
    for (let r = 0; r < this.rows; r++) {
      const row = [];
      for (let c = 0; c < this.cols; c++) {
        row.push({
          state: 'hidden',
          mine: false,
          adjacentMines: 0,
        });
      }
      this.board.push(row);
    }
  }

  /**
   * 配置地雷（首次點擊後呼叫）
   * 確保點擊位置 3x3 區域不放置地雷
   * @param {number} safeRow - 安全區域中心列
   * @param {number} safeCol - 安全區域中心欄
   */
  _placeMines(safeRow, safeCol) {
    // 收集所有可放置地雷的位置
    const candidates = [];
    for (let r = 0; r < this.rows; r++) {
      for (let c = 0; c < this.cols; c++) {
        // 排除 3x3 安全區域
        if (Math.abs(r - safeRow) <= 1 && Math.abs(c - safeCol) <= 1) {
          continue;
        }
        candidates.push([r, c]);
      }
    }

    // Fisher-Yates 洗牌
    for (let i = candidates.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [candidates[i], candidates[j]] = [candidates[j], candidates[i]];
    }

    // 放置地雷
    const count = Math.min(this.mineCount, candidates.length);
    for (let i = 0; i < count; i++) {
      const [r, c] = candidates[i];
      this.board[r][c].mine = true;
    }

    // 計算每格相鄰地雷數
    this._calcAdjacent();
  }

  /** 計算所有格子的相鄰地雷數 */
  _calcAdjacent() {
    for (let r = 0; r < this.rows; r++) {
      for (let c = 0; c < this.cols; c++) {
        if (this.board[r][c].mine) continue;
        let count = 0;
        for (const [nr, nc] of this._neighbors(r, c)) {
          if (this.board[nr][nc].mine) count++;
        }
        this.board[r][c].adjacentMines = count;
      }
    }
  }

  /**
   * 取得相鄰格子座標
   * @param {number} r - 列
   * @param {number} c - 欄
   * @returns {Array<[number, number]>} 相鄰座標陣列
   */
  _neighbors(r, c) {
    const result = [];
    for (let dr = -1; dr <= 1; dr++) {
      for (let dc = -1; dc <= 1; dc++) {
        if (dr === 0 && dc === 0) continue;
        const nr = r + dr;
        const nc = c + dc;
        if (nr >= 0 && nr < this.rows && nc >= 0 && nc < this.cols) {
          result.push([nr, nc]);
        }
      }
    }
    return result;
  }

  /**
   * 左鍵點擊揭露格子
   * @param {number} r - 列
   * @param {number} c - 欄
   * @returns {Object} { cells: 受影響的格子, exploded: 是否踩雷, won: 是否勝利 }
   */
  reveal(r, c) {
    if (this.gameOver) return { cells: [], exploded: false, won: false };

    const cell = this.board[r][c];
    if (cell.state === 'flagged' || cell.state === 'revealed') {
      return { cells: [], exploded: false, won: false };
    }

    // 首次點擊：配置地雷
    if (!this.started) {
      this.started = true;
      this._placeMines(r, c);
    }

    // 問號格子也可以揭露
    if (cell.state === 'question') {
      cell.state = 'hidden';
    }

    // 踩到地雷
    if (cell.mine) {
      this.gameOver = true;
      cell.state = 'revealed';
      cell.exploded = true;
      return {
        cells: this._revealAllMines(),
        exploded: true,
        won: false,
      };
    }

    // BFS 揭露
    const revealed = this._bfsReveal(r, c);

    // 檢查勝利
    const won = this._checkWin();

    return { cells: revealed, exploded: false, won };
  }

  /**
   * BFS 揭露空白格連通區域
   * @param {number} startR - 起始列
   * @param {number} startC - 起始欄
   * @returns {Array} 被揭露的格子資訊
   */
  _bfsReveal(startR, startC) {
    const revealed = [];
    const queue = [[startR, startC]];
    const visited = new Set();
    visited.add(`${startR},${startC}`);

    while (queue.length > 0) {
      const [r, c] = queue.shift();
      const cell = this.board[r][c];

      if (cell.state === 'flagged') continue;

      if (cell.state !== 'revealed') {
        cell.state = 'revealed';
        this.revealedCount++;
        revealed.push({ row: r, col: c, cell });
      }

      // 如果是空白格（無相鄰地雷），繼續展開
      if (cell.adjacentMines === 0) {
        for (const [nr, nc] of this._neighbors(r, c)) {
          const key = `${nr},${nc}`;
          if (!visited.has(key)) {
            visited.add(key);
            const neighbor = this.board[nr][nc];
            if (neighbor.state !== 'revealed' && !neighbor.mine) {
              queue.push([nr, nc]);
            }
          }
        }
      }
    }

    return revealed;
  }

  /**
   * Chord Click：數字格 + 周圍旗幟數 = 數字 → 揭露剩餘鄰居
   * @param {number} r - 列
   * @param {number} c - 欄
   * @returns {Object} { cells, exploded, won }
   */
  chord(r, c) {
    if (this.gameOver) return { cells: [], exploded: false, won: false };

    const cell = this.board[r][c];
    if (cell.state !== 'revealed' || cell.adjacentMines === 0) {
      return { cells: [], exploded: false, won: false };
    }

    // 計算周圍旗幟數
    let flagCount = 0;
    for (const [nr, nc] of this._neighbors(r, c)) {
      if (this.board[nr][nc].state === 'flagged') flagCount++;
    }

    if (flagCount !== cell.adjacentMines) {
      return { cells: [], exploded: false, won: false };
    }

    // 揭露剩餘未標記的鄰居
    let allCells = [];
    let exploded = false;

    for (const [nr, nc] of this._neighbors(r, c)) {
      const neighbor = this.board[nr][nc];
      if (neighbor.state === 'hidden' || neighbor.state === 'question') {
        if (neighbor.mine) {
          this.gameOver = true;
          exploded = true;
          neighbor.state = 'revealed';
          neighbor.exploded = true;
          allCells = this._revealAllMines();
          break;
        }
        const revealed = this._bfsReveal(nr, nc);
        allCells.push(...revealed);
      }
    }

    const won = !exploded && this._checkWin();

    return { cells: allCells, exploded, won };
  }

  /**
   * 右鍵循環：hidden → flagged → question → hidden
   * @param {number} r - 列
   * @param {number} c - 欄
   * @returns {Object|null} 格子新狀態或 null
   */
  toggleFlag(r, c) {
    if (this.gameOver) return null;

    const cell = this.board[r][c];
    if (cell.state === 'revealed') return null;

    switch (cell.state) {
      case 'hidden':
        cell.state = 'flagged';
        this.flagCount++;
        break;
      case 'flagged':
        cell.state = this.markEnabled ? 'question' : 'hidden';
        this.flagCount--;
        break;
      case 'question':
        cell.state = 'hidden';
        break;
    }

    return { row: r, col: c, state: cell.state, flagCount: this.flagCount };
  }

  /** 檢查是否勝利 */
  _checkWin() {
    if (this.revealedCount === this.rows * this.cols - this.mineCount) {
      this.gameOver = true;
      this.won = true;
      // 自動標記所有未標記的地雷
      for (let r = 0; r < this.rows; r++) {
        for (let c = 0; c < this.cols; c++) {
          if (this.board[r][c].mine && this.board[r][c].state !== 'flagged') {
            this.board[r][c].state = 'flagged';
            this.flagCount++;
          }
        }
      }
      return true;
    }
    return false;
  }

  /** 遊戲結束時揭露所有地雷 */
  _revealAllMines() {
    const cells = [];
    for (let r = 0; r < this.rows; r++) {
      for (let c = 0; c < this.cols; c++) {
        const cell = this.board[r][c];
        if (cell.mine && cell.state !== 'revealed') {
          // 被旗幟正確標記的地雷保持旗幟狀態
          if (cell.state !== 'flagged') {
            cell.state = 'revealed';
          }
          cells.push({ row: r, col: c, cell });
        }
        // 錯誤的旗幟
        if (!cell.mine && cell.state === 'flagged') {
          cell.wrongFlag = true;
          cells.push({ row: r, col: c, cell });
        }
      }
    }
    return cells;
  }

  /**
   * 取得 chord click 預覽（滑鼠按住時顯示凹陷效果）
   * @param {number} r - 列
   * @param {number} c - 欄
   * @returns {Array<[number, number]>} 會被影響的格子座標
   */
  getChordTargets(r, c) {
    const cell = this.board[r][c];
    if (cell.state !== 'revealed' || cell.adjacentMines === 0) return [];

    const targets = [];
    for (const [nr, nc] of this._neighbors(r, c)) {
      const neighbor = this.board[nr][nc];
      if (neighbor.state === 'hidden' || neighbor.state === 'question') {
        targets.push([nr, nc]);
      }
    }
    return targets;
  }

  /** 取得剩餘地雷數（地雷總數 - 旗幟數） */
  getRemainingMines() {
    return this.mineCount - this.flagCount;
  }
}

if (isNode) {
  module.exports = { Game };
}
