/**
 * 全局状态管理模块
 *
 * 集中管理应用的所有可变状态，替代散落在各模块中的全局变量。
 * 对外暴露读写方法，保持状态变更可追踪。
 */

import { CORE_COMPONENTS, BUNDLED_TOOLS } from '../config/constants.js';

const state = {
  installPath: 'D:\\develop\\software',
  mysqlPassword: '123456',
  dryRun: false,

  versions: { nodejs: '', jdk: '', maven: '', mysql: '' },

  /** 核心环境安装开关（由 Step2 检测结果 + 用户手动切换决定） */
  coreInstallFlags: { nodejs: true, jdk: true, maven: true, mysql: true },

  /** 附加工具安装开关（由 Step2 用户勾选决定） */
  bundledTools: { idea: false, navicat: false, redis: false },

  /** 最近一次环境检测结果 */
  detectResults: [],

  /** 安装结果（Step3 完成后填入） */
  installResults: [],

  /** 当前安装是否正在进行 */
  installing: false,

  /** 已完成安装的组件（用于回滚） */
  completedComponents: [],
};

export function getState() {
  return state;
}

export function setInstallPath(path) {
  state.installPath = path;
}

export function setMysqlPassword(pwd) {
  state.mysqlPassword = pwd;
}

export function setDryRun(val) {
  state.dryRun = val;
}

export function setVersion(component, version) {
  state.versions[component] = version;
}

export function setCoreInstallFlag(component, flag) {
  state.coreInstallFlags[component] = flag;
}

export function setBundledTool(tool, flag) {
  state.bundledTools[tool] = flag;
}

export function setDetectResults(results) {
  state.detectResults = results;
}

export function setInstallResults(results) {
  state.installResults = results;
}

export function setInstalling(val) {
  state.installing = val;
}

export function addCompletedComponent(comp) {
  state.completedComponents.push(comp);
}

export function clearCompletedComponents() {
  state.completedComponents = [];
}

/**
 * 获取实际需要安装的组件列表（供 Step3 动态渲染进度卡片）
 * @returns {Array<{id: string, name: string, iconClass: string, iconSrc: string}>}
 */
export function getComponentsToInstall() {
  const list = [];

  for (const comp of CORE_COMPONENTS) {
    if (state.dryRun || state.coreInstallFlags[comp.id]) {
      list.push(comp);
    }
  }

  for (const tool of BUNDLED_TOOLS) {
    if (state.dryRun || state.bundledTools[tool.id]) {
      list.push(tool);
    }
  }

  return list;
}

/**
 * 重置所有安装相关状态（返回首页时调用）
 */
export function resetInstallState() {
  state.coreInstallFlags = { nodejs: true, jdk: true, maven: true, mysql: true };
  state.bundledTools = { idea: false, navicat: false, redis: false };
  state.detectResults = [];
  state.installResults = [];
  state.installing = false;
  state.completedComponents = [];
}
