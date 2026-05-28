use std::process::Command;
use winreg::enums::*;
use winreg::RegKey;

/// HKLM 系统级环境变量路径。
pub const SYS_ENV: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";
/// HKCU 用户级环境变量路径。
pub const USER_ENV: &str = r"Environment";

/// 写入系统级环境变量，需要管理员权限。
pub fn try_set_hklm(key: &str, value: &str) -> Result<(), String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let env = hklm
        .open_subkey_with_flags(SYS_ENV, KEY_SET_VALUE)
        .map_err(|e| format!("HKLM 打开失败: {e}"))?;
    env.set_value(key, &value)
        .map_err(|e| format!("HKLM 写入 {key} 失败: {e}"))
}

/// 写入用户级环境变量，作为无管理员权限时的降级方案。
pub fn try_set_hkcu(key: &str, value: &str) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags(USER_ENV, KEY_SET_VALUE)
        .map_err(|e| format!("HKCU 打开失败: {e}"))?;
    env.set_value(key, &value)
        .map_err(|e| format!("HKCU 写入 {key} 失败: {e}"))
}

/// 通过 setx 写入用户级环境变量，作为注册表 API 失败时的最后兜底。
pub fn try_setx(key: &str, value: &str) -> Result<(), String> {
    let output = Command::new("setx")
        .args([key, value])
        .output()
        .map_err(|e| format!("setx 执行失败: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(format!("setx {key} 失败: {err}"))
    }
}

pub fn read_path_from_hklm() -> Result<String, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let env = hklm
        .open_subkey_with_flags(SYS_ENV, KEY_READ)
        .map_err(|e| format!("{e}"))?;
    env.get_value("Path").map_err(|e| format!("{e}"))
}

pub fn read_path_from_hkcu() -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags(USER_ENV, KEY_READ)
        .map_err(|e| format!("{e}"))?;
    env.get_value("Path").map_err(|e| format!("{e}"))
}

pub fn try_write_path_hklm(value: &str) -> Result<(), String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let env = hklm
        .open_subkey_with_flags(SYS_ENV, KEY_SET_VALUE)
        .map_err(|e| format!("{e}"))?;
    env.set_value("Path", &value).map_err(|e| format!("{e}"))
}

pub fn try_write_path_hkcu(value: &str) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags(USER_ENV, KEY_SET_VALUE)
        .map_err(|e| format!("{e}"))?;
    env.set_value("Path", &value).map_err(|e| format!("{e}"))
}

pub fn delete_env_value_hklm(key: &str) {
    let _ = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(SYS_ENV, KEY_SET_VALUE)
        .and_then(|env| env.delete_value(key));
}

pub fn delete_env_value_hkcu(key: &str) {
    let _ = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(USER_ENV, KEY_SET_VALUE)
        .and_then(|env| env.delete_value(key));
}
