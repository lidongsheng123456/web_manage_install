//! PATH 净化器
//!
//! 提供通用的 PATH 扫描和清理能力：根据可执行文件名在系统 PATH 中
//! 查找包含该文件的目录条目，并将其从注册表 PATH 中移除。
//! 确保安装新版本前，旧版本的 PATH 条目不会干扰新版本。

use crate::detection::env::build_fresh_path;
use crate::system::env_config;
use std::path::Path;

/// 扫描系统 PATH 中所有包含指定可执行文件的目录条目。
///
/// 从注册表实时读取 HKLM + HKCU 的 PATH，按 `;` 分割后逐一检查
/// 每个目录下是否存在目标可执行文件。
///
/// # 示例
/// ```ignore
/// // 查找 PATH 中所有包含 node.exe 的目录
/// let entries = find_path_entries_for_exe("node.exe");
/// // 可能返回 ["C:\\Program Files\\nodejs", "D:\\DevSetup\\nodejs"]
/// ```
pub fn find_path_entries_for_exe(exe_name: &str) -> Vec<String> {
    let fresh_path = build_fresh_path();
    fresh_path
        .split(';')
        .filter(|entry| !entry.trim().is_empty())
        .filter(|entry| {
            let full = format!("{}\\{}", entry.trim_end_matches('\\'), exe_name);
            Path::new(&full).exists()
        })
        .map(|s| s.to_string())
        .collect()
}

/// 从系统 PATH 中移除所有包含指定可执行文件的目录条目。
///
/// 可选参数 `exclude_dir` 用于排除即将安装的目标目录，
/// 避免误删新安装路径（当新版本已预先写入目录时）。
///
/// 内部调用 `env_config::remove_from_path()` 逐条移除，
/// 并在完成后广播环境变更通知。
pub fn remove_path_entries_for_exe(exe_name: &str, exclude_dir: Option<&str>) {
    let entries = find_path_entries_for_exe(exe_name);
    for entry in &entries {
        if let Some(exclude) = exclude_dir {
            let entry_lower = entry.trim_end_matches('\\').to_lowercase();
            let exclude_lower = exclude.trim_end_matches('\\').to_lowercase();
            if entry_lower == exclude_lower {
                continue;
            }
        }
        env_config::remove_from_path(entry);
    }
}

/// 从系统 PATH 中移除包含指定子串的所有条目（大小写不敏感）。
///
/// 用于清理特定目录模式，例如移除所有包含 "Oracle\\Java\\javapath" 的条目。
pub fn remove_path_entries_containing(substring: &str) {
    let fresh_path = build_fresh_path();
    let needle = substring.to_lowercase();
    let entries: Vec<String> = fresh_path
        .split(';')
        .filter(|entry| !entry.trim().is_empty())
        .filter(|entry| entry.to_lowercase().contains(&needle))
        .map(|s| s.to_string())
        .collect();

    for entry in &entries {
        env_config::remove_from_path(entry);
    }
}
