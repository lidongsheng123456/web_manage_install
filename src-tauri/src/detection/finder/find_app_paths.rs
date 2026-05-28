use std::path::Path;
use winreg::enums::*;
use winreg::RegKey;

/// 查询 Windows App Paths 注册表，获取已注册应用的可执行文件路径。
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
