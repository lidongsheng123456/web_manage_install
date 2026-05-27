//! 系统环境变量配置模块
//!
//! 设置 Windows 环境变量，支持三级降级策略：
//! 1. 优先写入 HKLM（系统级，需要管理员权限）
//! 2. HKLM 失败时回退到 HKCU（用户级，无需管理员）
//! 3. 注册表操作全部失败时通过 `setx` 命令兜底

use std::process::Command;
use winreg::enums::*;
use winreg::RegKey;

/// HKLM 系统级环境变量路径
const SYS_ENV: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";
/// HKCU 用户级环境变量路径
const USER_ENV: &str = r"Environment";

/// 设置环境变量，自动降级：HKLM → HKCU → setx。
///
/// 返回 Ok 表示至少一种方式成功。
pub fn set_system_env(key: &str, value: &str) -> Result<(), String> {
    // 尝试 HKLM（系统级）
    if try_set_hklm(key, value).is_ok() {
        return Ok(());
    }

    // HKLM 失败，尝试 HKCU（用户级）
    if try_set_hkcu(key, value).is_ok() {
        broadcast_env_change();
        return Ok(());
    }

    // 注册表都失败，用 setx 兜底
    try_setx(key, value)
}

/// 向 PATH 追加路径条目，自动降级。
///
/// 先读取当前 PATH 判断是否已存在（大小写不敏感），
/// 不存在才追加。写入策略同 `set_system_env`。
pub fn append_to_path(new_entry: &str) -> Result<(), String> {
    let normalized = new_entry.trim_end_matches('\\').to_lowercase();

    // 优先从 HKLM 读取 + 写入
    if let Ok(current) = read_path_from_hklm() {
        if path_contains(&current, &normalized) {
            return Ok(());
        }
        let updated = append_entry(&current, new_entry);
        if try_write_path_hklm(&updated).is_ok() {
            broadcast_env_change();
            return Ok(());
        }
    }

    // HKLM 失败，尝试 HKCU
    if let Ok(current) = read_path_from_hkcu() {
        if path_contains(&current, &normalized) {
            return Ok(());
        }
        let updated = append_entry(&current, new_entry);
        if try_write_path_hkcu(&updated).is_ok() {
            broadcast_env_change();
            return Ok(());
        }
    }

    // 都失败，用 setx PATH 追加（setx 会写 HKCU）
    let current_path = std::env::var("PATH").unwrap_or_default();
    if path_contains(&current_path, &normalized) {
        return Ok(());
    }
    // setx 单次最大值 1024 字符，只追加新条目
    try_setx("PATH", &format!("{current_path};{new_entry}"))
}

// ─── 内部实现 ──────────────────────────────────────────────────

fn try_set_hklm(key: &str, value: &str) -> Result<(), String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let env = hklm.open_subkey_with_flags(SYS_ENV, KEY_SET_VALUE)
        .map_err(|e| format!("HKLM 打开失败: {e}"))?;
    env.set_value(key, &value)
        .map_err(|e| format!("HKLM 写入 {key} 失败: {e}"))
}

fn try_set_hkcu(key: &str, value: &str) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu.open_subkey_with_flags(USER_ENV, KEY_SET_VALUE)
        .map_err(|e| format!("HKCU 打开失败: {e}"))?;
    env.set_value(key, &value)
        .map_err(|e| format!("HKCU 写入 {key} 失败: {e}"))
}

fn try_setx(key: &str, value: &str) -> Result<(), String> {
    let output = Command::new("setx").args([key, value]).output()
        .map_err(|e| format!("setx 执行失败: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(format!("setx {key} 失败: {err}"))
    }
}

fn read_path_from_hklm() -> Result<String, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let env = hklm.open_subkey_with_flags(SYS_ENV, KEY_READ)
        .map_err(|e| format!("{e}"))?;
    env.get_value("Path").map_err(|e| format!("{e}"))
}

fn read_path_from_hkcu() -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu.open_subkey_with_flags(USER_ENV, KEY_READ)
        .map_err(|e| format!("{e}"))?;
    env.get_value("Path").map_err(|e| format!("{e}"))
}

fn try_write_path_hklm(value: &str) -> Result<(), String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let env = hklm.open_subkey_with_flags(SYS_ENV, KEY_SET_VALUE)
        .map_err(|e| format!("{e}"))?;
    env.set_value("Path", &value).map_err(|e| format!("{e}"))
}

fn try_write_path_hkcu(value: &str) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu.open_subkey_with_flags(USER_ENV, KEY_SET_VALUE)
        .map_err(|e| format!("{e}"))?;
    env.set_value("Path", &value).map_err(|e| format!("{e}"))
}

/// 检查 PATH 字符串中是否已包含目标路径（大小写不敏感）
fn path_contains(path_str: &str, normalized_entry: &str) -> bool {
    path_str.split(';')
        .filter(|s| !s.is_empty())
        .any(|e| e.trim_end_matches('\\').to_lowercase() == *normalized_entry)
}

/// 向 PATH 字符串末尾追加条目
fn append_entry(current: &str, new_entry: &str) -> String {
    if current.ends_with(';') || current.is_empty() {
        format!("{current}{new_entry}")
    } else {
        format!("{current};{new_entry}")
    }
}

/// 广播环境变量变更通知
fn broadcast_env_change() {
    let _ = Command::new("cmd")
        .args(["/C", "setx", "DEVENV_UPDATED", "1"])
        .output();
}
