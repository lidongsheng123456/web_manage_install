//! 注册表环境变量实时读取 + 动态可执行文件发现
//!
//! Tauri 进程启动时继承的 PATH 是启动那一刻的快照，
//! 安装器写入注册表的新路径不会自动刷新到当前进程。
//! 本模块从 HKLM + HKCU 注册表实时读取最新环境变量，
//! 并提供多策略动态发现可执行文件的工具函数。

use std::path::Path;
use std::process::Command;
use winreg::enums::*;
use winreg::RegKey;

const SYS_ENV: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";
const USER_ENV: &str = r"Environment";

// ─── 注册表环境变量 ──────────────────────────────────────────

/// 从注册表实时读取 HKLM + HKCU 的 PATH 并合并。
pub fn build_fresh_path() -> String {
    let sys_path: String = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(SYS_ENV, KEY_READ)
        .and_then(|key| key.get_value("Path"))
        .unwrap_or_default();

    let user_path: String = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(USER_ENV, KEY_READ)
        .and_then(|key| key.get_value("Path"))
        .unwrap_or_default();

    match (sys_path.is_empty(), user_path.is_empty()) {
        (_, true) => sys_path,
        (true, _) => user_path,
        _ => format!("{};{}", sys_path.trim_end_matches(';'), user_path),
    }
}

/// 从注册表实时读取 JAVA_HOME / MAVEN_HOME / MYSQL_HOME / NODE_HOME。
pub fn read_fresh_env_vars() -> Vec<(String, String)> {
    let keys = ["JAVA_HOME", "MAVEN_HOME", "MYSQL_HOME", "NODE_HOME"];

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(SYS_ENV, KEY_READ)
        .ok();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(USER_ENV, KEY_READ)
        .ok();

    keys.iter()
        .filter_map(|&name| {
            let val: Option<String> = hkcu
                .as_ref()
                .and_then(|k| k.get_value(name).ok())
                .or_else(|| hklm.as_ref().and_then(|k| k.get_value(name).ok()));
            val.map(|v| (name.to_string(), v))
        })
        .collect()
}

// ─── 命令执行 ────────────────────────────────────────────────

/// 使用最新 PATH + 环境变量执行外部命令，返回合并的 stdout+stderr。
pub fn run_cmd_fresh(program: &str, args: &[&str]) -> Option<String> {
    let fresh_path = build_fresh_path();
    let fresh_envs = read_fresh_env_vars();

    let mut cmd = Command::new(program);
    cmd.args(args).env("PATH", &fresh_path);
    for (k, v) in &fresh_envs {
        cmd.env(k, v);
    }

    collect_output(&mut cmd)
}

/// 在指定路径直接运行可执行文件并获取输出。
pub fn try_exe_at(exe_path: &str, args: &[&str]) -> Option<String> {
    if !Path::new(exe_path).exists() {
        return None;
    }
    let fresh_path = build_fresh_path();
    let fresh_envs = read_fresh_env_vars();

    let mut cmd = Command::new(exe_path);
    cmd.args(args).env("PATH", &fresh_path);
    for (k, v) in &fresh_envs {
        cmd.env(k, v);
    }

    collect_output(&mut cmd)
}

fn collect_output(cmd: &mut Command) -> Option<String> {
    cmd.output().ok().and_then(|o| {
        let stdout = String::from_utf8_lossy(&o.stdout).to_string();
        let stderr = String::from_utf8_lossy(&o.stderr).to_string();
        let text = if stdout.trim().is_empty() { stderr } else { stdout };
        if text.trim().is_empty() { None } else { Some(text) }
    })
}

/// 用正则从文本中提取第一个捕获组。
pub fn extract_ver(text: &str, pattern: &str) -> String {
    regex_lite::Regex::new(pattern)
        .ok()
        .and_then(|r| r.captures(text))
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_default()
}

// ─── 动态发现工具 ────────────────────────────────────────────

/// 通过 `where` 命令搜索 PATH 中的可执行文件，返回所有匹配路径。
///
/// 使用注册表实时 PATH 而非进程启动时 PATH。
pub fn find_via_where(exe_name: &str) -> Vec<String> {
    let fresh_path = build_fresh_path();
    let mut cmd = Command::new("where");
    cmd.arg(exe_name)
        .env("PATH", &fresh_path)
        .stderr(std::process::Stdio::null());

    match cmd.output() {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && Path::new(l).exists())
            .collect(),
        _ => Vec::new(),
    }
}

/// 查询 Windows App Paths 注册表，获取已注册应用的可执行文件路径。
///
/// 许多安装程序会在此注册表键下注册（如 HKLM\...\App Paths\xxx.exe），
/// 无需修改 PATH 即可被 ShellExecute 发现。
pub fn check_app_paths(exe_name: &str) -> Option<String> {
    let subkey = format!(
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\{}",
        exe_name
    );

    for hkey in [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
        if let Ok(key) = RegKey::predef(hkey).open_subkey_with_flags(&subkey, KEY_READ) {
            if let Ok(path) = key.get_value::<String, _>("") {
                let clean = path.trim_matches('"').to_string();
                if Path::new(&clean).exists() {
                    return Some(clean);
                }
            }
        }
    }
    None
}

