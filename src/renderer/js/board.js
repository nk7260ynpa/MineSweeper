/**
 * 面板 DOM 操作與事件處理
 * 使用 CSS Grid 排版，事件委派處理所有格子互動
 */

class Board {
  /**
   * @param {HTMLElement} boardElement - 面板容器
   * @param {HTMLElement} smileyButton - 笑臉按鈕
   * @param {Counter} mineCounter - 地雷計數器
   * @param {Timer} timer - 計時器
   */
  constructor(boardElement, smileyButton, mineCounter, timer) {
    this.boardEl = boardElement;
    this.smileyBtn = smileyButton;
    this.mineCounter = mineCounter;
    this.timer = timer;
    this.game = null;
    this.cells = [];
    this.mouseDown = false;
    this.chordTargets = [];

    this._bindEvents();
  }

  /**
   * 初始化新遊戲
   * @param {Object} config - { rows, cols, mines }
   * @param {boolean} markEnabled - 是否啟用問號標記
   */
  init(config, markEnabled) {
    this.game = new Game(config.rows, config.cols, config.mines);
    this.game.markEnabled = markEnabled;
    this.cells = [];
    this.boardEl.innerHTML = '';
    this.boardEl.style.gridTemplateColumns = `repeat(${config.cols}, 20px)`;
    this.boardEl.style.gridTemplateRows = `repeat(${config.rows}, 20px)`;

    for (let r = 0; r < config.rows; r++) {
      const row = [];
      for (let c = 0; c < config.cols; c++) {
        const cellEl = document.createElement('div');
        cellEl.className = 'cell hidden';
        cellEl.dataset.row = r;
        cellEl.dataset.col = c;
        this.boardEl.appendChild(cellEl);
        row.push(cellEl);
      }
      this.cells.push(row);
    }

    this.timer.reset();
    this.mineCounter.setValue(config.mines);
    this._setSmiley('normal');
  }

  /** 綁定事件 */
  _bindEvents() {
    // 笑臉按鈕
    this.smileyBtn.addEventListener('mousedown', () => {
      this.smileyBtn.classList.add('pressed');
    });
    this.smileyBtn.addEventListener('mouseup', () => {
      this.smileyBtn.classList.remove('pressed');
    });
    this.smileyBtn.addEventListener('click', () => {
      if (this._onNewGame) this._onNewGame();
    });

    // 面板事件委派
    this.boardEl.addEventListener('mousedown', (e) => this._onMouseDown(e));
    this.boardEl.addEventListener('mouseup', (e) => this._onMouseUp(e));
    this.boardEl.addEventListener('mouseleave', () => this._onMouseLeave());
    this.boardEl.addEventListener('mouseover', (e) => this._onMouseOver(e));

    // 禁止右鍵選單
    this.boardEl.addEventListener('contextmenu', (e) => e.preventDefault());
  }

  /** 設定新遊戲回呼 */
  setNewGameCallback(callback) {
    this._onNewGame = callback;
  }

  /** 取得格子座標 */
  _getCellPos(e) {
    const cellEl = e.target.closest('.cell');
    if (!cellEl) return null;
    return {
      row: parseInt(cellEl.dataset.row),
      col: parseInt(cellEl.dataset.col),
      el: cellEl,
    };
  }

  /** 滑鼠按下 */
  _onMouseDown(e) {
    if (!this.game || this.game.gameOver) return;
    e.preventDefault();

    const pos = this._getCellPos(e);
    if (!pos) return;

    this.mouseDown = true;

    // 檢測 chord click（左右同時按或中鍵）
    const isChord = (e.buttons === 3) || (e.button === 1);

    if (isChord) {
      this._showChordPreview(pos.row, pos.col);
      this._setSmiley('surprised');
    } else if (e.button === 0) {
      // 左鍵：顯示凹陷
      const cell = this.game.board[pos.row][pos.col];
      if (cell.state === 'hidden' || cell.state === 'question') {
        pos.el.classList.add('pressed');
        this._setSmiley('surprised');
      }
    }
  }

  /** 滑鼠移動 */
  _onMouseOver(e) {
    if (!this.mouseDown || !this.game || this.game.gameOver) return;

    const pos = this._getCellPos(e);
    if (!pos) return;

    // 清除之前的凹陷效果
    this._clearPressed();

    if (e.buttons === 3) {
      this._showChordPreview(pos.row, pos.col);
    } else if (e.buttons === 1) {
      const cell = this.game.board[pos.row][pos.col];
      if (cell.state === 'hidden' || cell.state === 'question') {
        pos.el.classList.add('pressed');
      }
    }
  }

