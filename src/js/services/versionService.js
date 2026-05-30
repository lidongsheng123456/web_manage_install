/**
 * 版本目录服务
 *
 * 从后端加载可用版本目录，填充版本选择器。
 */

import { invoke } from './tauriApi.js';

/**
 * 加载版本目录
 * @returns {Promise<object>} 包含 nodejs/jdk/maven/mysql 版本列表的对象
 */
export async function loadVersionCatalog() {
  return invoke('get_version_catalog');
}
