/**
 * 应用入口模块
 *
 * 认证守卫优先加载（无 Tauri 依赖），认证通过后再动态加载业务模块。
 * 对标 Spring Boot 的 Application.java，负责引导启动和模块编排。
 */

import { initAuthUI, initLogoutBtn } from './js/authUI.js';
import { initSidebarResize } from './js/utils/resize.js';

initAuthUI(bootstrapApp);
initSidebarResize();

/**
 * 认证通过后动态加载 Tauri 依赖模块并启动主应用逻辑
 */
async function bootstrapApp() {
  const [
    { goToStep },
    { initStep1 },
    { initStep2, runStep2Detection },
    { initStep3, runStep3Install },
    { initStep4, renderStep4Results },
    { getVersion }
  ] = await Promise.all([
    import('./js/navigation.js'),
    import('./js/pages/step1.js'),
    import('./js/pages/step2.js'),
    import('./js/pages/step3.js'),
    import('./js/pages/step4.js'),
    import('./js/services/tauriApi.js'),
  ]);

  const { resetInstallState } = await import('./js/state/appState.js');

  displayVersion(getVersion);

  initStep1(() => {
    goToStep(2);
    runStep2Detection();
  });

  initStep2(
    () => goToStep(1),
    () => {
      goToStep(3);
      runStep3Install();
    }
  );

  initStep3(() => {
    goToStep(4);
    renderStep4Results();
  });

  initStep4(resetAndGoHome);

  initLogoutBtn();

  function resetAndGoHome() {
    resetInstallState();
    window._installResults = null;

    const btnNext2 = document.getElementById('btn-next-2');
    const btnNext3 = document.getElementById('btn-next-3');
    const btnCancel = document.getElementById('btn-cancel-install');
    const btnRollback = document.getElementById('btn-rollback');
    const btnBackConfig = document.getElementById('btn-back-to-config');

    if (btnNext2) btnNext2.disabled = true;
    if (btnNext3) btnNext3.disabled = true;
    if (btnCancel) { btnCancel.style.display = 'none'; btnCancel.disabled = false; btnCancel.textContent = '取消安装'; }
    if (btnRollback) btnRollback.style.display = 'none';
    if (btnBackConfig) btnBackConfig.style.display = 'none';

    const logContent = document.getElementById('log-content');
    if (logContent) logContent.innerHTML = '';

    const installProgress = document.getElementById('install-progress');
    if (installProgress) installProgress.innerHTML = '';

    goToStep(1);
  }
}

async function displayVersion(getVersion) {
  try {
    const version = await getVersion();
    const badge = document.getElementById('app-version');
    if (badge) badge.textContent = `v${version}`;
  } catch (_) {}
}
