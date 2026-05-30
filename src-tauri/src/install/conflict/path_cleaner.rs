//! PATH 清理器
//!
//! 从注册表系统 PATH 中移除指定条目。
//! 仅负责"移除 PATH 条目"，不做扫描和文件系统操作。

use crate::detection::env::build_fresh_path;
use crate::system::env_config;

/// 将预先采集的 PATH 条目列表从系统 PATH 中移除。
///
/// 用于"先扫描 → 后删文件 → 最后移除 PATH"的安全流程：
/// 文件删除后 `find_path_entries_for_exe` 无法再探测到已删除的 exe，
/// 因此必须在删除前保存条目列表，删除后用此函数清理。
pub fn remove_entries(entries: &[String], exclude_dir: Option<&str>) {
    for entry in entries {
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
pub fn remove_entries_containing(substring: &str) {
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
