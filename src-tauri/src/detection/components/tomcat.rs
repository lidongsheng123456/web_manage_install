//! Tomcat 环境检测
//!
//! 检测策略（按优先级）：
//! 1. 从注册表 CATALINA_HOME 定位 version.sh/RELEASE-NOTES
//! 2. 扫描常见安装目录中的 tomcat 目录

use crate::common::types::ComponentStatus;
use crate::detection::env::*;
use regex_lite::Regex;

pub fn detect(expected: &str) -> ComponentStatus {
    for (k, v) in read_fresh_env_vars() {
        if k == "CATALINA_HOME" {
            if let Some(ver) = detect_from_dir(v.trim_end_matches('\\')) {
                return status(ver, expected);
            }
        }
    }

    for path in scan_common_subdirs("", "tomcat", "RELEASE-NOTES") {
        let dir = std::path::Path::new(&path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        if let Some(ver) = detect_from_dir(&dir) {
            return status(ver, expected);
        }
    }

    for path in scan_common_subdirs("", "apache-tomcat", "RELEASE-NOTES") {
        let dir = std::path::Path::new(&path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        if let Some(ver) = detect_from_dir(&dir) {
            return status(ver, expected);
        }
    }

    ComponentStatus {
        name: "Tomcat".into(),
        installed: false,
        version: String::new(),
        expected_version: expected.into(),
        version_match: false,
    }
}

fn detect_from_dir(dir: &str) -> Option<String> {
    let notes = format!(r"{}\RELEASE-NOTES", dir);
    let content = std::fs::read_to_string(notes).ok()?;
    parse_release_notes(&content)
}

fn parse_release_notes(text: &str) -> Option<String> {
    let re = Regex::new(r"Apache Tomcat Version (\d+\.\d+\.\d+)").ok()?;
    re.captures(text)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

fn status(ver: String, expected: &str) -> ComponentStatus {
    let expected_major = expected.split('.').next().unwrap_or("");
    let actual_major = ver.split('.').next().unwrap_or("");
    ComponentStatus {
        name: "Tomcat".into(),
        installed: true,
        version_match: ver == expected || actual_major == expected_major,
        version: ver,
        expected_version: expected.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_release_notes() {
        let text = "                  Apache Tomcat Version 9.0.102\n                            Release Notes";
        assert_eq!(parse_release_notes(text), Some("9.0.102".into()));
    }
}
