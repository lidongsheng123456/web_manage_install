use super::find_program_files::{append_existing_relative_paths, scan_dir_with_prefix};
use std::path::Path;

/// 在可用磁盘驱动器的常见开发目录中搜索可执行文件。
pub fn scan_common_install_dirs(relative_paths: &[&str]) -> Vec<String> {
    let mut results = Vec::new();
    for drive in get_local_drives() {
        for root in common_roots() {
            let base = format!("{}\\{}", drive, root);
            if !Path::new(&base).is_dir() {
                continue;
            }
            append_existing_relative_paths(&mut results, &base, relative_paths);
        }
        append_existing_relative_paths(&mut results, &drive, relative_paths);
    }
    results
}

/// 在可用磁盘驱动器的常见目录中扫描匹配前缀的子目录。
pub fn scan_common_subdirs(dir_name: &str, dir_prefix: &str, exe_suffix: &str) -> Vec<String> {
    let mut results = Vec::new();
    for drive in get_local_drives() {
        for root in common_roots() {
            let search = format!("{}\\{}\\{}", drive, root, dir_name);
            results.extend(scan_dir_with_prefix(&search, dir_prefix, exe_suffix));
        }
        let search = format!("{}\\{}", drive, dir_name);
        results.extend(scan_dir_with_prefix(&search, dir_prefix, exe_suffix));
    }
    results
}

fn common_roots() -> &'static [&'static str] {
    &[
        "develop\\software",
        "software",
        "dev",
        "tools",
        "DevTools",
        "Programs",
    ]
}

fn get_local_drives() -> Vec<String> {
    ('C'..='H')
        .map(|c| format!("{}:", c))
        .filter(|d| Path::new(&format!("{}\\", d)).exists())
        .collect()
}
