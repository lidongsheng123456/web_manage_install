/**
 * DOM 操作辅助工具
 */

/**
 * 向日志面板追加一行
 * @param {string} msg - 消息内容
 * @param {string} [type] - 类型: 'info' | 'success' | 'error'
 */
export function addLog(msg, type = '') {
  const log = document.getElementById('log-content');
  if (!log) return;
  const line = document.createElement('div');
  line.className = `log-line ${type ? 'log-' + type : ''}`;
  line.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
  log.appendChild(line);
  log.scrollTop = log.scrollHeight;
}

/**
 * 安全获取 DOM 元素
 * @param {string} id
 * @returns {HTMLElement|null}
 */
export function $(id) {
  return document.getElementById(id);
}