/// 查询 Uninstall 注册表键，通过 DisplayName 匹配查找软件安装位置。
///
/// 返回匹配项的 InstallLocation 路径。
pub fn find_install_location(display_name_contains: &str) -> Vec<String> {
    let uninstall_keys = [
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
    ];

    let mut results = Vec::new();
    let needle = display_name_contains.to_lowercase();

    for (root, path) in &uninstall_keys {
        let Ok(key) = RegKey::predef(*root).open_subkey_with_flags(path, KEY_READ) else {
            continue;
        };
        for name in key.enum_keys().filter_map(|k| k.ok()) {
            let Ok(subkey) = key.open_subkey_with_flags(&name, KEY_READ) else {
                continue;
            };
            let display: String = subkey.get_value("DisplayName").unwrap_or_default();
            if display.to_lowercase().contains(&needle) {
                if let Ok(loc) = subkey.get_value::<String, _>("InstallLocation") {
                    let loc = loc.trim_end_matches('\\').to_string();
                    if !loc.is_empty() && Path::new(&loc).is_dir() && !results.contains(&loc) {
                        results.push(loc);
                    }
                }
            }
        }
    }
    results
}

/// 在 Program Files 目录中按相对路径模式搜索可执行文件。
///
/// `relative_paths` 为相对于 Program Files 的路径，如 `nodejs\\node.exe`。
pub fn scan_program_files(relative_paths: &[&str]) -> Vec<String> {
    let pf = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into());
    let pf86 =
        std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".into());

    let mut results = Vec::new();
    for base in [&pf, &pf86] {
        for rel in relative_paths {
            let full = format!("{}\\{}", base, rel);
            if Path::new(&full).exists() && !results.contains(&full) {
                results.push(full);
            }
        }
    }
    results
}

/// 在 Program Files 下扫描匹配前缀的子目录，查找目标可执行文件。
///
/// 例如查找 `Program Files\Java\jdk-*\bin\java.exe`：
/// `scan_program_subdirs("Java", "jdk-", r"bin\java.exe")`
pub fn scan_program_subdirs(parent_dir: &str, dir_prefix: &str, exe_suffix: &str) -> Vec<String> {
    let pf = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into());
    let pf86 =
        std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".into());

    let mut results = Vec::new();
    for base in [&pf, &pf86] {
        let search_dir = format!("{}\\{}", base, parent_dir);
        results.extend(scan_dir_with_prefix(&search_dir, dir_prefix, exe_suffix));
    }
    results
}

/// 在可用磁盘驱动器的常见开发目录中搜索可执行文件。
///
/// 扫描各盘符下的 develop/software, software, dev, tools 等常见目录。
pub fn scan_common_install_dirs(relative_paths: &[&str]) -> Vec<String> {
    let common_roots = [
        "develop\\software",
        "software",
        "dev",
        "tools",
        "DevTools",
        "Programs",
    ];

    let mut results = Vec::new();
    for drive in get_local_drives() {
        for root in &common_roots {
            let base = format!("{}\\{}", drive, root);
            if !Path::new(&base).is_dir() {
                continue;
            }
            for rel in relative_paths {
                let full = format!("{}\\{}", base, rel);
                if Path::new(&full).exists() && !results.contains(&full) {
                    results.push(full);
                }
            }
        }
        for rel in relative_paths {
            let full = format!("{}\\{}", drive, rel);
            if Path::new(&full).exists() && !results.contains(&full) {
                results.push(full);
            }
        }
    }
    results
}

/// 在可用磁盘驱动器的常见目录中扫描匹配前缀的子目录。
pub fn scan_common_subdirs(dir_name: &str, dir_prefix: &str, exe_suffix: &str) -> Vec<String> {
    let common_roots = [
        "develop\\software",
        "software",
        "dev",
        "tools",
        "DevTools",
        "Programs",
    ];

    let mut results = Vec::new();
    for drive in get_local_drives() {
        for root in &common_roots {
            let search = format!("{}\\{}\\{}", drive, root, dir_name);
            results.extend(scan_dir_with_prefix(&search, dir_prefix, exe_suffix));
        }
        let search = format!("{}\\{}", drive, dir_name);
        results.extend(scan_dir_with_prefix(&search, dir_prefix, exe_suffix));
    }
    results
}

fn scan_dir_with_prefix(parent: &str, dir_prefix: &str, exe_suffix: &str) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(parent) else {
        return Vec::new();
    };
    let prefix_lower = dir_prefix.to_lowercase();
    entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().map(|t| t.is_dir()).unwrap_or(false)
                && e.file_name()
                    .to_string_lossy()
                    .to_lowercase()
                    .starts_with(&prefix_lower)
        })
        .filter_map(|e| {
            let full = format!("{}\\{}", e.path().to_string_lossy(), exe_suffix);
            if Path::new(&full).exists() {
                Some(full)
            } else {
                None
            }
        })
        .collect()
}

fn get_local_drives() -> Vec<String> {
    ('C'..='H')
        .map(|c| format!("{}:", c))
        .filter(|d| Path::new(&format!("{}\\", d)).exists())
        .collect()
}
