/**
 * Step4 页面控制器
 *
 * 负责完成报告页面：
 * - 动态渲染安装结果列表
 * - 动态渲染验证命令（仅已安装组件）
 * - 激活按钮（IDEA/Navicat）
 */

import { getState, getComponentsToInstall } from '../state/appState.js';
import { runVerify } from '../services/activationService.js';
import { handleActivation } from '../components/ActivationCard.js';
import { getComponentDisplayName, CORE_COMPONENTS, BUNDLED_TOOLS } from '../config/constants.js';
import { getCurrentWindow } from '../services/tauriApi.js';
import { $ } from '../utils/dom.js';

/**
 * 初始化 Step4 页面事件绑定
 * @param {function} onRestart - 返回首页回调
 */
export function initStep4(onRestart) {
  $('btn-restart').addEventListener('click', onRestart);
  $('btn-finish').addEventListener('click', async () => {
    const win = getCurrentWindow();
    await win.close();
  });

  $('btn-activate-idea').addEventListener('click', function () {
    handleActivation('idea', this, $('activation-result'));
  });
  $('btn-activate-navicat').addEventListener('click', function () {
    handleActivation('navicat', this, $('activation-result'));
  });
}

/**
 * 渲染 Step4 结果页内容
 */
export function renderStep4Results() {
  const state = getState();
  const results = state.installResults.length > 0 ? state.installResults : (window._installResults || []);

  renderResultSummary(results, state);
  renderVerifyCommands(state);
  renderActivationSection(results);
}

function renderResultSummary(results, state) {
  const list = $('result-list');
  const iconEl = $('result-icon');
  const titleEl = $('result-title');
  const descEl = $('result-desc');

  const successCount = results.filter(r => r.success).length;
  const totalCount = results.length;

  if (successCount === totalCount && totalCount > 0) {
    iconEl.className = 'result-icon success';
    iconEl.innerHTML = `<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>`;
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

  const skippedCoreIds = Object.entries(state.coreInstallFlags)
    .filter(([_, v]) => !v)
    .map(([k]) => k);

  let html = '';
  for (const r of results) {
    const name = getComponentDisplayName(r.component, state.versions);
    html += `
      <div class="result-item">
        <span class="result-badge ${r.success ? 'ok' : 'fail'}">${r.success ? '成功' : '失败'}</span>
        <span class="result-text">${name}: ${r.message}</span>
      </div>`;
  }
  for (const comp of skippedCoreIds) {
    const name = getComponentDisplayName(comp, state.versions);
    html += `
      <div class="result-item">
        <span class="result-badge skip">跳过</span>
        <span class="result-text">${name}: 已安装，跳过</span>
      </div>`;
  }
  list.innerHTML = html;
}

function renderVerifyCommands(state) {
  const container = $('verify-commands');
  if (!container) return;

  const installedComponents = getComponentsToInstall();
  const coreInstalled = installedComponents.filter(c =>
    CORE_COMPONENTS.some(cc => cc.id === c.id)
  );

  let html = '';
  for (const comp of coreInstalled) {
    const coreComp = CORE_COMPONENTS.find(cc => cc.id === comp.id);
    if (!coreComp || !coreComp.verifyCmd) continue;
    html += `
      <div class="verify-cmd" data-cmd="${coreComp.verifyCmd}">
        <span class="verify-cmd-text">${coreComp.verifyCmd}</span>
        <span class="verify-cmd-result" id="verify-${comp.id}"></span>
      </div>
    `;
  }
  container.innerHTML = html;

  container.querySelectorAll('.verify-cmd').forEach(el => {
    el.addEventListener('click', () => {
      const cmd = el.dataset.cmd;
      const resultEl = el.querySelector('.verify-cmd-result');
      runVerifyCmd(cmd, resultEl);
    });
  });

  const btnVerifyAll = $('btn-verify-all');
  if (btnVerifyAll) {
    btnVerifyAll.onclick = () => {
      container.querySelectorAll('.verify-cmd').forEach(el => {
        const cmd = el.dataset.cmd;
        const resultEl = el.querySelector('.verify-cmd-result');
        runVerifyCmd(cmd, resultEl);
      });
    };
  }
}

async function runVerifyCmd(cmd, resultEl) {
  resultEl.textContent = '验证中...';
  resultEl.className = 'verify-cmd-result loading';
  try {
    const output = await runVerify(cmd);
    resultEl.textContent = output;
    resultEl.className = 'verify-cmd-result success';
  } catch (e) {
    resultEl.textContent = String(e);
    resultEl.className = 'verify-cmd-result fail';
  }
}

function renderActivationSection(results) {
  const ideaInstalled = results.some(r => r.component === 'idea' && r.success);
  const navicatInstalled = results.some(r => r.component === 'navicat' && r.success);
  const section = $('activation-section');
  const cardIdea = $('card-activate-idea');
  const cardNavicat = $('card-activate-navicat');

  if (ideaInstalled || navicatInstalled) {
    section.classList.remove('hidden');
    if (ideaInstalled) cardIdea.classList.remove('hidden');
    if (navicatInstalled) cardNavicat.classList.remove('hidden');
  } else {
    section.classList.add('hidden');
    if (cardIdea) cardIdea.classList.add('hidden');
    if (cardNavicat) cardNavicat.classList.add('hidden');
  }
}
