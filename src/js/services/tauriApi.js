/**
 * Tauri IPC 统一封装
 *
 * 对 Tauri API 的统一入口封装，所有后端通信都通过此模块进行，
 * 使业务代码与 Tauri 运行时解耦。
 */

const { invoke, Channel } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { open } = window.__TAURI__.dialog;
const { getCurrentWindow } = window.__TAURI__.window;
const { getVersion } = window.__TAURI__.app;

export { invoke, Channel, listen, open, getCurrentWindow, getVersion };
