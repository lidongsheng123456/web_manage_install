/**
 * 认证模块
 *
 * 负责令牌校验、会话持久化和登录/登出状态管理。
 * 与 UI 层解耦，仅提供纯逻辑接口。
 */

const TOKEN = '123456';
const SESSION_KEY = 'auth_session';

/**
 * 校验令牌是否正确
 * @param {string} input - 用户输入的令牌
 * @returns {boolean}
 */
export function validateToken(input) {
  return input.trim() === TOKEN;
}

/**
 * 将认证状态写入 sessionStorage
 */
export function persistSession() {
  sessionStorage.setItem(SESSION_KEY, '1');
}

/**
 * 清除认证会话
 */
export function clearSession() {
  sessionStorage.removeItem(SESSION_KEY);
}

/**
 * 检查当前是否已通过认证
 * @returns {boolean}
 */
export function isAuthenticated() {
  return sessionStorage.getItem(SESSION_KEY) === '1';
}

/**
 * 执行登录流程
 * @param {string} input - 用户输入的令牌
 * @returns {{ success: boolean, message: string }}
 */
export function login(input) {
  if (!input || !input.trim()) {
    return { success: false, message: '请输入访问令牌' };
  }
  if (!validateToken(input)) {
    return { success: false, message: '令牌无效，请检查后重试' };
  }
  persistSession();
  return { success: true, message: '' };
}
