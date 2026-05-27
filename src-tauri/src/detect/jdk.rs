//! JDK 环境检测
//!
//! 检测策略（按优先级）：
//! 1. 使用最新注册表 PATH 执行 `java -version`
//! 2. 读取注册表中 JAVA_HOME 后直接执行
//! 3. 扫描常见安装目录

use super::env_reader::{extract_ver, read_fresh_env_vars, run_cmd_fresh, try_exe_at};
use crate::types::ComponentStatus;

/// 常见 JDK 安装路径
const COMMON_PATHS: &[&str] = &[
    r"C:\Program Files\Java\jdk-17\bin\java.exe",
    r"C:\Program Files\Java\jdk-21\bin\java.exe",
    r"D:\develop\software\jdk17\bin\java.exe",
    r"D:\develop\software\jdk21\bin\java.exe",
    r"D:\jdks\ms-17\bin\java.exe",
    r"D:\jdks\ms-21.0.9\bin\java.exe",
];

pub fn detect(expected_major: &str) -> ComponentStatus {
    let expected_label = format!("JDK {expected_major}");

    // 1) PATH 执行 java -version
    if let Some(ver) = detect_from_output(run_cmd_fresh("java", &["-version"])) {
        let major = ver.split('.').next().unwrap_or("");
        return ComponentStatus {
            name: "JDK".into(),
            installed: true,
            version_match: major == expected_major,
            version: ver,
            expected_version: expected_label,
        };
    }

    // 2) 从注册表 JAVA_HOME 找 java.exe
    for (k, v) in read_fresh_env_vars() {
        if k == "JAVA_HOME" {
            let exe = format!(r"{}\bin\java.exe", v.trim_end_matches('\\'));
            if let Some(ver) = detect_from_output(try_exe_at(&exe, &["-version"])) {
                let major = ver.split('.').next().unwrap_or("");
                return ComponentStatus {
                    name: "JDK".into(),
                    installed: true,
                    version_match: major == expected_major,
                    version: ver,
                    expected_version: expected_label,
                };
            }
        }
    }

    // 3) 扫描常见目录
    for path in COMMON_PATHS {
        if let Some(ver) = detect_from_output(try_exe_at(path, &["-version"])) {
            let major = ver.split('.').next().unwrap_or("");
            return ComponentStatus {
                name: "JDK".into(),
                installed: true,
                version_match: major == expected_major,
                version: ver,
                expected_version: expected_label,
            };
        }
    }

    ComponentStatus {
        name: "JDK".into(),
        installed: false,
        version: String::new(),
        expected_version: expected_label,
        version_match: false,
    }
}

/// 从 `java -version` 的输出中提取版本号
fn detect_from_output(output: Option<String>) -> Option<String> {
    let text = output?;
    let ver = extract_ver(&text, r#"(?:version\s+")(\d+[\.\d]*)"#);
    if ver.is_empty() { None } else { Some(ver) }
}
