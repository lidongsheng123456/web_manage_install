//! MySQL 环境检测
//!
//! 检测策略（按优先级）：
//! 1. 使用最新注册表 PATH 执行 `mysql -V`
//! 2. 从 MYSQL_HOME 环境变量定位
//! 3. 通过 Windows 服务定位 MySQL 安装路径
//! 4. 查询 MySQL AB 注册表键
//! 5. 通过 `where mysql` 搜索 PATH
//! 6. 查询 Uninstall 注册表获取安装位置
//! 7. 扫描 Program Files 和常见安装目录

use crate::common::types::ComponentStatus;
use crate::common::version_policy::mysql as mysql_policy;
use crate::detection::env::*;

pub fn detect(expected_version: &str) -> ComponentStatus {
    let expected_label = expected_version.to_string();

    // 1) PATH 执行 mysql -V
    if let Some(ver) = parse_mysql_output(run_cmd_fresh("mysql", &["-V"])) {
        return status(ver, expected_version, &expected_label);
    }

    // 2) MYSQL_HOME 环境变量
    for (k, v) in read_fresh_env_vars() {
        if k == "MYSQL_HOME" {
            let exe = format!(r"{}\bin\mysql.exe", v.trim_end_matches('\\'));
            if let Some(ver) = parse_mysql_output(try_exe_at(&exe, &["-V"])) {
                return status(ver, expected_version, &expected_label);
            }
        }
    }

    // 3) Windows 服务 → 找到 mysql.exe
    if let Some(exe) = detect_via_service() {
        if let Some(ver) = parse_mysql_output(try_exe_at(&exe, &["-V"])) {
            return status(ver, expected_version, &expected_label);
        }
    }

    // 4) MySQL AB 注册表
    if let Some(exe) = detect_via_mysql_registry() {
        if let Some(ver) = parse_mysql_output(try_exe_at(&exe, &["-V"])) {
            return status(ver, expected_version, &expected_label);
        }
    }

    // 5) where 命令搜索
    for path in find_via_where("mysql") {
        if let Some(ver) = parse_mysql_output(try_exe_at(&path, &["-V"])) {
            return status(ver, expected_version, &expected_label);
        }
    }

    // 6) Uninstall 注册表
    for loc in find_install_location("MySQL") {
        let exe = format!(r"{}\bin\mysql.exe", loc.trim_end_matches('\\'));
        if let Some(ver) = parse_mysql_output(try_exe_at(&exe, &["-V"])) {
            return status(ver, expected_version, &expected_label);
        }
    }

    // 7) 扫描 Program Files
    for path in scan_program_subdirs("MySQL", "MySQL Server", r"bin\mysql.exe") {
        if let Some(ver) = parse_mysql_output(try_exe_at(&path, &["-V"])) {
            return status(ver, expected_version, &expected_label);
        }
    }

    // 8) 扫描常见安装目录
    let patterns = &[r"mysql\bin\mysql.exe"];
    for path in scan_common_install_dirs(patterns) {
        if let Some(ver) = parse_mysql_output(try_exe_at(&path, &["-V"])) {
            return status(ver, expected_version, &expected_label);
        }
    }
    for path in scan_common_subdirs("", "mysql", r"bin\mysql.exe") {
        if let Some(ver) = parse_mysql_output(try_exe_at(&path, &["-V"])) {
            return status(ver, expected_version, &expected_label);
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

fn parse_mysql_output(output: Option<String>) -> Option<String> {
    let text = output?;
    let ver = extract_ver(&text, r"(\d+\.\d+\.\d+)");
    if ver.is_empty() {
        None
    } else {
        Some(ver)
    }
}

fn status(ver: String, expected_version: &str, expected_label: &str) -> ComponentStatus {
    ComponentStatus {
        name: "MySQL".into(),
        installed: true,
        version_match: ver == expected_version,
        version: ver,
        expected_version: expected_label.into(),
    }
}

/// 通过 `sc qc` 查询 Windows 服务获取 MySQL 安装路径
fn detect_via_service() -> Option<String> {
    for svc in mysql_policy::DETECT_SERVICE_NAMES {
        if let Some(exe) = try_service(svc) {
            return Some(exe);
        }
    }
    None
}

fn try_service(service_name: &str) -> Option<String> {
    let output = run_cmd_fresh("sc", &["qc", service_name])?;
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

/// 从注册表查找 MySQL 安装路径（动态枚举版本号）
fn detect_via_mysql_registry() -> Option<String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    for prefix in [r"SOFTWARE\MySQL AB", r"SOFTWARE\WOW6432Node\MySQL AB"] {
        let Ok(ab_key) = hklm.open_subkey_with_flags(prefix, KEY_READ) else {
            continue;
        };
        for name in ab_key.enum_keys().filter_map(|k| k.ok()) {
            let Ok(sub) = ab_key.open_subkey_with_flags(&name, KEY_READ) else {
                continue;
            };
            if let Ok(loc) = sub.get_value::<String, _>("Location") {
                let exe = format!(r"{}\bin\mysql.exe", loc.trim_end_matches('\\'));
                if std::path::Path::new(&exe).exists() {
                    return Some(exe);
                }
            }
        }
    }
    None
}
