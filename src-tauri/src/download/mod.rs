//! 下载业务模块。
//!
//! 入口文件仅负责组织子模块和导出对外 API；具体职责分别放在
//! `sources / service / stream / cache / proxy / preflight`。

mod cache;
pub mod preflight;
mod proxy;
mod service;
pub mod sources;
mod stream;

pub use proxy::configure_proxy_bypass;
pub use service::download_with_version;
