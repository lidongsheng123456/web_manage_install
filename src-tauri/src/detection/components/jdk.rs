//! JDK 环境检测
//!
//! 检测策略（按优先级）：
//! 1. 使用最新注册表 PATH 执行 `java -version`
//! 2. 从注册表 JAVA_HOME 定位
//! 3. 通过 `where java` 搜索 PATH
//! 4. 查询 App Paths 注册表
//! 5. 查询 Uninstall 注册表获取安装位置
//! 6. 扫描 Program Files\Java 下的 jdk-* 子目录
//! 7. 扫描常见安装目录

use crate::common::types::ComponentStatus;
use crate::detection::env::*;

pub fn detect(expected_major: &str) -> ComponentStatus {
    let expected_label = format!("JDK {expected_major}");

    // 1) PATH 执行 java -version
    if let Some(ver) = parse_java_output(run_cmd_fresh("java", &["-version"])) {
        return status(ver, expected_major, &expected_label);
    }

    // 2) 从注册表 JAVA_HOME 找 java.exe
    for (k, v) in read_fresh_env_vars() {
        if k == "JAVA_HOME" {
            let exe = format!(r"{}\bin\java.exe", v.trim_end_matches('\\'));
            if let Some(ver) = parse_java_output(try_exe_at(&exe, &["-version"])) {
                return status(ver, expected_major, &expected_label);
            }
        }
    }

    // 3) where 命令搜索
    for path in find_via_where("java") {
        if let Some(ver) = parse_java_output(try_exe_at(&path, &["-version"])) {
            return status(ver, expected_major, &expected_label);
        }
    }

    // 4) App Paths 注册表
    if let Some(path) = check_app_paths("java.exe") {
        if let Some(ver) = parse_java_output(try_exe_at(&path, &["-version"])) {
            return status(ver, expected_major, &expected_label);
        }
    }

    // 5) Uninstall 注册表
    for loc in find_install_location("Java") {
        for suffix in [r"bin\java.exe", r"jre\bin\java.exe"] {
            let exe = format!(r"{}\{}", loc.trim_end_matches('\\'), suffix);
            if let Some(ver) = parse_java_output(try_exe_at(&exe, &["-version"])) {
                return status(ver, expected_major, &expected_label);
            }
        }
    }

    // 6) 扫描 Program Files\Java 下的 jdk-* 子目录
    for path in scan_program_subdirs("Java", "jdk", r"bin\java.exe") {
        if let Some(ver) = parse_java_output(try_exe_at(&path, &["-version"])) {
            return status(ver, expected_major, &expected_label);
        }
    }

    // 7) 扫描常见安装目录中的 jdk* 目录
    for path in scan_common_subdirs("", "jdk", r"bin\java.exe") {
        if let Some(ver) = parse_java_output(try_exe_at(&path, &["-version"])) {
            return status(ver, expected_major, &expected_label);
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

fn parse_java_output(output: Option<String>) -> Option<String> {
    let text = output?;
    let ver = extract_ver(&text, r#"(?:version\s+")(\d+[\.\d]*)"#);
    if ver.is_empty() {
        None
    } else {
        Some(ver)
    }
}

fn status(ver: String, expected_major: &str, expected_label: &str) -> ComponentStatus {
    let major = ver.split('.').next().unwrap_or("");
    ComponentStatus {
        name: "JDK".into(),
        installed: true,
        version_match: major == expected_major,
        version: ver,
        expected_version: expected_label.into(),
    }
}
