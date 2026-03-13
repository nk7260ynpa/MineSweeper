const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const { Game } = require('../src/renderer/js/game.js');

describe('Game 建構', () => {
  it('應正確初始化面板大小', () => {
    const game = new Game(9, 9, 10);
    assert.equal(game.rows, 9);
    assert.equal(game.cols, 9);
    assert.equal(game.mineCount, 10);
    assert.equal(game.board.length, 9);
    assert.equal(game.board[0].length, 9);
  });

  it('所有格子初始狀態為 hidden', () => {
    const game = new Game(9, 9, 10);
    for (let r = 0; r < 9; r++) {
      for (let c = 0; c < 9; c++) {
        assert.equal(game.board[r][c].state, 'hidden');
        assert.equal(game.board[r][c].mine, false);
      }
    }
  });

  it('初始狀態未開始', () => {
    const game = new Game(9, 9, 10);
    assert.equal(game.started, false);
    assert.equal(game.gameOver, false);
    assert.equal(game.won, false);
    assert.equal(game.revealedCount, 0);
    assert.equal(game.flagCount, 0);
  });
});

describe('地雷配置', () => {
  it('首次點擊後配置正確數量的地雷', () => {
    const game = new Game(9, 9, 10);
    game.reveal(4, 4);

    let mineCount = 0;
    for (let r = 0; r < 9; r++) {
      for (let c = 0; c < 9; c++) {
        if (game.board[r][c].mine) mineCount++;
      }
    }
    assert.equal(mineCount, 10);
  });

  it('首次點擊位置 3x3 安全區域無地雷', () => {
    const game = new Game(9, 9, 10);
    game.reveal(4, 4);

    for (let r = 3; r <= 5; r++) {
      for (let c = 3; c <= 5; c++) {
        assert.equal(game.board[r][c].mine, false,
          `位置 (${r},${c}) 應為安全區域`);
      }
    }
  });

  it('角落點擊的安全區域正確', () => {
    const game = new Game(9, 9, 10);
    game.reveal(0, 0);

    // 左上角 2x2 區域應安全
    for (let r = 0; r <= 1; r++) {
      for (let c = 0; c <= 1; c++) {
        assert.equal(game.board[r][c].mine, false,
          `位置 (${r},${c}) 應為安全區域`);
      }
    }
  });

  it('相鄰地雷數計算正確', () => {
    const game = new Game(9, 9, 10);
    game.reveal(4, 4);

    for (let r = 0; r < 9; r++) {
      for (let c = 0; c < 9; c++) {
        if (game.board[r][c].mine) continue;

        let expected = 0;
        for (let dr = -1; dr <= 1; dr++) {
          for (let dc = -1; dc <= 1; dc++) {
            if (dr === 0 && dc === 0) continue;
            const nr = r + dr;
            const nc = c + dc;
            if (nr >= 0 && nr < 9 && nc >= 0 && nc < 9) {
              if (game.board[nr][nc].mine) expected++;
            }
          }
        }
        assert.equal(game.board[r][c].adjacentMines, expected,
          `位置 (${r},${c}) 相鄰地雷數應為 ${expected}`);
      }
    }
  });
});

describe('揭露邏輯', () => {
  it('揭露非地雷格子', () => {
    const game = new Game(9, 9, 10);
    const result = game.reveal(4, 4);

    assert.equal(result.exploded, false);
    assert.ok(result.cells.length > 0);
    assert.equal(game.started, true);
  });

  it('揭露已標記旗幟的格子不動作', () => {
    const game = new Game(9, 9, 10);
    game.reveal(4, 4); // 先啟動

    // 找一個未揭露的格子標記旗幟
    let targetR = -1, targetC = -1;
    for (let r = 0; r < 9 && targetR === -1; r++) {
      for (let c = 0; c < 9 && targetR === -1; c++) {
        if (game.board[r][c].state === 'hidden') {
          targetR = r;
          targetC = c;
        }
      }
    }

    game.toggleFlag(targetR, targetC);
    const result = game.reveal(targetR, targetC);
    assert.equal(result.cells.length, 0);
  });

  it('BFS 展開空白區域', () => {
    const game = new Game(9, 9, 1); // 只放1顆雷，大部分是空白
    const result = game.reveal(4, 4);

    // 應展開大量格子
    assert.ok(result.cells.length > 1);
  });

  it('遊戲結束後不能再揭露', () => {
    const game = new Game(9, 9, 10);
    game.gameOver = true;
    const result = game.reveal(0, 0);
    assert.equal(result.cells.length, 0);
  });
});

describe('右鍵標記', () => {
  it('hidden → flagged → question → hidden 循環', () => {
    const game = new Game(9, 9, 10);
    game.markEnabled = true;

    let result = game.toggleFlag(0, 0);
    assert.equal(result.state, 'flagged');
    assert.equal(game.flagCount, 1);

    result = game.toggleFlag(0, 0);
    assert.equal(result.state, 'question');
    assert.equal(game.flagCount, 0);

    result = game.toggleFlag(0, 0);
    assert.equal(result.state, 'hidden');
  });

  it('markEnabled=false 時跳過 question', () => {
    const game = new Game(9, 9, 10);
    game.markEnabled = false;

    let result = game.toggleFlag(0, 0);
    assert.equal(result.state, 'flagged');

    result = game.toggleFlag(0, 0);
    assert.equal(result.state, 'hidden'); // 直接回到 hidden
  });

  it('已揭露格子不能標記', () => {
    const game = new Game(9, 9, 10);
    game.reveal(4, 4);

    const result = game.toggleFlag(4, 4);
    assert.equal(result, null);
  });

  it('getRemainingMines 正確計算', () => {
    const game = new Game(9, 9, 10);
    assert.equal(game.getRemainingMines(), 10);

    game.toggleFlag(0, 0);
    assert.equal(game.getRemainingMines(), 9);

    game.toggleFlag(0, 1);
    assert.equal(game.getRemainingMines(), 8);
  });
});

