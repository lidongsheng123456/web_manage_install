/// 用正则从文本中提取第一个捕获组。
pub fn extract_ver(text: &str, pattern: &str) -> String {
    regex_lite::Regex::new(pattern)
        .ok()
        .and_then(|r| r.captures(text))
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_default()
}
