const { Menu, dialog } = require('electron');

function createMenu(window, handlers) {
  const template = [
    {
      label: '遊戲',
      submenu: [
        {
          label: '新遊戲',
          accelerator: 'F2',
          click: handlers.onNewGame,
        },
        { type: 'separator' },
        {
          label: '初級',
          type: 'radio',
          checked: handlers.getDifficulty() === 'beginner',
          click: () => handlers.onDifficulty('beginner'),
        },
        {
          label: '中級',
          type: 'radio',
          checked: handlers.getDifficulty() === 'intermediate',
          click: () => handlers.onDifficulty('intermediate'),
        },
        {
          label: '高級',
          type: 'radio',
          checked: handlers.getDifficulty() === 'expert',
          click: () => handlers.onDifficulty('expert'),
        },
        { type: 'separator' },
        {
          label: '自訂...',
          click: async () => {
            // 使用對話框讓使用者輸入自訂大小
            const result = await dialog.showMessageBox(window, {
              type: 'question',
              title: '自訂',
              message: '請在主畫面使用預設難度。\n自訂功能：列(9-24) × 欄(9-30) × 雷(10-667)',
              buttons: ['確定'],
            });
          },
        },
        { type: 'separator' },
        {
          label: '標記 (?)',
          type: 'checkbox',
          checked: handlers.getMarkEnabled(),
          click: (menuItem) => {
            const enabled = handlers.onToggleMark();
            menuItem.checked = enabled;
          },
        },
        { type: 'separator' },
        {
          label: '結束',
          accelerator: 'CmdOrCtrl+Q',
          role: 'quit',
        },
      ],
    },
    {
      label: '說明',
      submenu: [
        {
          label: '關於 MineSweeper',
          click: () => {
            dialog.showMessageBox(window, {
              type: 'info',
              title: '關於 MineSweeper',
              message: 'MineSweeper v1.0.0',
              detail: 'Windows 7 風格 MineSweeper\n使用 Electron + 7.css 建立',
            });
          },
        },
      ],
    },
  ];

  const menu = Menu.buildFromTemplate(template);
  Menu.setApplicationMenu(menu);
}

module.exports = { createMenu };
