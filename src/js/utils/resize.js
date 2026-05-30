/**
 * 侧边栏宽度拖拽调整模块
 *
 * 实现类似 VS Code 的侧边栏拖拽调整功能。
 */

const MIN_WIDTH = 120;
const MAX_WIDTH = 280;
const STORAGE_KEY = 'sidebar-width';

export function initSidebarResize() {
  const handle = document.getElementById('sidebar-resize-handle');
  const sidebar = document.querySelector('.app-sidebar');
  if (!handle || !sidebar) return;

  const savedWidth = localStorage.getItem(STORAGE_KEY);
  if (savedWidth) {
    const w = parseInt(savedWidth, 10);
    if (w >= MIN_WIDTH && w <= MAX_WIDTH) {
      sidebar.style.width = w + 'px';
    }
  }

  let dragging = false;
  let startX = 0;
  let startWidth = 0;

  handle.addEventListener('mousedown', (e) => {
    e.preventDefault();
    dragging = true;
    startX = e.clientX;
    startWidth = sidebar.offsetWidth;
    handle.classList.add('active');
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
  });

  document.addEventListener('mousemove', (e) => {
    if (!dragging) return;
    const delta = e.clientX - startX;
    const newWidth = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, startWidth + delta));
    sidebar.style.width = newWidth + 'px';
  });

  document.addEventListener('mouseup', () => {
    if (!dragging) return;
    dragging = false;
    handle.classList.remove('active');
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
    localStorage.setItem(STORAGE_KEY, sidebar.offsetWidth.toString());
  });
}
