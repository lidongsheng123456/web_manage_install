//! 安装业务区。
//!
//! - `components`: 各组件安装器和组件级命令。
//! - `workflow`: 安装流程编排、取消、回滚和流程级命令。
//! - `mysql`: MySQL 专用安装细节。

pub mod components;
mod mysql;
pub(crate) mod workflow;

pub(crate) use components::utils;
pub(crate) use workflow::events::{emit_done, emit_status};
