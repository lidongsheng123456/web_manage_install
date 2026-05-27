//! 注册表环境变量实时读取器
//!
//! Tauri 进程启动时继承的 PATH 是启动那一刻的快照，
//! 安装器写入注册表的新路径不会自动刷新到当前进程。
//! 本模块从 HKLM + HKCU 注册表实时读取最新环境变量，
//! 供检测和验证命令使用。

use std::process::Command;
use winreg::enums::*;
use winreg::RegKey;

const SYS_ENV: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";
const USER_ENV: &str = r"Environment";

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

/// 使用最新 PATH + 环境变量执行外部命令，返回合并的 stdout+stderr。
///
/// 解决安装后当前进程 PATH 过期导致检测不到的问题。
pub fn run_cmd_fresh(program: &str, args: &[&str]) -> Option<String> {
    let fresh_path = build_fresh_path();
    let fresh_envs = read_fresh_env_vars();

    let mut cmd = Command::new(program);
    cmd.args(args).env("PATH", &fresh_path);
    for (k, v) in &fresh_envs {
        cmd.env(k, v);
    }

    cmd.output().ok().and_then(|o| {
        let stdout = String::from_utf8_lossy(&o.stdout).to_string();
        let stderr = String::from_utf8_lossy(&o.stderr).to_string();
        let text = if stdout.trim().is_empty() { stderr } else { stdout };
        if text.trim().is_empty() { None } else { Some(text) }
    })
}

/// 在指定路径直接运行可执行文件并获取输出。
pub fn try_exe_at(exe_path: &str, args: &[&str]) -> Option<String> {
    if !std::path::Path::new(exe_path).exists() {
        return None;
    }
    let fresh_path = build_fresh_path();
    let fresh_envs = read_fresh_env_vars();

    let mut cmd = Command::new(exe_path);
    cmd.args(args).env("PATH", &fresh_path);
    for (k, v) in &fresh_envs {
        cmd.env(k, v);
    }

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
