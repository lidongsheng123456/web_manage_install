//! MySQL 环境检测
//!
//! 检测策略（按优先级）：
//! 1. 使用最新注册表 PATH 执行 `mysql -V`
//! 2. 通过 Windows 服务定位 MySQL 安装路径
//! 3. 查询 MySQL 注册表键
//! 4. 扫描常见安装目录

use super::env_reader::{extract_ver, run_cmd_fresh, try_exe_at};
use crate::types::ComponentStatus;

/// 常见 MySQL 安装路径
const COMMON_PATHS: &[&str] = &[
    r"C:\Program Files\MySQL\MySQL Server 8.0\bin\mysql.exe",
    r"D:\develop\software\mysql\bin\mysql.exe",
    r"D:\develop\software\mysql-8.0.36-winx64\bin\mysql.exe",
    r"D:\develop\software\mysql-8.0.37-winx64\bin\mysql.exe",
    r"C:\mysql\bin\mysql.exe",
];

pub fn detect(expected_prefix: &str) -> ComponentStatus {
    let expected_label = format!("{expected_prefix}.x");

    // 1) PATH 执行 mysql -V
    if let Some(ver) = version_from_output(run_cmd_fresh("mysql", &["-V"])) {
        return build_status(ver, expected_prefix, &expected_label);
    }

    // 2) Windows 服务 → 找到 mysql.exe
    if let Some(exe) = detect_via_service() {
        if let Some(ver) = version_from_output(try_exe_at(&exe, &["-V"])) {
            return build_status(ver, expected_prefix, &expected_label);
        }
    }

    // 3) 注册表
    if let Some(exe) = detect_via_registry() {
        if let Some(ver) = version_from_output(try_exe_at(&exe, &["-V"])) {
            return build_status(ver, expected_prefix, &expected_label);
        }
    }

    // 4) 常见路径
    for path in COMMON_PATHS {
        if let Some(ver) = version_from_output(try_exe_at(path, &["-V"])) {
            return build_status(ver, expected_prefix, &expected_label);
        }
    }

    ComponentStatus {
        name: "MySQL".into(),
        installed: false,
        version: String::new(),
        expected_version: expected_label,
        version_match: false,
    }
}

fn version_from_output(output: Option<String>) -> Option<String> {
    let text = output?;
    let ver = extract_ver(&text, r"(\d+\.\d+\.\d+)");
    if ver.is_empty() { None } else { Some(ver) }
}

fn build_status(ver: String, expected_prefix: &str, expected_label: &str) -> ComponentStatus {
    ComponentStatus {
        name: "MySQL".into(),
        installed: true,
        version_match: ver.starts_with(&format!("{expected_prefix}.")),
        version: ver,
        expected_version: expected_label.into(),
    }
}

/// 通过 `sc qc MySQL80` 查询 Windows 服务获取 MySQL 安装路径
fn detect_via_service() -> Option<String> {
    let output = run_cmd_fresh("sc", &["qc", "MySQL80"])?;
    let bin_path = extract_ver(&output, r"(?i)BINARY_PATH_NAME\s*:\s*(.+)");
    if bin_path.is_empty() {
        return None;
    }
    let clean = bin_path.trim().trim_matches('"').to_string();
    let bin_dir = std::path::Path::new(&clean).parent()?;
    let mysql_exe = bin_dir.join("mysql.exe");
    if mysql_exe.exists() {
        Some(mysql_exe.to_string_lossy().to_string())
    } else {
        None
    }
}

/// 从注册表查找 MySQL 安装路径
fn detect_via_registry() -> Option<String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let reg_paths = [
        r"SOFTWARE\MySQL AB\MySQL Server 8.0",
        r"SOFTWARE\WOW6432Node\MySQL AB\MySQL Server 8.0",
    ];
    for reg_path in &reg_paths {
        if let Ok(key) = hklm.open_subkey_with_flags(reg_path, KEY_READ) {
            if let Ok(loc) = key.get_value::<String, _>("Location") {
                let exe = format!(r"{}\bin\mysql.exe", loc.trim_end_matches('\\'));
                if std::path::Path::new(&exe).exists() {
                    return Some(exe);
                }
            }
        }
    }
    None
}
