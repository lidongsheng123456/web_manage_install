use std::path::Path;

/// 在 Program Files 目录中按相对路径模式搜索可执行文件。
pub fn scan_program_files(relative_paths: &[&str]) -> Vec<String> {
    let mut results = Vec::new();
    for base in program_files_roots() {
        append_existing_relative_paths(&mut results, &base, relative_paths);
    }
    results
}

/// 在 Program Files 下扫描匹配前缀的子目录，查找目标可执行文件。
pub fn scan_program_subdirs(parent_dir: &str, dir_prefix: &str, exe_suffix: &str) -> Vec<String> {
    let mut results = Vec::new();
    for base in program_files_roots() {
        let search_dir = format!("{}\\{}", base, parent_dir);
        results.extend(scan_dir_with_prefix(&search_dir, dir_prefix, exe_suffix));
    }
    results
}

pub fn append_existing_relative_paths(
    results: &mut Vec<String>,
    base: &str,
    relative_paths: &[&str],
) {
    for rel in relative_paths {
        let full = format!("{}\\{}", base, rel);
        if Path::new(&full).exists() && !results.contains(&full) {
            results.push(full);
        }
    }
}

pub fn scan_dir_with_prefix(parent: &str, dir_prefix: &str, exe_suffix: &str) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(parent) else {
        return Vec::new();
    };
    let prefix_lower = dir_prefix.to_lowercase();
    entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().map(|t| t.is_dir()).unwrap_or(false)
                && e.file_name()
                    .to_string_lossy()
                    .to_lowercase()
                    .starts_with(&prefix_lower)
        })
        .filter_map(|e| {
            let full = format!("{}\\{}", e.path().to_string_lossy(), exe_suffix);
            if Path::new(&full).exists() {
                Some(full)
            } else {
                None
            }
        })
        .collect()
}

fn program_files_roots() -> [String; 2] {
    [
        std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into()),
        std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".into()),
    ]
}
