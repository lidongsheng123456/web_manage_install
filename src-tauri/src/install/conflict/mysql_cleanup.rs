//! MySQL 冲突清理
//!
//! 安装新版 MySQL 前，清理旧版本残留：
//! 1. 移除系统 PATH 中所有指向旧 MySQL 的条目（包含 mysql.exe 的目录）
//! 2. 清除旧的 MYSQL_HOME 环境变量
//!
//! 注意：MySQL 服务的停止和删除已由 `install::mysql::service::cleanup_old_service()`
//! 在安装流程中处理，此处不重复操作，仅负责 PATH 和环境变量层面的清理。

use super::path_sanitizer;
use crate::install::emit_status;
use crate::system::env_config;
use tauri::AppHandle;

/// 执行 MySQL 安装前的冲突清理。
///
/// 仅处理 PATH 净化和环境变量重置；
/// 服务层面的清理由 `mysql::service::cleanup_old_service()` 单独负责。
pub fn cleanup(app: &AppHandle) -> Result<(), String> {
    emit_status(app, "mysql", "config", "正在清理旧版 MySQL 环境变量...");

    // 移除 PATH 中所有包含 mysql.exe 的目录条目
    path_sanitizer::remove_path_entries_for_exe("mysql.exe", None);

    // 清除旧的 MYSQL_HOME，后续安装步骤会重新设置
    env_config::remove_env("MYSQL_HOME");

    emit_status(app, "mysql", "config", "旧版 MySQL 环境清理完成");
    Ok(())
}
