/**
 * 环境检测服务
 *
 * 封装后端 detect_environment 命令的调用逻辑。
 */

import { invoke } from './tauriApi.js';

/**
 * 执行环境检测
 * @param {{nodeVersion: string, jdkVersion: string, mavenVersion: string, mysqlVersion: string}} versions
 * @returns {Promise<Array>} 检测结果数组
 */
export async function detectEnvironment(versions) {
  return invoke('detect_environment', {
    nodeVersion: versions.nodejs || '20.19.0',
    jdkVersion: versions.jdk || '17',
    mavenVersion: versions.maven || '3.9.6',
    mysqlVersion: versions.mysql || '8.0.36',
  });
}
