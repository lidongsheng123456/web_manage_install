/**
 * 应用入口模块
 *
 * 认证守卫优先加载（无 Tauri 依赖），认证通过后再动态加载业务模块。
 */

import { initAuthUI, initLogoutBtn } from './js/authUI.js';

initAuthUI(bootstrapApp);

/**
 * 认证通过后动态加载 Tauri 依赖模块并启动主应用逻辑
 */
async function bootstrapApp() {
  const [
    { goToStep },
    { runDetection, resetFlags },
    { runInstall },
    { renderResults, initResultEvents },
    { loadVersionCatalog }
  ] = await Promise.all([
    import('./js/navigation.js'),
    import('./js/detect.js'),
    import('./js/installer.js'),
    import('./js/results.js'),
    import('./js/versions.js')
  ]);

  const { open } = window.__TAURI__.dialog;

  const CORE_VERSION_SELECTS = ['ver-nodejs', 'ver-jdk', 'ver-maven', 'ver-mysql'];

  function populateVersionSelect(selectId, versions) {
    const select = document.getElementById(selectId);
    if (!select) return;
    select.innerHTML = versions.map(v =>
      `<option value="${v.value}" ${v.default ? 'selected' : ''} title="${v.source || ''}">${v.label}</option>`
    ).join('');
  }

  function renderVersionCatalog(catalog) {
    populateVersionSelect('ver-nodejs', catalog.nodejs);
    populateVersionSelect('ver-jdk', catalog.jdk);
    populateVersionSelect('ver-maven', catalog.maven);
    populateVersionSelect('ver-mysql', catalog.mysql);
  }

  function setCoreVersionSelectsDisabled(disabled) {
    CORE_VERSION_SELECTS.forEach(id => {
      const select = document.getElementById(id);
      if (select) select.disabled = disabled;
    });
  }

  function renderVersionLoading() {
    CORE_VERSION_SELECTS.forEach(id => {
      const select = document.getElementById(id);
      if (!select) return;
      select.innerHTML = '<option value="">正在加载版本...</option>';
      select.disabled = true;
    });
    document.getElementById('btn-next-1').disabled = true;
  }

  function renderVersionError(message) {
    CORE_VERSION_SELECTS.forEach(id => {
      const select = document.getElementById(id);
      if (!select) return;
      select.innerHTML = '<option value="">版本加载失败</option>';
      select.disabled = true;
    });
    document.getElementById('btn-next-1').disabled = true;
    console.error('加载实时版本目录失败', message);
  }

  renderVersionLoading();

  loadVersionCatalog()
    .then(catalog => {
      renderVersionCatalog(catalog);
      setCoreVersionSelectsDisabled(false);
      document.getElementById('btn-next-1').disabled = false;
    })
    .catch(renderVersionError);

  // Step 1: 配置页事件

  document.getElementById('btn-browse').addEventListener('click', async () => {
    try {
      const selected = await open({ directory: true, multiple: false });
      if (selected) {
        document.getElementById('install-path').value = selected;
      }
    } catch (e) {
      console.error('浏览目录失败', e);
    }
  });

  document.getElementById('btn-next-1').addEventListener('click', () => {
    goToStep(2);
    runDetection();
  });

  // Step 2: 检测页事件

  document.getElementById('btn-prev-2').addEventListener('click', () => goToStep(1));

  document.getElementById('btn-next-2').addEventListener('click', () => {
    goToStep(3);
    runInstall();
  });

  // Step 3: 安装页事件

  document.getElementById('btn-next-3').addEventListener('click', () => {
    goToStep(4);
    renderResults();
  });

  // Step 4: 结果页事件

  function resetAndGoHome() {
    resetFlags();
    window._installResults = null;

    document.getElementById('btn-next-2').disabled = true;
    document.getElementById('btn-next-3').disabled = true;
    document.getElementById('btn-cancel-install').style.display = 'none';
    document.getElementById('btn-cancel-install').disabled = false;
    document.getElementById('btn-cancel-install').textContent = '取消安装';
    document.getElementById('btn-rollback').style.display = 'none';
    document.getElementById('btn-back-to-config').style.display = 'none';

    document.getElementById('log-content').innerHTML = '';

    document.querySelectorAll('.progress-card').forEach(c => c.classList.remove('done', 'error'));
    document.querySelectorAll('.prog-bar').forEach(b => { b.style.width = '0%'; b.className = 'prog-bar'; });
    document.querySelectorAll('.prog-status').forEach(s => { s.textContent = '等待中'; s.className = 'prog-status'; });
    document.querySelectorAll('.prog-detail').forEach(d => { d.textContent = ''; });
    ['prog-idea', 'prog-navicat', 'prog-redis'].forEach(id => {
      const el = document.getElementById(id);
      if (el) el.style.display = 'none';
    });

    document.querySelectorAll('.verify-cmd-result').forEach(r => { r.textContent = ''; r.className = 'verify-cmd-result'; });

    goToStep(1);
  }

  initResultEvents(resetAndGoHome);
  initLogoutBtn();
}
