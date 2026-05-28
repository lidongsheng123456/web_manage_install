/**
 * 版本目录模块
 *
 * 版本列表完全由后端实时请求获取；前端不再维护固定版本列表。
 */

const { invoke } = window.__TAURI__.core;

/**
 * 从后端实时加载版本目录。
 * @returns {Promise<object>} nodejs / jdk / maven / mysql 四组版本列表
 */
export async function loadVersionCatalog() {
  const catalog = await invoke('get_version_catalog');
  return normalizeCatalog(catalog);
}

/**
 * 获取指定组件的默认版本号。
 * @param {string} component - 组件标识 (nodejs / jdk / maven / mysql)
 * @param {object} catalog - 实时版本目录
 * @returns {string} 默认版本号
 */
export function getDefaultVersion(component, catalog) {
  const list = catalog?.[component] || [];
  const found = list.find(v => v.default);
  return found ? found.value : (list[0]?.value || '');
}

function normalizeCatalog(catalog) {
  return {
    nodejs: normalizeList(catalog?.nodejs, 'Node.js'),
    jdk: normalizeList(catalog?.jdk, 'JDK'),
    maven: normalizeList(catalog?.maven, 'Maven'),
    mysql: normalizeList(catalog?.mysql, 'MySQL'),
  };
}

function normalizeList(list, componentName) {
  if (!Array.isArray(list) || list.length === 0) {
    throw new Error(`${componentName} 未获取到可用版本`);
  }

  const normalized = list.map(item => ({
    value: item.value,
    label: item.label || item.value,
    default: Boolean(item.default),
    lts: Boolean(item.lts),
    source: item.source || '实时请求',
  }));

  if (!normalized.some(item => item.default)) {
    normalized[0].default = true;
  }
  return normalized;
}