describe('Chord Click', () => {
  /**
   * 建立可控的測試面板：手動配置地雷
   */
  function setupBoard() {
    const game = new Game(5, 5, 2);
    game.started = true;

    // 手動放地雷在 (0,0) 和 (0,1)
    game.board[0][0].mine = true;
    game.board[0][1].mine = true;
    game._calcAdjacent();

    return game;
  }

  it('旗幟數匹配時揭露鄰居', () => {
    const game = setupBoard();
    // 先揭露 (1,1)，adjacentMines 應該是 2
    game.board[1][1].state = 'revealed';
    game.revealedCount = 1;
    assert.equal(game.board[1][1].adjacentMines, 2);

    // 標記兩顆地雷
    game.toggleFlag(0, 0);
    game.toggleFlag(0, 1);

    const result = game.chord(1, 1);
    assert.equal(result.exploded, false);
    assert.ok(result.cells.length > 0);
  });

  it('旗幟數不匹配時不動作', () => {
    const game = setupBoard();
    game.board[1][1].state = 'revealed';
    game.revealedCount = 1;

    // 只標記一個
    game.toggleFlag(0, 0);

    const result = game.chord(1, 1);
    assert.equal(result.cells.length, 0);
  });

  it('chord 在未揭露格子上不動作', () => {
    const game = setupBoard();
    const result = game.chord(1, 1);
    assert.equal(result.cells.length, 0);
  });

  it('chord 標記錯誤會踩雷', () => {
    const game = setupBoard();
    game.board[1][1].state = 'revealed';
    game.revealedCount = 1;
    // adjacentMines = 2

    // 標記錯誤位置
    game.toggleFlag(0, 0); // 正確
    game.toggleFlag(1, 0); // 錯誤！(1,0) 不是地雷

    const result = game.chord(1, 1);
    // 揭露 (0,1) 是地雷 → 爆炸
    assert.equal(result.exploded, true);
    assert.equal(game.gameOver, true);
  });
});

describe('勝利判定', () => {
  it('揭露所有非地雷格子即勝利', () => {
    const game = new Game(3, 3, 1);
    game.started = true;
    game.board[0][0].mine = true;
    game._calcAdjacent();

    // 手動揭露所有非地雷格子
    for (let r = 0; r < 3; r++) {
      for (let c = 0; c < 3; c++) {
        if (r === 0 && c === 0) continue;
        game.board[r][c].state = 'revealed';
        game.revealedCount++;
      }
    }

    const won = game._checkWin();
    assert.equal(won, true);
    assert.equal(game.won, true);
    assert.equal(game.gameOver, true);
  });

  it('勝利時自動標記剩餘地雷', () => {
    const game = new Game(3, 3, 1);
    game.started = true;
    game.board[0][0].mine = true;
    game._calcAdjacent();

    for (let r = 0; r < 3; r++) {
      for (let c = 0; c < 3; c++) {
        if (r === 0 && c === 0) continue;
        game.board[r][c].state = 'revealed';
        game.revealedCount++;
      }
    }

    game._checkWin();
    assert.equal(game.board[0][0].state, 'flagged');
  });
});

describe('難度設定', () => {
  it('初級: 9x9, 10 雷', () => {
    const game = new Game(9, 9, 10);
    game.reveal(0, 0);
    let mines = 0;
    for (let r = 0; r < 9; r++)
      for (let c = 0; c < 9; c++)
        if (game.board[r][c].mine) mines++;
    assert.equal(mines, 10);
  });

  it('中級: 16x16, 40 雷', () => {
    const game = new Game(16, 16, 40);
    game.reveal(0, 0);
    let mines = 0;
    for (let r = 0; r < 16; r++)
      for (let c = 0; c < 16; c++)
        if (game.board[r][c].mine) mines++;
    assert.equal(mines, 40);
  });

  it('高級: 30x16, 99 雷', () => {
    const game = new Game(16, 30, 99);
    game.reveal(0, 0);
    let mines = 0;
    for (let r = 0; r < 16; r++)
      for (let c = 0; c < 30; c++)
        if (game.board[r][c].mine) mines++;
    assert.equal(mines, 99);
  });
});

describe('getChordTargets', () => {
  it('回傳未揭露的鄰居座標', () => {
    const game = new Game(3, 3, 1);
    game.started = true;
    game.board[0][0].mine = true;
    game._calcAdjacent();

    game.board[1][1].state = 'revealed';
    game.board[0][1].state = 'flagged';

    const targets = game.getChordTargets(1, 1);
    // 應包含 hidden 的鄰居，不包含 flagged 和 revealed
    for (const [r, c] of targets) {
      const state = game.board[r][c].state;
      assert.ok(state === 'hidden' || state === 'question');
    }
  });

  it('未揭露格子回傳空陣列', () => {
    const game = new Game(3, 3, 1);
    const targets = game.getChordTargets(1, 1);
    assert.equal(targets.length, 0);
  });
});
