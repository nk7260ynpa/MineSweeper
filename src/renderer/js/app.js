/**
 * 應用程式入口
 * 串接所有模組並初始化遊戲
 */

(async function () {
  const boardEl = document.getElementById('board');
  const smileyBtn = document.getElementById('smiley');
  const mineCounterEl = document.getElementById('mine-counter');
  const timerCounterEl = document.getElementById('timer-counter');

  // 建立計數器
  const mineCounter = new Counter(mineCounterEl);
  const timerCounter = new Counter(timerCounterEl);
  const timer = new Timer(timerCounter);

  // 建立面板
  const board = new Board(boardEl, smileyBtn, mineCounter, timer);

  // 取得初始設定
  let config = await window.minesweeper.getConfig();
  let markEnabled = await window.minesweeper.getMarkEnabled();

  function newGame() {
    board.init(config, markEnabled);
  }

  board.setNewGameCallback(newGame);

  // 監聽 IPC 事件
  window.minesweeper.onNewGame(() => newGame());
  window.minesweeper.onSetConfig((newConfig) => {
    config = newConfig;
    newGame();
  });
  window.minesweeper.onToggleMark((enabled) => {
    markEnabled = enabled;
    if (board.game) {
      board.game.markEnabled = enabled;
    }
  });

  // 啟動遊戲
  newGame();
})();
