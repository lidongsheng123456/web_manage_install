//! Maven 环境检测
//!
//! 检测策略（按优先级）：
//! 1. 使用最新注册表 PATH 执行 `mvn -v`
//! 2. 从注册表 MAVEN_HOME 定位
//! 3. 通过 `where mvn` 搜索 PATH
//! 4. 查询 Uninstall 注册表获取安装位置
//! 5. 扫描 Program Files 和常见安装目录中的 apache-maven-* / maven 目录

use crate::common::types::ComponentStatus;
use crate::detection::env::*;

pub fn detect(expected: &str) -> ComponentStatus {
    // 1) PATH 执行 mvn -v
    if let Some(ver) = parse_mvn_output(run_cmd_fresh("mvn", &["-v"])) {
        return status(ver, expected);
    }

    // 2) 从注册表 MAVEN_HOME 找 mvn.cmd
    for (k, v) in read_fresh_env_vars() {
        if k == "MAVEN_HOME" {
            let base = v.trim_end_matches('\\');
            for ext in ["mvn.cmd", "mvn.bat", "mvn"] {
                let path = format!(r"{}\bin\{}", base, ext);
                if let Some(ver) = parse_mvn_output(try_exe_at(&path, &["-v"])) {
                    return status(ver, expected);
                }
            }
        }
    }

    // 3) where 命令搜索
    for path in find_via_where("mvn") {
        if let Some(ver) = parse_mvn_output(try_exe_at(&path, &["-v"])) {
            return status(ver, expected);
        }
    }
    for path in find_via_where("mvn.cmd") {
        if let Some(ver) = parse_mvn_output(try_exe_at(&path, &["-v"])) {
            return status(ver, expected);
        }
    }

    // 4) Uninstall 注册表
    for loc in find_install_location("Maven") {
        for ext in ["mvn.cmd", "mvn.bat"] {
            let path = format!(r"{}\bin\{}", loc.trim_end_matches('\\'), ext);
            if let Some(ver) = parse_mvn_output(try_exe_at(&path, &["-v"])) {
                return status(ver, expected);
            }
        }
    }

    // 5) 扫描 Program Files 中的 Maven 目录
    for path in scan_program_files(&[r"Maven\bin\mvn.cmd", r"apache-maven\bin\mvn.cmd"]) {
        if let Some(ver) = parse_mvn_output(try_exe_at(&path, &["-v"])) {
            return status(ver, expected);
        }
    }
    for path in scan_program_subdirs("", "apache-maven", r"bin\mvn.cmd") {
        if let Some(ver) = parse_mvn_output(try_exe_at(&path, &["-v"])) {
            return status(ver, expected);
        }
    }

    // 6) 扫描常见安装目录
    for path in scan_common_install_dirs(&[r"maven\bin\mvn.cmd", r"maven\bin\mvn.bat"]) {
        if let Some(ver) = parse_mvn_output(try_exe_at(&path, &["-v"])) {
            return status(ver, expected);
        }
    }
    for path in scan_common_subdirs("", "apache-maven", r"bin\mvn.cmd") {
        if let Some(ver) = parse_mvn_output(try_exe_at(&path, &["-v"])) {
            return status(ver, expected);
        }
    }
    for path in scan_common_subdirs("", "maven", r"bin\mvn.cmd") {
        if let Some(ver) = parse_mvn_output(try_exe_at(&path, &["-v"])) {
            return status(ver, expected);
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

fn parse_mvn_output(output: Option<String>) -> Option<String> {
    let text = output?;
    let ver = extract_ver(&text, r"Apache Maven (\d+\.\d+\.\d+)");
    if ver.is_empty() {
        None
    } else {
        Some(ver)
    }
}

fn status(ver: String, expected: &str) -> ComponentStatus {
    ComponentStatus {
        name: "Maven".into(),
        installed: true,
        version_match: ver == expected,
        version: ver,
        expected_version: expected.into(),
    }
}