  /** 滑鼠放開 */
  _onMouseUp(e) {
    if (!this.game || !this.mouseDown) return;
    this.mouseDown = false;

    const pos = this._getCellPos(e);
    this._clearPressed();
    this._clearChordPreview();

    if (this.game.gameOver) return;

    if (!pos) {
      this._setSmiley('normal');
      return;
    }

    const wasChord = this.chordTargets.length > 0;
    this.chordTargets = [];

    if (wasChord || e.button === 1) {
      // Chord click
      this._handleChord(pos.row, pos.col);
    } else if (e.button === 0) {
      // 左鍵揭露
      this._handleReveal(pos.row, pos.col);
    } else if (e.button === 2) {
      // 右鍵標記
      this._handleFlag(pos.row, pos.col);
    }
  }

  /** 滑鼠離開面板 */
  _onMouseLeave() {
    this.mouseDown = false;
    this._clearPressed();
    this._clearChordPreview();
    if (this.game && !this.game.gameOver) {
      this._setSmiley('normal');
    }
  }

  /** 處理左鍵揭露 */
  _handleReveal(r, c) {
    if (!this.game.started) {
      this.timer.start();
    }

    const result = this.game.reveal(r, c);
    this._applyResult(result);
  }

  /** 處理 Chord click */
  _handleChord(r, c) {
    const result = this.game.chord(r, c);
    this._applyResult(result);
  }

  /** 處理右鍵標記 */
  _handleFlag(r, c) {
    const result = this.game.toggleFlag(r, c);
    if (!result) return;

    const cellEl = this.cells[r][c];
    cellEl.className = 'cell';

    switch (result.state) {
      case 'hidden':
        cellEl.classList.add('hidden');
        break;
      case 'flagged':
        cellEl.classList.add('flagged');
        break;
      case 'question':
        cellEl.classList.add('question');
        break;
    }

    this.mineCounter.setValue(this.game.getRemainingMines());
  }

  /** 套用遊戲結果到 DOM */
  _applyResult(result) {
    if (result.cells.length === 0 && !result.exploded && !result.won) {
      this._setSmiley('normal');
      return;
    }

    for (const { row, col, cell } of result.cells) {
      const cellEl = this.cells[row][col];
      cellEl.className = 'cell';

      if (cell.mine) {
        if (cell.exploded) {
          cellEl.classList.add('mine-exploded');
        } else if (cell.state === 'flagged') {
          cellEl.classList.add('flagged');
        } else {
          cellEl.classList.add('mine');
        }
      } else if (cell.wrongFlag) {
        cellEl.classList.add('mine-wrong');
      } else {
        cellEl.classList.add('revealed');
        if (cell.adjacentMines > 0) {
          cellEl.classList.add(`n${cell.adjacentMines}`);
          cellEl.textContent = cell.adjacentMines;
        }
      }
    }

    if (result.exploded) {
      this.timer.stop();
      this._setSmiley('dead');
      this.mineCounter.setValue(0);
    } else if (result.won) {
      this.timer.stop();
      this._setSmiley('win');
      this.mineCounter.setValue(0);
    } else {
      this._setSmiley('normal');
    }
  }

  /** 顯示 Chord 預覽凹陷效果 */
  _showChordPreview(r, c) {
    this._clearChordPreview();
    const targets = this.game.getChordTargets(r, c);
    for (const [nr, nc] of targets) {
      this.cells[nr][nc].classList.add('pressed');
      this.chordTargets.push([nr, nc]);
    }
  }

  /** 清除凹陷效果 */
  _clearPressed() {
    const pressed = this.boardEl.querySelectorAll('.pressed');
    pressed.forEach((el) => el.classList.remove('pressed'));
  }

  /** 清除 Chord 預覽 */
  _clearChordPreview() {
    for (const [r, c] of this.chordTargets) {
      this.cells[r][c].classList.remove('pressed');
    }
    this.chordTargets = [];
  }

  /** 設定笑臉狀態 */
  _setSmiley(state) {
    this.smileyBtn.className = 'smiley-btn';
    this.smileyBtn.classList.add(`smiley-${state}`);
  }
}
