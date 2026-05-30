//! PATH 扫描器
//!
//! 从注册表实时读取系统 PATH，扫描其中包含指定可执行文件的目录条目。
//! 仅负责"查找"，不做任何修改操作。

use crate::detection::env::{build_fresh_path, read_fresh_env_vars};
use std::path::Path;

/// 扫描系统 PATH 中所有包含指定可执行文件的目录条目。
///
/// 从注册表实时读取 HKLM + HKCU 的 PATH，按 `;` 分割后逐一检查
/// 每个目录下是否存在目标可执行文件。
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

/// 根据可执行文件在 PATH 中的位置，推断其安装根目录。
///
/// 例如 `java.exe` 位于 `D:\jdk17\bin\java.exe`，
/// 则传入 `("java.exe", 1)` 返回 `D:\jdk17`（向上 1 级）。
/// 传入 `("node.exe", 0)` 返回 `D:\nodejs`（当前目录）。
pub fn find_install_roots_from_path(exe_name: &str, parent_levels: usize) -> Vec<String> {
    find_path_entries_for_exe(exe_name)
        .into_iter()
        .filter_map(|entry| {
            let mut path = Path::new(&entry).to_path_buf();
            for _ in 0..parent_levels {
                path = path.parent()?.to_path_buf();
            }
            let dir = path.to_string_lossy().to_string();
            if Path::new(&dir).is_dir() && dir.len() > 10 {
                Some(dir)
            } else {
                None
            }
        })
        .collect()
}

/// 从注册表读取指定环境变量的值。
///
/// 用于获取 JAVA_HOME、MAVEN_HOME 等变量指向的旧安装目录。
pub fn read_env_var(key: &str) -> Option<String> {
    read_fresh_env_vars()
        .into_iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
}
