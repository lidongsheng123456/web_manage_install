//! 可执行文件发现门面
//!
//! 各种查找策略拆到独立 finder 文件；这里保留统一出口，便于组件检测
//! 使用 `env::*` 时不感知内部拆分。

mod find_app_paths;
mod find_common_dirs;
mod find_program_files;
mod find_uninstall;
mod find_where;

pub use find_app_paths::check_app_paths;
pub use find_common_dirs::{scan_common_install_dirs, scan_common_subdirs};
pub use find_program_files::{scan_program_files, scan_program_subdirs};
pub use find_uninstall::find_install_location;
pub use find_where::find_via_where;
