/**
 * LED 七段顯示器計數器
 * 顯示 3 位數，紅色數字，黑底
 */

class Counter {
  /**
   * @param {HTMLElement} element - 計數器容器元素
   */
  constructor(element) {
    this.element = element;
    this.digits = [];

    // 建立 3 個數字位
    for (let i = 0; i < 3; i++) {
      const digit = document.createElement('span');
      digit.className = 'led-digit';
      digit.dataset.value = '0';
      this.digits.push(digit);
      this.element.appendChild(digit);
    }

    this.setValue(0);
  }

  /**
   * 設定顯示值
   * @param {number} value - 要顯示的數值（-99 ~ 999）
   */
  setValue(value) {
    // 限制範圍
    value = Math.max(-99, Math.min(999, value));

    const isNegative = value < 0;
    const absValue = Math.abs(value);
    const str = String(absValue).padStart(isNegative ? 2 : 3, '0');

    if (isNegative) {
      this.digits[0].dataset.value = '-';
      this.digits[1].dataset.value = str[0];
      this.digits[2].dataset.value = str[1];
    } else {
      for (let i = 0; i < 3; i++) {
        this.digits[i].dataset.value = str[i];
      }
    }
  }
}
