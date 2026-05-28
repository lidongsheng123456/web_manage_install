//! 公共类型聚合出口。
//!
//! 业务模块按职责拆分类型文件；这里仅保留兼容性的统一 re-export。

pub use super::cancel::CancelToken;
pub use super::detection::ComponentStatus;
pub use super::download::{DownloadProgress, MirrorSource, PreflightResult};
pub use super::install::{InstallConfig, InstallEvent, InstallResult};
pub use super::version::{VersionCatalog, VersionOption};
