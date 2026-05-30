/**
 * 格式化工具函数
 */

/**
 * 格式化字节数为人类可读字符串
 * @param {number} bytes
 * @returns {string}
 */
export function formatBytes(bytes) {
  if (bytes >= 1048576) return (bytes / 1048576).toFixed(1) + ' MB';
  if (bytes >= 1024) return (bytes / 1024).toFixed(0) + ' KB';
  return bytes + ' B';
}

/**
 * 获取当前时间戳字符串
 * @returns {string}
 */
export function timestamp() {
  return new Date().toLocaleTimeString();
}
