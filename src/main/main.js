const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const { createMenu } = require('./menu');

// 難度設定：[列數, 欄數, 地雷數]
const DIFFICULTY = {
  beginner:     { rows: 9,  cols: 9,  mines: 10 },
  intermediate: { rows: 16, cols: 16, mines: 40 },
  expert:       { rows: 16, cols: 30, mines: 99 },
};

// 格子大小（像素）— 28px 格子 + 1px gap
const CELL_SIZE = 29;
// 面板外框額外空間（邊距 + 邊框 + 狀態列 + gap）
// X: 8*2(padding) + 1*2(容器border) + 1*2(面板border) + 2(面板padding) + 1(末列無gap補正) = 23
// Y: 8*2(padding) + 1*2(容器border) + 8(gap) + 46+2(狀態列) + 1*2(面板border) + 2(面板padding) + 1 = 87
const PADDING_X = 23;
const PADDING_Y = 87;

let mainWindow = null;
let currentDifficulty = 'expert';
let customConfig = null;
let markEnabled = true;

function getConfig() {
  if (customConfig) return customConfig;
  return DIFFICULTY[currentDifficulty];
}

function createWindow() {
  const config = getConfig();
  const width = config.cols * CELL_SIZE + PADDING_X;
  const height = config.rows * CELL_SIZE + PADDING_Y;

  mainWindow = new BrowserWindow({
    width,
    height,
    useContentSize: true,
    resizable: false,
    maximizable: false,
    fullscreenable: false,
    title: 'MineSweeper',
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });

  mainWindow.loadFile(path.join(__dirname, '..', 'renderer', 'index.html'));

  // 開發模式下開啟 DevTools
  if (process.argv.includes('--dev')) {
    mainWindow.webContents.openDevTools();
  }

  createMenu(mainWindow, {
    onNewGame: () => mainWindow.webContents.send('new-game'),
    onDifficulty: (diff) => {
      currentDifficulty = diff;
      customConfig = null;
      resizeAndRestart();
    },
    onCustom: (config) => {
      customConfig = config;
      resizeAndRestart();
    },
    onToggleMark: () => {
      markEnabled = !markEnabled;
      mainWindow.webContents.send('toggle-mark', markEnabled);
      return markEnabled;
    },
    getDifficulty: () => currentDifficulty,
    getMarkEnabled: () => markEnabled,
  });
}

function resizeAndRestart() {
  const config = getConfig();
  const width = config.cols * CELL_SIZE + PADDING_X;
  const height = config.rows * CELL_SIZE + PADDING_Y;
  mainWindow.setContentSize(width, height);
  mainWindow.webContents.send('set-config', config);
}

// IPC 處理
ipcMain.handle('get-config', () => getConfig());
ipcMain.handle('get-mark-enabled', () => markEnabled);

app.whenReady().then(createWindow);

app.on('window-all-closed', () => {
  app.quit();
});

app.on('activate', () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow();
  }
});
