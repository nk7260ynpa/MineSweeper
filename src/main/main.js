const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const { createMenu } = require('./menu');

// 難度設定：[列數, 欄數, 地雷數]
const DIFFICULTY = {
  beginner:     { rows: 9,  cols: 9,  mines: 10 },
  intermediate: { rows: 16, cols: 16, mines: 40 },
  expert:       { rows: 16, cols: 30, mines: 99 },
};

// 格子大小（像素）
const CELL_SIZE = 20;
// 面板外框與標題列額外空間
const PADDING_X = 40;
const PADDING_Y = 120;

let mainWindow = null;
let currentDifficulty = 'beginner';
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
    resizable: false,
    maximizable: false,
    fullscreenable: false,
    title: '踩地雷',
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });

  mainWindow.loadFile(path.join(__dirname, '..', 'renderer', 'index.html'));

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
