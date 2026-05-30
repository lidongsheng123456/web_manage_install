//! MySQL 冲突清理
//!
//! 安装新版 MySQL 前，清理旧版本残留：
//! 1. 停止并删除旧 MySQL 服务（释放文件锁）
//! 2. 删除旧 MySQL 安装目录（保留 data 数据目录）
//! 3. 移除系统 PATH 中所有指向旧 MySQL 的条目（包含 mysql.exe 的目录）
//! 4. 清除旧的 MYSQL_HOME 环境变量
//!
//! 注意：必须先停止服务再删除文件，否则服务进程锁定文件会导致
//! "另一个程序正在使用此文件"（os error 32）错误。

use super::{file_cleaner, path_cleaner, path_scanner};
use crate::common::process::hide_window;
use crate::common::version_policy::mysql as mysql_policy;
use crate::install::emit_status;
use crate::system::env_config;
use std::process::Command;
use tauri::AppHandle;

/// 执行 MySQL 安装前的冲突清理。
///
/// 安全流程：停止服务 → 扫描 PATH → 删除旧文件 → 移除 PATH 条目 → 环境变量重置。
/// 必须先停止服务释放文件锁，才能成功删除旧安装目录中的文件。
pub fn cleanup(app: &AppHandle) -> Result<(), String> {
    emit_status(app, "mysql", "config", "正在清理旧版 MySQL...");

    // 第 1 步：停止并删除旧 MySQL 服务（释放文件锁）
    stop_and_remove_services(app);

    // 第 2 步：扫描 PATH，保存所有包含 mysql.exe 的目录条目
    let path_entries = path_scanner::find_path_entries_for_exe("mysql.exe");

    // 第 3 步：删除旧 MySQL 安装目录（保留 data 子目录防止数据丢失）
    delete_old_files(app);

    // 第 4 步：移除已采集的 PATH 条目
    path_cleaner::remove_entries(&path_entries, None);

    // 第 5 步：清除旧的 MYSQL_HOME
    env_config::remove_env("MYSQL_HOME");

    emit_status(app, "mysql", "config", "旧版 MySQL 清理完成");
    Ok(())
}

/// 停止并删除所有已知的 MySQL Windows 服务。
///
/// 遍历版本策略中定义的受管服务名（MySQL80、MySQL57 等），
/// 逐一执行 `sc stop` 和 `sc delete`。等待服务完全停止后再返回，
/// 确保文件锁释放。
fn stop_and_remove_services(app: &AppHandle) {
    emit_status(app, "mysql", "config", "正在停止旧 MySQL 服务...");
    for &svc in mysql_policy::MANAGED_SERVICE_NAMES {
        let _ = hide_window(Command::new("sc").args(["stop", svc])).output();
    }
    // 等待服务进程完全退出，释放文件锁
    std::thread::sleep(std::time::Duration::from_secs(3));
    for &svc in mysql_policy::MANAGED_SERVICE_NAMES {
        let _ = hide_window(Command::new("sc").args(["delete", svc])).output();
    }
    std::thread::sleep(std::time::Duration::from_secs(1));
}

/// 删除旧 MySQL 的安装文件（保留 data 数据目录）。
///
/// 定位策略：
/// 1. 从 MYSQL_HOME 环境变量读取安装目录
/// 2. 从 PATH 中 mysql.exe 的位置反推安装根目录（向上 1 级，bin → root）
///
/// 安全策略：使用 `file_cleaner::remove_dir_except` 跳过 `data` 子目录。
fn delete_old_files(app: &AppHandle) {
    let mut roots = Vec::new();

    if let Some(mysql_home) = path_scanner::read_env_var("MYSQL_HOME") {
        roots.push(mysql_home);
    }

    // mysql.exe 在 bin 下，向上 1 级得到 MySQL 根目录
    roots.extend(path_scanner::find_install_roots_from_path("mysql.exe", 1));

    for root in &roots {
        emit_status(
            app,
            "mysql",
            "config",
            &format!("正在清理旧 MySQL 目录: {root}（保留 data）"),
        );
        file_cleaner::remove_dir_except(root, &["data"]);
    }
}
