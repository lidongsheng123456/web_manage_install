/**
 * 认证 UI 交互模块
 *
 * 负责登录表单的 DOM 事件绑定和视觉反馈。
 * 仅处理 UI 层，核心逻辑委托给 auth.js。
 */

import { login, isAuthenticated, clearSession } from './auth.js';

/**
 * 初始化登录界面
 * @param {Function} onSuccess - 认证成功后的回调（用于显示主应用）
 */
export function initAuthUI(onSuccess) {
  if (isAuthenticated()) {
    showApp();
    onSuccess();
    return;
  }

  const overlay = document.getElementById('auth-overlay');
  const input = document.getElementById('auth-token');
  const errorEl = document.getElementById('auth-error');
  const submitBtn = document.getElementById('auth-submit');
  const toggleBtn = document.getElementById('auth-toggle-vis');

  toggleBtn.addEventListener('click', () => {
    const isPassword = input.type === 'password';
    input.type = isPassword ? 'text' : 'password';
  });

  function handleSubmit() {
    const result = login(input.value);
    if (result.success) {
      // 开启梦幻云端退出渐变动画
      overlay.classList.add('fade-out');
      
      const appMain = document.getElementById('app-main');
      appMain.style.display = '';
      appMain.style.opacity = '0';
      appMain.style.transition = 'opacity 0.8s cubic-bezier(0.25, 1, 0.5, 1)';
      
      // 稍微延迟以触发过渡
      setTimeout(() => {
        appMain.style.opacity = '1';
      }, 50);

      setTimeout(() => {
        overlay.classList.add('hidden');
        overlay.classList.remove('fade-out');
        onSuccess();
      }, 800);
    } else {
      errorEl.textContent = result.message;
      input.classList.add('shake');
      setTimeout(() => input.classList.remove('shake'), 400);
    }
  }

  submitBtn.addEventListener('click', handleSubmit);

  input.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') handleSubmit();
  });

  input.addEventListener('input', () => {
    errorEl.textContent = '';
  });
}

function showApp() {
  document.getElementById('auth-overlay').classList.add('hidden');
  document.getElementById('app-main').style.display = '';
  document.getElementById('app-main').style.opacity = '1';
}

/**
 * 绑定退出登录按钮事件
 */
export function initLogoutBtn() {
  const btn = document.getElementById('btn-logout');
  if (!btn) return;
  btn.addEventListener('click', () => {
    clearSession();
    const overlay = document.getElementById('auth-overlay');
    
    // 让登录遮罩先透明，然后再慢慢显现，达到行云流水般的倒回动画
    overlay.classList.add('fade-out');
    overlay.classList.remove('hidden');
    
    setTimeout(() => {
      overlay.classList.remove('fade-out');
      document.getElementById('app-main').style.display = 'none';
    }, 50);
    
    document.getElementById('auth-token').value = '';
    document.getElementById('auth-error').textContent = '';
  });
}
