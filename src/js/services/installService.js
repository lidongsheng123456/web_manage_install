/**
 * 安装流程服务
 *
 * 封装后端 install_all、cancel_install、rollback_install 命令。
 */

import { invoke, Channel, listen } from './tauriApi.js';

/**
 * 启动安装流程
 * @param {object} config - 安装配置参数
 * @param {function} onProgress - 下载进度回调
 * @param {function} onStatus - 安装状态事件回调
 * @param {function} onCancelled - 取消事件回调
 * @returns {Promise<{results: Array, unlisten: function}>}
 */
export async function startInstall(config, { onProgress, onStatus, onCancelled }) {
  const progressChannel = new Channel();
  progressChannel.onmessage = onProgress;

  const unlistenStatus = await listen('install-status', (event) => {
    onStatus(event.payload);
  });

  const unlistenCancel = await listen('install-cancelled', (event) => {
    onCancelled(event.payload);
  });

  const cleanup = () => {
    unlistenStatus();
    unlistenCancel();
  };

  try {
    const results = await invoke('install_all', { config, onProgress: progressChannel });
    return { results, cleanup };
  } catch (e) {
    cleanup();
    throw e;
  }
}

/**
 * 取消当前安装
 */
export async function cancelInstall() {
  return invoke('cancel_install');
}

/**
 * 回滚已安装组件
 * @param {string[]} components - 要回滚的组件ID列表
 * @param {string} installRoot - 安装路径
 */
export async function rollbackInstall(components, installRoot) {
  return invoke('rollback_install', { components, installRoot });
}
