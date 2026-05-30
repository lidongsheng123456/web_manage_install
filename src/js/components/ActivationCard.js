/**
 * 激活卡片组件
 *
 * 渲染 IDEA/Navicat 激活按钮，支持 Step1 快捷入口和 Step4 结果页复用。
 */

import { activateIdea, activateNavicat } from '../services/activationService.js';

/**
 * 执行激活操作并更新 UI
 * @param {'idea'|'navicat'} type - 激活目标
 * @param {HTMLButtonElement} btn - 触发按钮
 * @param {HTMLElement} [resultEl] - 可选的结果展示元素
 */
export async function handleActivation(type, btn, resultEl) {
  const label = type === 'idea' ? 'IDEA' : 'Navicat';
  const activateFn = type === 'idea' ? activateIdea : activateNavicat;

  btn.disabled = true;
  const spanEl = btn.querySelector('span');
  if (spanEl) spanEl.textContent = '激活中...';

  if (resultEl) {
    resultEl.textContent = '';
    resultEl.className = 'activation-result';
  }

  try {
    const msg = await activateFn();
    if (spanEl) spanEl.textContent = '已激活';
    btn.classList.add('activated');

    const card = btn.closest('.activation-card, .activation-shortcut-item');
    if (card) {
      card.classList.add('activated');
      const hint = card.querySelector('.act-card-hint, .shortcut-hint');
      if (hint) hint.textContent = '激活成功，可正常使用';
    }

    if (resultEl) {
      resultEl.textContent = `\u2713 ${msg}`;
      resultEl.className = 'activation-result success';
    }
  } catch (e) {
    if (resultEl) {
      resultEl.textContent = `${label} 激活失败: ${e}`;
      resultEl.className = 'activation-result fail';
    }
    btn.disabled = false;
    if (spanEl) spanEl.textContent = '重试';
  }
}
