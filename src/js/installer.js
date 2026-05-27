/**
 * 安装流程模块
 *
 * 负责 Step 3 的下载与安装：
 * - 创建 Tauri Channel 接收实时下载进度
 * - 监听 install-status 事件更新安装状态卡片
 * - 调用后端 install_all 命令启动安装流水线
 */

const { invoke, Channel } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

import { installFlags } from './detect.js';

/**
 * 向安装日志面板追加一行
 * @param {string} msg  - 日志消息
 * @param {string} type - 日志类型: info / success / error
 */
function addLog(msg, type = '') {
  const log = document.getElementById('log-content');
  const line = document.createElement('div');
  line.className = `log-line ${type ? 'log-' + type : ''}`;
  line.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
  log.appendChild(line);
  log.scrollTop = log.scrollHeight;
}

/**
 * 格式化字节数为人类可读字符串
 * @param {number} bytes - 字节数
 * @returns {string} 如 "12.5 MB"
 */
function formatBytes(bytes) {
  if (bytes >= 1048576) return (bytes / 1048576).toFixed(1) + ' MB';
  if (bytes >= 1024) return (bytes / 1024).toFixed(0) + ' KB';
  return bytes + ' B';
}

/**
 * 执行安装流程
 *
 * 从表单读取安装路径、密码、版本等配置，
 * 通过 Channel 接收下载进度，通过 Event 接收安装状态，
 * 最终将结果保存到 window._installResults 供 Step 4 使用。
 */
export async function runInstall() {
  const installRoot = document.getElementById('install-path').value;
  const mysqlPassword = document.getElementById('mysql-password').value;
  const dryRun = document.getElementById('dry-run').checked;

  /* 读取用户选择的版本号 */
  const nodeVersion = document.getElementById('ver-nodejs').value;
  const jdkVersion = document.getElementById('ver-jdk').value;
  const mavenVersion = document.getElementById('ver-maven').value;
  const mysqlVersion = document.getElementById('ver-mysql').value;

  /* 同步版本号到 Step 3 进度卡片 */
  document.getElementById('name-nodejs').textContent = `Node.js v${nodeVersion}`;
  document.getElementById('name-jdk').textContent = `JDK ${jdkVersion}`;
  document.getElementById('name-maven').textContent = `Maven ${mavenVersion}`;
  document.getElementById('name-mysql').textContent = `MySQL ${mysqlVersion}`;

  addLog(`安装路径: ${installRoot}`, 'info');
  if (dryRun) addLog('[测试模式] 仅验证下载，不执行安装', 'info');
  addLog('开始安装流程...', 'info');

  /* 创建下载进度通道 */
  const onProgress = new Channel();
  onProgress.onmessage = (data) => {
    const comp = data.component;
    const bar = document.getElementById(`bar-${comp}`);
    const status = document.getElementById(`status-${comp}`);
    const detail = document.getElementById(`detail-${comp}`);
    if (!bar) return;

    if (data.status === 'cached') {
      bar.style.width = '100%';
      status.textContent = '已缓存';
      status.className = 'prog-status done';
      detail.textContent = '使用已下载的缓存文件';
      addLog(`${comp}: 使用缓存文件`, 'success');
    } else if (data.status === 'downloading') {
      bar.style.width = `${data.percent.toFixed(0)}%`;
      status.textContent = `${data.percent.toFixed(0)}%`;
      status.className = 'prog-status downloading';
      detail.textContent = `${formatBytes(data.downloaded)} / ${formatBytes(data.total)} · ${data.speed}`;
    } else {
      status.textContent = data.status;
      status.className = 'prog-status downloading';
      detail.textContent = data.status;
    }
  };

  /* 监听安装状态事件 */
  const unlistenStatus = await listen('install-status', (event) => {
    const d = event.payload;
    const status = document.getElementById(`status-${d.component}`);
    const bar = document.getElementById(`bar-${d.component}`);
    const card = document.getElementById(`prog-${d.component}`);
    const detail = document.getElementById(`detail-${d.component}`);

    if (d.done) {
      if (d.success) {
        status.textContent = '完成';
        status.className = 'prog-status done';
        bar.style.width = '100%';
        bar.className = 'prog-bar done';
        card.classList.add('done');
        detail.textContent = d.message;
        addLog(`${d.component}: ${d.message}`, 'success');
      } else {
        status.textContent = '失败';
        status.className = 'prog-status error';
        bar.className = 'prog-bar error';
        card.classList.add('error');
        detail.textContent = d.message;
        addLog(`${d.component}: ${d.message}`, 'error');
      }
    } else {
      const phaseText = { download: '下载中', install: '安装中', config: '配置中' };
      status.textContent = phaseText[d.phase] || d.phase;
      status.className = 'prog-status installing';
      detail.textContent = d.message;
      addLog(`${d.component}: ${d.message}`, 'info');
    }
  });

  /* 调用后端安装命令 */
  try {
    const config = {
      installRoot,
      mysqlPassword,
      installNodejs: installFlags.nodejs,
      installJdk: installFlags.jdk,
      installMaven: installFlags.maven,
      installMysql: installFlags.mysql,
      dryRun,
      nodeVersion,
      jdkVersion,
      mavenVersion,
      mysqlVersion,
    };

    const results = await invoke('install_all', { config, onProgress });
    addLog('所有安装任务完成', 'success');
    document.getElementById('btn-next-3').disabled = false;
    window._installResults = results;
  } catch (e) {
    addLog(`安装出错: ${e}`, 'error');
    document.getElementById('btn-next-3').disabled = false;
    window._installResults = [{ component: 'error', success: false, message: String(e) }];
  }

  unlistenStatus();
}
