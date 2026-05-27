//! Maven 环境检测
//!
//! 检测策略（按优先级）：
//! 1. 使用最新注册表 PATH 执行 `mvn -v`
//! 2. 读取注册表中 MAVEN_HOME 后直接执行
//! 3. 扫描常见安装目录
//! 4. 通过 where 命令查找

use super::env_reader::{extract_ver, read_fresh_env_vars, run_cmd_fresh, try_exe_at};
use crate::types::ComponentStatus;

/// 常见 Maven 安装路径（cmd 脚本和 bat 都尝试）
const COMMON_PATHS: &[&str] = &[
    r"D:\develop\software\maven\bin\mvn.cmd",
    r"D:\develop\software\maven\bin\mvn.bat",
    r"C:\Program Files\Maven\bin\mvn.cmd",
    r"C:\Program Files\apache-maven\bin\mvn.cmd",
    r"D:\develop\software\apache-maven-3.9.6\bin\mvn.cmd",
    r"D:\develop\software\apache-maven-3.9.9\bin\mvn.cmd",
];

pub fn detect(expected: &str) -> ComponentStatus {
    // 1) PATH 执行 mvn -v（使用实时注册表 PATH）
    if let Some(ver) = detect_from_output(run_cmd_fresh("mvn", &["-v"])) {
        return build_status(ver, expected);
    }

    // 2) 从注册表 MAVEN_HOME 找 mvn.cmd
    for (k, v) in read_fresh_env_vars() {
        if k == "MAVEN_HOME" {
            let cmd_path = format!(r"{}\bin\mvn.cmd", v.trim_end_matches('\\'));
            if let Some(ver) = detect_from_output(try_exe_at(&cmd_path, &["-v"])) {
                return build_status(ver, expected);
            }
            let bat_path = format!(r"{}\bin\mvn.bat", v.trim_end_matches('\\'));
            if let Some(ver) = detect_from_output(try_exe_at(&bat_path, &["-v"])) {
                return build_status(ver, expected);
            }
        }
    }

    // 3) 扫描常见目录
    for path in COMMON_PATHS {
        if let Some(ver) = detect_from_output(try_exe_at(path, &["-v"])) {
            return build_status(ver, expected);
        }
    }

    // 4) 通过 where 命令查找
    if let Some(output) = run_cmd_fresh("where", &["mvn"]) {
        let first_line = output.lines().next().unwrap_or("").trim();
        if !first_line.is_empty() && std::path::Path::new(first_line).exists() {
            if let Some(ver) = detect_from_output(try_exe_at(first_line, &["-v"])) {
                return build_status(ver, expected);
            }
        }
    }

    ComponentStatus {
        name: "Maven".into(),
        installed: false,
        version: String::new(),
        expected_version: expected.into(),
        version_match: false,
    }
}

/// 从 `mvn -v` 输出中提取版本号
fn detect_from_output(output: Option<String>) -> Option<String> {
    let text = output?;
    let ver = extract_ver(&text, r"Apache Maven (\d+\.\d+\.\d+)");
    if ver.is_empty() { None } else { Some(ver) }
}

fn build_status(ver: String, expected: &str) -> ComponentStatus {
    ComponentStatus {
        name: "Maven".into(),
        installed: true,
        version_match: ver == expected,
        version: ver,
        expected_version: expected.into(),
    }
}
