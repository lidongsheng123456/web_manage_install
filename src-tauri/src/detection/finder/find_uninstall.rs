use std::path::Path;
use winreg::enums::*;
use winreg::RegKey;

/// 查询 Uninstall 注册表键，通过 DisplayName 匹配查找软件安装位置。
pub fn find_install_location(display_name_contains: &str) -> Vec<String> {
    let uninstall_keys = [
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_CURRENT_USER,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
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
