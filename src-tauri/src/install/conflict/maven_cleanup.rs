//! Maven 冲突清理
//!
//! 安装新版 Maven 前，清理旧版本残留：
//! 1. 移除系统 PATH 中所有指向旧 Maven 的条目（包含 mvn.cmd 或 mvn.bat 的目录）
//! 2. 清除旧的 MAVEN_HOME 和 M2_HOME 环境变量
//!
//! Maven 3.x 使用 `mvn.cmd` 启动脚本，旧版 Maven 2.x 使用 `mvn.bat`，
//! 两者都需要清理以确保新版本生效。

use super::path_sanitizer;
use crate::install::emit_status;
use crate::system::env_config;
use tauri::AppHandle;

/// 执行 Maven 安装前的冲突清理。
///
/// 依次完成：PATH 净化（mvn.cmd + mvn.bat）→ 环境变量重置。
pub fn cleanup(app: &AppHandle) -> Result<(), String> {
    emit_status(app, "maven", "config", "正在清理旧版 Maven 环境变量...");

    // 移除 PATH 中所有包含 mvn.cmd 的目录（Maven 3.x）
    path_sanitizer::remove_path_entries_for_exe("mvn.cmd", None);
    // 兼容旧版 Maven 2.x 使用的 mvn.bat
    path_sanitizer::remove_path_entries_for_exe("mvn.bat", None);

    // 清除旧的 MAVEN_HOME，后续安装步骤会重新设置
    env_config::remove_env("MAVEN_HOME");
    // 兼容清理 M2_HOME（Maven 2.x 时代的变量名，部分用户仍在使用）
    env_config::remove_env("M2_HOME");

    emit_status(app, "maven", "config", "旧版 Maven 环境清理完成");
    Ok(())
}
