//! Maven 冲突清理
//!
//! 安装新版 Maven 前，清理旧版本残留：
//! 1. 删除旧 Maven 安装目录
//! 2. 移除系统 PATH 中所有指向旧 Maven 的条目（包含 mvn.cmd 或 mvn.bat 的目录）
//! 3. 清除旧的 MAVEN_HOME 和 M2_HOME 环境变量
//!
//! Maven 3.x 使用 `mvn.cmd` 启动脚本，旧版 Maven 2.x 使用 `mvn.bat`，
//! 两者都需要清理以确保新版本生效。

use super::{file_cleaner, path_cleaner, path_scanner};
use crate::install::emit_status;
use crate::system::env_config;
use tauri::AppHandle;

/// 执行 Maven 安装前的冲突清理。
///
/// 安全流程：先扫描 PATH → 删除旧文件 → 移除已采集的 PATH 条目 → 环境变量重置。
pub fn cleanup(app: &AppHandle) -> Result<(), String> {
    emit_status(app, "maven", "config", "正在清理旧版 Maven...");

    // 第 1 步：先扫描 PATH，保存所有包含 mvn.cmd/mvn.bat 的目录条目
    let mut path_entries = path_scanner::find_path_entries_for_exe("mvn.cmd");
    path_entries.extend(path_scanner::find_path_entries_for_exe("mvn.bat"));

    // 第 2 步：删除旧 Maven 安装目录
    delete_old_files(app);

    // 第 3 步：移除已采集的 PATH 条目
    path_cleaner::remove_entries(&path_entries, None);

    // 第 4 步：清除旧环境变量
    env_config::remove_env("MAVEN_HOME");
    env_config::remove_env("M2_HOME");

    emit_status(app, "maven", "config", "旧版 Maven 清理完成");
    Ok(())
}

/// 删除旧 Maven 的安装文件。
///
/// 定位策略：
/// 1. 从 MAVEN_HOME 或 M2_HOME 环境变量读取安装目录
/// 2. 从 PATH 中 mvn.cmd 的位置反推安装根目录（向上 1 级，bin → root）
fn delete_old_files(app: &AppHandle) {
    if let Some(maven_home) = path_scanner::read_env_var("MAVEN_HOME") {
        emit_status(app, "maven", "config", &format!("正在删除旧 Maven 目录: {maven_home}"));
        file_cleaner::remove_dir_safe(&maven_home);
    }

    if let Some(m2_home) = path_scanner::read_env_var("M2_HOME") {
        emit_status(app, "maven", "config", &format!("正在删除旧 Maven 目录: {m2_home}"));
        file_cleaner::remove_dir_safe(&m2_home);
    }

    // mvn.cmd 在 bin 下，向上 1 级得到 Maven 根目录
    for root in path_scanner::find_install_roots_from_path("mvn.cmd", 1) {
        emit_status(app, "maven", "config", &format!("正在删除旧 Maven 目录: {root}"));
        file_cleaner::remove_dir_safe(&root);
    }
}
