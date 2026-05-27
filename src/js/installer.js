/**
 * 安装流程模块
 *
 * 负责 Step 3 的下载与安装：
 * - 创建 Tauri Channel 接收实时下载进度
 * - 监听 install-status 事件更新安装状态卡片
 * - 调用后端 install_all 命令启动安装流水线
 * - 支持取消安装和回滚已完成组件
 */

const { invoke, Channel } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

import { installFlags } from './detect.js';
import { goToStep } from './navigation.js';

/** 跟踪已完成安装的组件（用于回滚） */
let completedComponents = [];
/** 当前安装路径（回滚时需要） */
let currentInstallRoot = '';
/** 安装是否正在进行 */
let installing = false;

/**
 * 向安装日志面板追加一行
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
 */
function formatBytes(bytes) {
  if (bytes >= 1048576) return (bytes / 1048576).toFixed(1) + ' MB';
  if (bytes >= 1024) return (bytes / 1024).toFixed(0) + ' KB';
  return bytes + ' B';
}

/**
 * 执行安装流程
 */
export async function runInstall() {
  const installRoot = document.getElementById('install-path').value;
  const mysqlPassword = document.getElementById('mysql-password').value;
  const dryRun = document.getElementById('dry-run').checked;

  currentInstallRoot = installRoot;
  completedComponents = [];
  installing = true;

  const nodeVersion = document.getElementById('ver-nodejs').value;
  const jdkVersion = document.getElementById('ver-jdk').value;
  const mavenVersion = document.getElementById('ver-maven').value;
  const mysqlVersion = document.getElementById('ver-mysql').value;

  /* 模拟测试模式：强制下载全部 7 个资源 */
  const installIdea = dryRun ? true : (document.getElementById('chk-idea')?.checked || false);
  const installNavicat = dryRun ? true : (document.getElementById('chk-navicat')?.checked || false);
  const installRedis = dryRun ? true : (document.getElementById('chk-redis')?.checked || false);

  document.getElementById('name-nodejs').textContent = `Node.js v${nodeVersion}`;
  document.getElementById('name-jdk').textContent = `JDK ${jdkVersion}`;
  document.getElementById('name-maven').textContent = `Maven ${mavenVersion}`;
  document.getElementById('name-mysql').textContent = `MySQL ${mysqlVersion}`;

  /* 显示/隐藏附加工具进度卡片 */
  const showIdea = dryRun || installIdea;
  const showNavicat = dryRun || installNavicat;
  const showRedis = dryRun || installRedis;
  const progIdea = document.getElementById('prog-idea');
  const progNavicat = document.getElementById('prog-navicat');
  const progRedis = document.getElementById('prog-redis');
  if (progIdea) progIdea.style.display = showIdea ? '' : 'none';
  if (progNavicat) progNavicat.style.display = showNavicat ? '' : 'none';
  if (progRedis) progRedis.style.display = showRedis ? '' : 'none';

  /* 显示取消按钮，隐藏回滚/返回按钮 */
  document.getElementById('btn-cancel-install').style.display = '';
  document.getElementById('btn-rollback').style.display = 'none';
  document.getElementById('btn-back-to-config').style.display = 'none';
  document.getElementById('btn-next-3').disabled = true;

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
        completedComponents.push(d.component);
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

  /* 监听取消事件 */
  const unlistenCancel = await listen('install-cancelled', (event) => {
    const { message } = event.payload;
    addLog(message, 'error');
  });

  /* 调用后端安装命令 */
  try {
    const config = {
      installRoot,
      mysqlPassword,
      installNodejs: dryRun ? true : installFlags.nodejs,
      installJdk: dryRun ? true : installFlags.jdk,
      installMaven: dryRun ? true : installFlags.maven,
      installMysql: dryRun ? true : installFlags.mysql,
      installIdea,
      installNavicat,
      installRedis,
      dryRun,
      nodeVersion,
      jdkVersion,
      mavenVersion,
      mysqlVersion,
    };

    const results = await invoke('install_all', { config, onProgress });
    window._installResults = results;

    const wasCancelled = results.length < [
      installFlags.nodejs, installFlags.jdk, installFlags.maven, installFlags.mysql,
      installIdea, installNavicat, installRedis
    ].filter(Boolean).length;

    if (wasCancelled) {
      addLog('安装被取消', 'error');
    } else {
      addLog('所有安装任务完成', 'success');
    }

    document.getElementById('btn-next-3').disabled = false;
  } catch (e) {
    addLog(`安装出错: ${e}`, 'error');
    window._installResults = [{ component: 'error', success: false, message: String(e) }];
    document.getElementById('btn-next-3').disabled = false;
  }

  installing = false;

  /* 隐藏取消按钮，显示回滚/返回选项 */
  document.getElementById('btn-cancel-install').style.display = 'none';
  if (completedComponents.length > 0) {
    document.getElementById('btn-rollback').style.display = '';
  }
  document.getElementById('btn-back-to-config').style.display = '';

  unlistenStatus();
  unlistenCancel();
}

// ─── 取消按钮事件 ──────────────────────────────────────────

document.getElementById('btn-cancel-install').addEventListener('click', async () => {
  if (!installing) return;
  addLog('正在发送取消信号...', 'error');
  try {
    await invoke('cancel_install');
    addLog('取消信号已发送，等待当前组件完成后停止...', 'error');
    document.getElementById('btn-cancel-install').disabled = true;
    document.getElementById('btn-cancel-install').textContent = '取消中...';
  } catch (e) {
    addLog(`取消失败: ${e}`, 'error');
  }
});

// ─── 回滚按钮事件 ──────────────────────────────────────────

document.getElementById('btn-rollback').addEventListener('click', async () => {
  if (completedComponents.length === 0) return;

  const btn = document.getElementById('btn-rollback');
  btn.disabled = true;
  btn.textContent = '回滚中...';
  addLog(`正在回滚 ${completedComponents.length} 个组件...`, 'info');

  try {
    const rolledBack = await invoke('rollback_install', {
      components: completedComponents,
      installRoot: currentInstallRoot,
    });
    addLog(`回滚完成: ${rolledBack.join(', ')}`, 'success');
    completedComponents = [];
    btn.style.display = 'none';

    /* 重置进度卡片显示 */
    document.querySelectorAll('.progress-card').forEach(c => {
      c.classList.remove('done', 'error');
    });
    document.querySelectorAll('.prog-bar').forEach(b => {
      b.style.width = '0%';
      b.className = 'prog-bar';
    });
    document.querySelectorAll('.prog-status').forEach(s => {
      s.textContent = '已回滚';
      s.className = 'prog-status';
    });
  } catch (e) {
    addLog(`回滚失败: ${e}`, 'error');
    btn.disabled = false;
    btn.textContent = '重试回滚';
  }
});

// ─── 返回配置按钮 ──────────────────────────────────────────

document.getElementById('btn-back-to-config').addEventListener('click', () => {
  goToStep(1);
});
