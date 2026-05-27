/**
 * 版本配置模块
 *
 * 定义各组件可选的版本列表。
 * 新增版本时只需在对应数组中添加条目，
 * 后端 download.rs 的 get_mirrors_versioned() 会根据版本号动态生成镜像 URL。
 */

/** Node.js 可选版本（LTS 优先） */
export const NODE_VERSIONS = [
  { value: '20.19.0', label: 'v20.19.0 (LTS)', default: true },
  { value: '22.13.1', label: 'v22.13.1 (LTS)' },
  { value: '18.20.4', label: 'v18.20.4 (维护期 LTS)' },
];

/** JDK 可选版本 */
export const JDK_VERSIONS = [
  { value: '17', label: 'JDK 17 (LTS)', default: true },
  { value: '21', label: 'JDK 21 (LTS)' },
];

/** Maven 可选版本 */
export const MAVEN_VERSIONS = [
  { value: '3.9.6', label: 'Maven 3.9.6', default: true },
  { value: '3.9.9', label: 'Maven 3.9.9' },
  { value: '3.8.8', label: 'Maven 3.8.8' },
];

/** MySQL 可选版本 */
export const MYSQL_VERSIONS = [
  { value: '8.0.36', label: 'MySQL 8.0.36', default: true },
  { value: '8.0.37', label: 'MySQL 8.0.37' },
];

/**
 * 获取指定组件的默认版本号
 * @param {string} component - 组件标识 (nodejs / jdk / maven / mysql)
 * @returns {string} 默认版本号
 */
export function getDefaultVersion(component) {
  const map = { nodejs: NODE_VERSIONS, jdk: JDK_VERSIONS, maven: MAVEN_VERSIONS, mysql: MYSQL_VERSIONS };
  const list = map[component] || [];
  const found = list.find(v => v.default);
  return found ? found.value : (list[0]?.value || '');
}
