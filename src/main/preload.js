const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('minesweeper', {
  getConfig: () => ipcRenderer.invoke('get-config'),
  getMarkEnabled: () => ipcRenderer.invoke('get-mark-enabled'),
  onNewGame: (callback) => ipcRenderer.on('new-game', callback),
  onSetConfig: (callback) => ipcRenderer.on('set-config', (_e, config) => callback(config)),
  onToggleMark: (callback) => ipcRenderer.on('toggle-mark', (_e, enabled) => callback(enabled)),
});
