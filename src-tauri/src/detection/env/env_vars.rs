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

/// 从注册表实时读取安装器关心的 HOME 类环境变量。
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
