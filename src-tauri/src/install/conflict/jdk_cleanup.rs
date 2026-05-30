//! JDK 冲突清理
//!
//! 安装新版 JDK 前，清理旧版本残留：
//! 1. 删除旧 JDK 安装目录
//! 2. 移除系统 PATH 中所有指向旧 JDK 的条目（包含 java.exe 的目录）
//! 3. 移除 Oracle 安装器创建的 javapath 快捷方式目录
//! 4. 清除旧的 JAVA_HOME 环境变量
//!
//! Oracle JDK 安装器会在 `C:\Program Files\Common Files\Oracle\Java\javapath`
//! 创建 java.exe/javac.exe 的快捷方式并加入 PATH，该目录优先级通常很高，
//! 必须移除才能让新安装的 JDK 生效。

use super::{file_cleaner, path_cleaner, path_scanner};
use crate::install::emit_status;
use crate::system::env_config;
use tauri::AppHandle;

/// 执行 JDK 安装前的冲突清理。
///
/// 安全流程：先扫描 PATH → 删除旧文件 → 移除已采集的 PATH 条目 → 环境变量重置。
/// 必须先扫描再删文件，否则文件删除后 exe 不存在，PATH 扫描器无法定位旧条目。
pub fn cleanup(app: &AppHandle) -> Result<(), String> {
    emit_status(app, "jdk", "config", "正在清理旧版 JDK...");

    // 第 1 步：先扫描 PATH，保存所有包含 java.exe 的目录条目
    let path_entries = path_scanner::find_path_entries_for_exe("java.exe");

    // 第 2 步：删除旧 JDK 安装目录
    delete_old_files(app);

    // 第 3 步：移除已采集的 PATH 条目（文件已删除，用保存的列表）
    path_cleaner::remove_entries(&path_entries, None);

    // 第 4 步：移除 Oracle JDK 安装器创建的 javapath 快捷方式 PATH 条目
    path_cleaner::remove_entries_containing("Oracle\\Java\\javapath");

    // 第 5 步：清除旧的 JAVA_HOME，后续安装步骤会重新设置
    env_config::remove_env("JAVA_HOME");

    emit_status(app, "jdk", "config", "旧版 JDK 清理完成");
    Ok(())
}

/// 删除旧 JDK 的安装文件。
///
/// 定位策略：
/// 1. 从 JAVA_HOME 环境变量读取安装目录
/// 2. 从 PATH 中 java.exe 的位置反推安装根目录（向上 1 级，bin → root）
///
/// 安全流程：先终止从旧目录启动的 java/javaw 进程（释放文件锁），再删除目录。
fn delete_old_files(app: &AppHandle) {
    let mut roots = Vec::new();

    if let Some(java_home) = path_scanner::read_env_var("JAVA_HOME") {
        roots.push(java_home);
    }
    roots.extend(path_scanner::find_install_roots_from_path("java.exe", 1));

    // 先终止旧目录下的 Java 进程，防止文件被锁
    for root in &roots {
        emit_status(
            app,
            "jdk",
            "config",
            &format!("正在终止旧 JDK 目录中的 Java 进程: {root}"),
        );
        file_cleaner::kill_processes_from_dir(root);
    }

    for root in &roots {
        emit_status(app, "jdk", "config", &format!("正在删除旧 JDK 目录: {root}"));
        file_cleaner::remove_dir_safe(root);
    }
}
