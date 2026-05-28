/// 检查 PATH 字符串中是否已包含目标路径（大小写不敏感）。
pub fn path_contains(path_str: &str, normalized_entry: &str) -> bool {
    path_str
        .split(';')
        .filter(|s| !s.is_empty())
        .any(|e| e.trim_end_matches('\\').to_lowercase() == *normalized_entry)
}

/// 向 PATH 字符串末尾追加条目。
pub fn append_entry(current: &str, new_entry: &str) -> String {
    if current.ends_with(';') || current.is_empty() {
        format!("{current}{new_entry}")
    } else {
        format!("{current};{new_entry}")
    }
}

/// 从 PATH 字符串中移除匹配的条目。
pub fn remove_entry(path_str: &str, normalized_entry: &str) -> String {
    path_str
        .split(';')
        .filter(|s| !s.is_empty())
        .filter(|s| s.trim_end_matches('\\').to_lowercase() != *normalized_entry)
        .collect::<Vec<_>>()
        .join(";")
}
