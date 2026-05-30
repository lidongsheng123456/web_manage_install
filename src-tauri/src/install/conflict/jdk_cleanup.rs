//! JDK 冲突清理
//!
//! 安装新版 JDK 前，清理旧版本残留：
//! 1. 移除系统 PATH 中所有指向旧 JDK 的条目（包含 java.exe 的目录）
//! 2. 移除 Oracle 安装器创建的 javapath 快捷方式目录
//! 3. 清除旧的 JAVA_HOME 环境变量
//!
//! Oracle JDK 安装器会在 `C:\Program Files\Common Files\Oracle\Java\javapath`
//! 创建 java.exe/javac.exe 的快捷方式并加入 PATH，该目录优先级通常很高，
//! 必须移除才能让新安装的 JDK 生效。

use super::path_sanitizer;
use crate::install::emit_status;
use crate::system::env_config;
use tauri::AppHandle;

/// 执行 JDK 安装前的冲突清理。
///
/// 依次完成：PATH 净化 → Oracle javapath 移除 → 环境变量重置。
pub fn cleanup(app: &AppHandle) -> Result<(), String> {
    emit_status(app, "jdk", "config", "正在清理旧版 JDK 环境变量...");

    // 移除 PATH 中所有包含 java.exe 的目录条目
    path_sanitizer::remove_path_entries_for_exe("java.exe", None);

    // 移除 Oracle JDK 安装器创建的 javapath 快捷方式目录
    // 该目录通常位于 C:\Program Files\Common Files\Oracle\Java\javapath
    // 或 C:\Program Files (x86)\Common Files\Oracle\Java\javapath
    remove_oracle_javapath();

    // 清除旧的 JAVA_HOME，后续安装步骤会重新设置
    env_config::remove_env("JAVA_HOME");

    emit_status(app, "jdk", "config", "旧版 JDK 环境清理完成");
    Ok(())
}

/// 从系统 PATH 中移除 Oracle JDK 安装器创建的 javapath 目录。
///
/// Oracle JDK 安装器会将以下路径加入 PATH（优先级通常排在最前面）：
/// - `C:\Program Files\Common Files\Oracle\Java\javapath`
/// - `C:\Program Files (x86)\Common Files\Oracle\Java\javapath`
///
/// 这些目录包含 java.exe 等文件的快捷方式，会覆盖用户新安装的 JDK。
fn remove_oracle_javapath() {
    path_sanitizer::remove_path_entries_containing("Oracle\\Java\\javapath");
}
