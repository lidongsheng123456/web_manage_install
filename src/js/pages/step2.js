/**
 * Step2 页面控制器
 *
 * 负责环境检测与安装选择页面：
 * - 调用后端进行环境扫描
 * - 渲染核心环境检测结果（含安装开关）
 * - 渲染附加工具选择区域
 * - 统一管理所有"装不装"的决策
 */

import { getState, setDetectResults } from '../state/appState.js';
import { detectEnvironment } from '../services/detectionService.js';
import { renderDetectCard, bindDetectCardEvents } from '../components/DetectCard.js';
import { renderBundledToolsSection, bindBundledToolEvents } from '../components/BundledToolItem.js';
import { CORE_COMPONENTS } from '../config/constants.js';
import { $ } from '../utils/dom.js';

/**
 * 初始化 Step2 页面
 * @param {function} onPrev - 点击上一步的回调
 * @param {function} onNext - 点击下一步的回调
 */
export function initStep2(onPrev, onNext) {
  $('btn-prev-2').addEventListener('click', onPrev);
  $('btn-next-2').addEventListener('click', onNext);
}

/**
 * 执行环境检测并渲染结果
 */
export async function runStep2Detection() {
  const container = $('detect-results');
  const bundledContainer = $('bundled-tools-area');

  container.innerHTML = `<div class="detect-loading"><div class="spinner"></div><span>正在扫描环境...</span></div>`;

  const state = getState();

  try {
    const results = await detectEnvironment(state.versions);
    setDetectResults(results);
    renderDetectionResults(results, container);
    renderBundledTools(bundledContainer);
    $('btn-next-2').disabled = false;
  } catch (e) {
    container.innerHTML = `<div class="detect-loading" style="color:var(--error)">检测失败: ${e}</div>`;
  }
}

function renderDetectionResults(results, container) {
  const nameToMeta = Object.fromEntries(
    CORE_COMPONENTS.map(c => [c.name, c])
  );

  const html = results.map(r => {
    const meta = nameToMeta[r.name];
    if (!meta) return '';
    return renderDetectCard(r, meta);
  }).join('');

  container.innerHTML = html;
  bindDetectCardEvents(container);
}

function renderBundledTools(container) {
  if (!container) return;
  container.innerHTML = renderBundledToolsSection();
  bindBundledToolEvents(container);
}
