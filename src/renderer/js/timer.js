/**
 * 計時器
 * 首次點擊啟動，最大 999 秒
 */

class Timer {
  /**
   * @param {Counter} counter - LED 計數器實例
   */
  constructor(counter) {
    this.counter = counter;
    this.seconds = 0;
    this.intervalId = null;
    this.running = false;
  }

  /** 啟動計時器 */
  start() {
    if (this.running) return;
    this.running = true;
    this.intervalId = setInterval(() => {
      if (this.seconds < 999) {
        this.seconds++;
        this.counter.setValue(this.seconds);
      }
    }, 1000);
  }

  /** 停止計時器 */
  stop() {
    this.running = false;
    if (this.intervalId !== null) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }

  /** 重置計時器 */
  reset() {
    this.stop();
    this.seconds = 0;
    this.counter.setValue(0);
  }
}
