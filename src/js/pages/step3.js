/**
 * Step3 页面控制器
 *
 * 负责自动安装页面：
 * - 动态渲染仅需安装的组件进度卡片
 * - 管理下载进度和安装状态更新
 * - 取消安装和回滚操作
 */

import { getState, getComponentsToInstall, setInstalling, setInstallResults, addCompletedComponent, clearCompletedComponents } from '../state/appState.js';
import { startInstall, cancelInstall, rollbackInstall } from '../services/installService.js';
import { createProgressCard } from '../components/ProgressCard.js';
import { getComponentDisplayName } from '../config/constants.js';
import { formatBytes } from '../utils/format.js';
import { addLog, $ } from '../utils/dom.js';
import { goToStep } from '../navigation.js';

/**
 * 初始化 Step3 页面事件绑定
 * @param {function} onNext - 查看部署报告的回调
 */
export function initStep3(onNext) {
  $('btn-next-3').addEventListener('click', onNext);

  $('btn-cancel-install').addEventListener('click', async () => {
    const state = getState();
    if (!state.installing) return;
    addLog('正在发送取消信号...', 'error');
    try {
      await cancelInstall();
      addLog('取消信号已发送，等待当前组件完成后停止...', 'error');
      $('btn-cancel-install').disabled = true;
      $('btn-cancel-install').textContent = '取消中...';
    } catch (e) {
      addLog(`取消失败: ${e}`, 'error');
    }
  });

  $('btn-rollback').addEventListener('click', async () => {
    const state = getState();
    if (state.completedComponents.length === 0) return;

    const btn = $('btn-rollback');
    btn.disabled = true;
    btn.textContent = '回滚中...';
    addLog(`正在回滚 ${state.completedComponents.length} 个组件...`, 'info');

    try {
      const rolledBack = await rollbackInstall(state.completedComponents, state.installPath);
      addLog(`回滚完成: ${rolledBack.join(', ')}`, 'success');
      clearCompletedComponents();
      btn.style.display = 'none';

      document.querySelectorAll('.progress-card').forEach(c => c.classList.remove('done', 'error'));
      document.querySelectorAll('.prog-bar').forEach(b => { b.style.width = '0%'; b.className = 'prog-bar'; });
      document.querySelectorAll('.prog-status').forEach(s => { s.textContent = '已回滚'; s.className = 'prog-status'; });
    } catch (e) {
      addLog(`回滚失败: ${e}`, 'error');
      btn.disabled = false;
      btn.textContent = '重试回滚';
    }
  });

  $('btn-back-to-config').addEventListener('click', () => goToStep(1));
}

/**
 * 执行安装流程
 * 动态渲染进度卡片 → 启动安装 → 处理进度事件
 */
export async function runStep3Install() {
  const state = getState();
  const components = getComponentsToInstall();

  renderProgressCards(components);
  setupInstallUI();
  setInstalling(true);
  clearCompletedComponents();

  const config = buildInstallConfig(state, components);

  updateCardNames(state);
  addLog(`安装路径: ${state.installPath}`, 'info');
  if (state.dryRun) addLog('[测试模式] 仅验证下载，不执行安装', 'info');
  addLog('开始安装流程...', 'info');

  try {
    const { results, cleanup } = await startInstall(config, {
      onProgress: handleProgress,
      onStatus: handleStatus,
      onCancelled: handleCancelled,
    });

    cleanup();
    setInstallResults(results);
    window._installResults = results;

    const expectedCount = Object.values(state.coreInstallFlags).filter(Boolean).length
      + Object.values(state.bundledTools).filter(Boolean).length;
    if (results.length < expectedCount) {
      addLog('安装被取消', 'error');
    } else {
      addLog('所有安装任务完成', 'success');
    }

    $('btn-next-3').disabled = false;
  } catch (e) {
    addLog(`安装出错: ${e}`, 'error');
    const errResults = [{ component: 'error', success: false, message: String(e) }];
    setInstallResults(errResults);
    window._installResults = errResults;
    $('btn-next-3').disabled = false;
  }

  setInstalling(false);
  $('btn-cancel-install').style.display = 'none';
  if (getState().completedComponents.length > 0) {
    $('btn-rollback').style.display = '';
  }
  $('btn-back-to-config').style.display = '';
}

function renderProgressCards(components) {
  const container = $('install-progress');
  container.innerHTML = '';
  components.forEach(comp => {
    container.appendChild(createProgressCard(comp));
  });
}

function setupInstallUI() {
  $('btn-cancel-install').style.display = '';
  $('btn-cancel-install').disabled = false;
  $('btn-cancel-install').textContent = '取消部署任务';
  $('btn-rollback').style.display = 'none';
  $('btn-back-to-config').style.display = 'none';
  $('btn-next-3').disabled = true;
  $('log-content').innerHTML = '';
}

function buildInstallConfig(state, components) {
  return {
    installRoot: state.installPath,
    mysqlPassword: state.mysqlPassword,
    installNodejs: state.dryRun ? true : state.coreInstallFlags.nodejs,
    installJdk: state.dryRun ? true : state.coreInstallFlags.jdk,
    installMaven: state.dryRun ? true : state.coreInstallFlags.maven,
    installMysql: state.dryRun ? true : state.coreInstallFlags.mysql,
    installIdea: state.dryRun ? true : state.bundledTools.idea,
    installNavicat: state.dryRun ? true : state.bundledTools.navicat,
    installRedis: state.dryRun ? true : state.bundledTools.redis,
    dryRun: state.dryRun,
    nodeVersion: state.versions.nodejs,
    jdkVersion: state.versions.jdk,
    mavenVersion: state.versions.maven,
    mysqlVersion: state.versions.mysql,
  };
}

function updateCardNames(state) {
  const nameNode = $('name-nodejs');
  const nameJdk = $('name-jdk');
  const nameMaven = $('name-maven');
  const nameMysql = $('name-mysql');

  if (nameNode) nameNode.textContent = `Node.js v${state.versions.nodejs}`;
  if (nameJdk) nameJdk.textContent = `JDK ${state.versions.jdk}`;
  if (nameMaven) nameMaven.textContent = `Maven ${state.versions.maven}`;
  if (nameMysql) nameMysql.textContent = `MySQL ${state.versions.mysql}`;
}

function handleProgress(data) {
  const bar = $(`bar-${data.component}`);
  const status = $(`status-${data.component}`);
  const detail = $(`detail-${data.component}`);
  if (!bar) return;

  if (data.status === 'cached') {
    bar.style.width = '100%';
    status.textContent = '已缓存';
    status.className = 'prog-status done';
    detail.textContent = '使用已下载的缓存文件';
    addLog(`${data.component}: 使用缓存文件`, 'success');
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
}

function handleStatus(d) {
  const status = $(`status-${d.component}`);
  const bar = $(`bar-${d.component}`);
  const card = $(`prog-${d.component}`);
  const detail = $(`detail-${d.component}`);

  if (d.done) {
    if (d.success) {
      status.textContent = '完成';
      status.className = 'prog-status done';
      bar.style.width = '100%';
      bar.className = 'prog-bar done';
      card.classList.add('done');
      detail.textContent = d.message;
      addLog(`${d.component}: ${d.message}`, 'success');
      addCompletedComponent(d.component);
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
}

function handleCancelled(payload) {
  addLog(payload.message, 'error');
}
