/**
 * 结果页模块
 *
 * 负责 Step 4 的安装结果展示和验证命令执行：
 * - 汇总展示各组件安装/跳过/失败状态
 * - 提供一键验证功能，直接调用后端执行命令
 */

const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;

import { installFlags } from './detect.js';

/**
 * 渲染安装结果列表
 *
 * 根据 window._installResults 显示成功/失败/跳过状态，
 * 并更新顶部图标和标题。
 */
export function renderResults() {
  const results = window._installResults || [];
  const list = document.getElementById('result-list');
  const iconEl = document.getElementById('result-icon');
  const titleEl = document.getElementById('result-title');
  const descEl = document.getElementById('result-desc');

  /** 组件名称映射 */
  const nameMap = {
    nodejs: `Node.js ${document.getElementById('ver-nodejs').value}`,
    jdk: `JDK ${document.getElementById('ver-jdk').value}`,
    maven: `Maven ${document.getElementById('ver-maven').value}`,
    mysql: `MySQL ${document.getElementById('ver-mysql').value}`,
  };

  const successCount = results.filter(r => r.success).length;
  const totalCount = results.length;
  const skippedComponents = Object.entries(installFlags).filter(([_, v]) => !v).map(([k]) => k);

  /* 更新顶部状态图标和标题 */
  if (successCount === totalCount && totalCount > 0) {
    iconEl.className = 'result-icon success';
    titleEl.textContent = '安装完成';
    descEl.textContent = '所有选中的组件均已成功安装并配置完毕。';
  } else if (successCount > 0) {
    iconEl.className = 'result-icon partial';
    iconEl.innerHTML = `<svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>`;
    titleEl.textContent = '部分安装完成';
    descEl.textContent = `${successCount}/${totalCount} 个组件安装成功，部分失败请查看详情。`;
  } else {
    iconEl.className = 'result-icon fail';
    iconEl.innerHTML = `<svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>`;
    titleEl.textContent = '安装失败';
    descEl.textContent = '所有组件安装失败，请检查网络或权限后重试。';
  }

  /* 渲染结果明细列表 */
  let html = '';
  for (const r of results) {
    const name = nameMap[r.component] || r.component;
    html += `
      <div class="result-item">
        <span class="result-badge ${r.success ? 'ok' : 'fail'}">${r.success ? '成功' : '失败'}</span>
        <span class="result-text">${name}: ${r.message}</span>
      </div>`;
  }
  for (const comp of skippedComponents) {
    const name = nameMap[comp] || comp;
    html += `
      <div class="result-item">
        <span class="result-badge skip">跳过</span>
        <span class="result-text">${name}: 已安装，跳过</span>
      </div>`;
  }
  list.innerHTML = html;
}

/**
 * 执行单条验证命令
 * @param {string} cmd - 要执行的命令（白名单限制）
 * @param {HTMLElement} resultEl - 用于显示结果的元素
 */
async function runVerifyCmd(cmd, resultEl) {
  resultEl.textContent = '验证中...';
  resultEl.className = 'verify-cmd-result loading';
  try {
    const output = await invoke('run_verify', { cmd });
    resultEl.textContent = output;
    resultEl.className = 'verify-cmd-result success';
  } catch (e) {
    resultEl.textContent = String(e);
    resultEl.className = 'verify-cmd-result fail';
  }
}

/**
 * 初始化 Step 4 的事件监听：
 * - 返回首页按钮
 * - 关闭窗口按钮
 * - 验证命令点击
 * - 一键验证按钮
 * @param {Function} resetAndGoHome - 重置 UI 并回到 Step 1 的回调
 */
export function initResultEvents(resetAndGoHome) {
  /* 返回首页 */
  document.getElementById('btn-restart').addEventListener('click', resetAndGoHome);

  /* 关闭窗口 */
  document.getElementById('btn-finish').addEventListener('click', async () => {
    const win = getCurrentWindow();
    await win.close();
  });

  /* 单击验证命令 */
  document.querySelectorAll('.verify-cmd').forEach(el => {
    el.addEventListener('click', () => {
      const cmd = el.dataset.cmd;
      const resultEl = el.querySelector('.verify-cmd-result');
      runVerifyCmd(cmd, resultEl);
    });
  });

  /* 一键验证全部 */
  document.getElementById('btn-verify-all').addEventListener('click', () => {
    document.querySelectorAll('.verify-cmd').forEach(el => {
      const cmd = el.dataset.cmd;
      const resultEl = el.querySelector('.verify-cmd-result');
      runVerifyCmd(cmd, resultEl);
    });
  });
}
