//! 安装业务包
//!
//! 子包职责划分：
//! - `components` — 各组件安装器（JDK/Maven/MySQL/IDEA/Navicat 等）
//! - `workflow`   — 安装流程编排、取消、回滚
//! - `mysql`      — MySQL 专用安装细节
//! - `commands`   — 对外暴露的 Tauri 命令入口

pub mod commands;
pub mod components;
mod mysql;
pub(crate) mod workflow;

pub(crate) use components::utils;
pub(crate) use workflow::events::{emit_done, emit_status};
pub use workflow::rollback::rollback;
