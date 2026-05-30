/**
 * 附加工具选择项组件
 *
 * 渲染 Step2 中的附加开发工具选择区域。
 */

import { BUNDLED_TOOLS } from '../config/constants.js';
import { setBundledTool, getState } from '../state/appState.js';

/**
 * 渲染附加工具选择区域 HTML
 * @returns {string} HTML 字符串
 */
export function renderBundledToolsSection() {
  const state = getState();

  const items = BUNDLED_TOOLS.map(tool => {
    const checked = state.bundledTools[tool.id] ? 'checked' : '';
    return `
      <label class="bundled-item" id="bundled-${tool.id}">
        <input type="checkbox" class="bundled-checkbox" data-tool="${tool.id}" ${checked} />
        <div class="comp-icon small ${tool.iconClass}"><img src="${tool.iconSrc}" alt="${tool.name}" /></div>
        <span class="bundled-title">${tool.name}</span>
        <span class="bundled-badge">${tool.badge}</span>
      </label>
    `;
  }).join('');

  return `
    <div class="bundled-tools-section">
      <div class="section-subtitle">
        <h3>附加开发工具 (选填)</h3>
        <p class="form-hint">仅自动下载离线安装包至安装目录，下载完成后需手动执行安装</p>
      </div>
      <div class="bundled-grid">
        ${items}
      </div>
    </div>
  `;
}

/**
 * 绑定附加工具 checkbox 的切换事件（事件委托）
 * @param {HTMLElement} container - 包含附加工具选项的容器
 */
export function bindBundledToolEvents(container) {
  container.addEventListener('change', (e) => {
    if (e.target.matches('.bundled-checkbox[data-tool]')) {
      setBundledTool(e.target.dataset.tool, e.target.checked);
    }
  });
}
