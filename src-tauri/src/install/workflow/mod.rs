//! 安装流程控制相关模块。

mod cancel;
pub(crate) mod commands;
mod dry_run;
pub(crate) mod events;
pub(crate) mod orchestrator;
mod privilege;
mod result;
pub mod rollback;
