/**
 * 环境检测卡片组件
 *
 * 渲染单个组件的检测结果（安装状态、版本匹配、安装开关）。
 */

import { setCoreInstallFlag, getState } from '../state/appState.js';

/**
 * 创建检测结果卡片 HTML
 * @param {object} result - 后端返回的 ComponentStatus
 * @param {object} meta - 组件元数据 {id, iconClass, iconSrc}
 * @returns {string} HTML 字符串
 */
export function renderDetectCard(result, meta) {
  const state = getState();
  let dotClass, actionClass, actionText, statusText;

  if (result.installed && result.versionMatch) {
    dotClass = 'green';
    actionClass = 'skip';
    actionText = '已安装';
    statusText = `v${result.version} - 版本匹配`;
    setCoreInstallFlag(meta.id, false);
  } else if (result.installed && !result.versionMatch) {
    dotClass = 'yellow';
    actionClass = 'upgrade';
    actionText = '将重新安装';
    statusText = `当前 v${result.version}，需要 ${result.expectedVersion}`;
    setCoreInstallFlag(meta.id, true);
  } else {
    dotClass = 'red';
    actionClass = 'will-install';
    actionText = '将安装';
    statusText = `未检测到，需要 ${result.expectedVersion}`;
    setCoreInstallFlag(meta.id, true);
  }

  const checked = state.coreInstallFlags[meta.id] ? 'checked' : '';

  return `
    <div class="detect-item">
      <div class="comp-icon ${meta.iconClass} small"><img src="${meta.iconSrc}" alt="${result.name}" /></div>
      <div class="status-dot ${dotClass}"></div>
      <div class="detect-info">
        <strong>${result.name}</strong>
        <span>${statusText}</span>
      </div>
      <label class="skip-checkbox">
        <input type="checkbox" class="switch-checkbox" ${checked} data-comp="${meta.id}" />
        <span>安装</span>
      </label>
      <span class="detect-action ${actionClass}">${actionText}</span>
    </div>
  `;
}

/**
 * 绑定检测卡片中 checkbox 的切换事件（事件委托）
 * @param {HTMLElement} container - 检测结果容器
 */
export function bindDetectCardEvents(container) {
  container.addEventListener('change', (e) => {
    if (e.target.matches('.switch-checkbox[data-comp]')) {
      setCoreInstallFlag(e.target.dataset.comp, e.target.checked);
    }
  });
}
