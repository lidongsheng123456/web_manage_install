/**
 * 激活服务
 *
 * 封装 IDEA 和 Navicat 的激活命令调用。
 * Step1 快捷入口和 Step4 结果页共用此服务。
 */

import { invoke } from './tauriApi.js';

/**
 * 激活 IntelliJ IDEA
 * @returns {Promise<string>} 成功消息
 */
export async function activateIdea() {
  return invoke('activate_idea');
}

/**
 * 激活 Navicat Premium
 * @returns {Promise<string>} 成功消息
 */
export async function activateNavicat() {
  return invoke('activate_navicat');
}

/**
 * 执行验证命令
 * @param {string} cmd - 命令字符串
 * @returns {Promise<string>} 命令输出
 */
export async function runVerify(cmd) {
  return invoke('run_verify', { cmd });
}
