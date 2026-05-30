//! 安装业务区。
//!
//! - `components`: 各组件安装器和组件级命令。
//! - `conflict`: 环境冲突解决，安装前清理旧版本 PATH、MSI、环境变量。
//! - `workflow`: 安装流程编排、取消、回滚和流程级命令。
//! - `mysql`: MySQL 专用安装细节。

pub mod components;
pub(crate) mod conflict;
mod mysql;
pub(crate) mod workflow;

pub(crate) use components::utils;
pub(crate) use workflow::events::{emit_done, emit_status};
