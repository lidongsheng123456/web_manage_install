/**
 * 进度卡片组件
 *
 * 动态创建安装进度卡片 DOM 节点，供 Step3 按需渲染。
 */

/**
 * 创建一个进度卡片元素
 * @param {{id: string, name: string, iconClass: string, iconSrc: string}} comp - 组件元数据
 * @returns {HTMLElement}
 */
export function createProgressCard(comp) {
  const card = document.createElement('div');
  card.className = 'progress-card';
  card.id = `prog-${comp.id}`;
  card.innerHTML = `
    <div class="prog-header">
      <div class="comp-icon ${comp.iconClass} small"><img src="${comp.iconSrc}" alt="${comp.name}" /></div>
      <span class="prog-name" id="name-${comp.id}">${comp.name}</span>
      <span class="prog-status" id="status-${comp.id}">等待中</span>
    </div>
    <div class="prog-bar-wrap"><div class="prog-bar" id="bar-${comp.id}"></div></div>
    <div class="prog-detail" id="detail-${comp.id}"></div>
  `;
  return card;
}

/**
 * 重置单个进度卡片的状态
 * @param {string} compId - 组件 ID
 */
export function resetProgressCard(compId) {
  const status = document.getElementById(`status-${compId}`);
  const bar = document.getElementById(`bar-${compId}`);
  const card = document.getElementById(`prog-${compId}`);
  const detail = document.getElementById(`detail-${compId}`);

  if (card) card.classList.remove('done', 'error');
  if (bar) { bar.style.width = '0%'; bar.className = 'prog-bar'; }
  if (status) { status.textContent = '等待中'; status.className = 'prog-status'; }
  if (detail) detail.textContent = '';
}
