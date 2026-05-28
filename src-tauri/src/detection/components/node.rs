//! Node.js 环境检测
//!
//! 检测策略（按优先级）：
//! 1. 使用最新注册表 PATH 执行 `node --version`
//! 2. 从 NODE_HOME 环境变量定位
//! 3. 通过 `where node` 搜索 PATH
//! 4. 查询 App Paths 注册表
//! 5. 查询 Uninstall 注册表获取安装位置
//! 6. 扫描 Program Files 和常见安装目录

use crate::common::types::ComponentStatus;
use crate::detection::env::*;

pub fn detect(expected: &str) -> ComponentStatus {
    // 1) PATH 中直接执行
    if let Some(ver) = try_version(run_cmd_fresh("node", &["--version"])) {
        return status(ver, expected);
    }

    // 2) NODE_HOME 环境变量
    for (k, v) in read_fresh_env_vars() {
        if k == "NODE_HOME" {
            let exe = format!(r"{}\node.exe", v.trim_end_matches('\\'));
            if let Some(ver) = try_version(try_exe_at(&exe, &["--version"])) {
                return status(ver, expected);
            }
        }
    }

    // 3) where 命令搜索
    for path in find_via_where("node") {
        if let Some(ver) = try_version(try_exe_at(&path, &["--version"])) {
            return status(ver, expected);
        }
    }

    // 4) App Paths 注册表
    if let Some(path) = check_app_paths("node.exe") {
        if let Some(ver) = try_version(try_exe_at(&path, &["--version"])) {
            return status(ver, expected);
        }
    }

    // 5) Uninstall 注册表
    for loc in find_install_location("Node.js") {
        let exe = format!(r"{}\node.exe", loc.trim_end_matches('\\'));
        if let Some(ver) = try_version(try_exe_at(&exe, &["--version"])) {
            return status(ver, expected);
        }
    }

    // 6) 扫描 Program Files
    for path in scan_program_files(&[r"nodejs\node.exe"]) {
        if let Some(ver) = try_version(try_exe_at(&path, &["--version"])) {
            return status(ver, expected);
        }
    }

    // 7) 扫描常见安装目录
    for path in scan_common_install_dirs(&[r"nodejs\node.exe"]) {
        if let Some(ver) = try_version(try_exe_at(&path, &["--version"])) {
            return status(ver, expected);
        }
    }

    ComponentStatus {
        name: "Node.js".into(),
        installed: false,
        version: String::new(),
        expected_version: expected.into(),
        version_match: false,
    }
}

fn try_version(output: Option<String>) -> Option<String> {
    let text = output?;
    let ver = extract_ver(&text, r"v?(\d+\.\d+\.\d+)");
    if ver.is_empty() {
        None
    } else {
        Some(ver)
    }
}

fn status(ver: String, expected: &str) -> ComponentStatus {
    ComponentStatus {
        name: "Node.js".into(),
        installed: true,
        version_match: ver == expected,
        version: ver,
        expected_version: expected.into(),
    }
}
