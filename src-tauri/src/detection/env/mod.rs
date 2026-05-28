//! 检测辅助能力门面
//!
//! 组件检测统一从这里引入环境变量、命令执行、查找器和版本解析工具。

mod command_runner;
mod env_vars;
mod version_parser;

pub use super::finder::{
    check_app_paths, find_install_location, find_via_where, scan_common_install_dirs,
    scan_common_subdirs, scan_program_files, scan_program_subdirs,
};
pub use command_runner::{run_cmd_fresh, try_exe_at};
pub use env_vars::{build_fresh_path, read_fresh_env_vars};
pub use version_parser::extract_ver;
