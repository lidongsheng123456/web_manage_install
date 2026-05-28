//! 安装业务包
//!
//! 只在入口声明三类子包：`components` 放组件安装器，`workflow` 放安装流程，
//! `mysql` 放 MySQL 专用安装细节。这样比全部平铺更清爽，也只嵌套一层。

pub mod components;
mod mysql;
pub(crate) mod workflow;

pub(crate) use components::utils;
pub(crate) use workflow::events::{emit_done, emit_status};
pub use workflow::rollback::rollback;
