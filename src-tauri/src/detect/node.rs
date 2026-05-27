//! Node.js 环境检测
//!
//! 检测策略（按优先级）：
//! 1. 使用最新注册表 PATH 执行 `node --version`
//! 2. 扫描常见安装目录

use super::env_reader::{extract_ver, run_cmd_fresh, try_exe_at};
use crate::types::ComponentStatus;

/// 常见 Node.js 安装路径
const COMMON_PATHS: &[&str] = &[
    r"C:\Program Files\nodejs\node.exe",
    r"D:\develop\software\nodejs\node.exe",
    r"D:\nodejs\node.exe",
];

pub fn detect(expected: &str) -> ComponentStatus {
    // 1) PATH 中直接执行
    if let Some(output) = run_cmd_fresh("node", &["--version"]) {
        let ver = extract_ver(&output, r"v?(\d+\.\d+\.\d+)");
        if !ver.is_empty() {
            return ComponentStatus {
                name: "Node.js".into(),
                installed: true,
                version_match: ver == expected,
                version: ver,
                expected_version: expected.into(),
            };
        }
    }

    // 2) 扫描常见目录
    for path in COMMON_PATHS {
        if let Some(output) = try_exe_at(path, &["--version"]) {
            let ver = extract_ver(&output, r"v?(\d+\.\d+\.\d+)");
            if !ver.is_empty() {
                return ComponentStatus {
                    name: "Node.js".into(),
                    installed: true,
                    version_match: ver == expected,
                    version: ver,
                    expected_version: expected.into(),
                };
            }
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
