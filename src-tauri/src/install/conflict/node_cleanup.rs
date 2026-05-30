//! Node.js 冲突清理
//!
//! 安装新版 Node.js 前，清理旧版本残留：
//! 1. 从注册表查找旧版 Node.js 的 MSI 产品代码并静默卸载
//! 2. 移除系统 PATH 中所有指向旧 Node.js 的条目
//! 3. 清除旧的 NODE_HOME 环境变量
//!
//! 参考旧 PowerShell 脚本 AutoSetup_EN_v3_Concurrent.ps1 第 220-251 行。

use super::path_sanitizer;
use crate::common::process::hide_window;
use crate::install::emit_status;
use crate::system::env_config;
use std::process::Command;
use tauri::AppHandle;
use winreg::enums::*;
use winreg::RegKey;

/// 执行 Node.js 安装前的冲突清理。
///
/// 依次完成：MSI 卸载 → PATH 净化 → 环境变量重置。
/// 任何步骤失败均记录警告但不中断流程，确保后续安装可以继续。
pub fn cleanup(app: &AppHandle) -> Result<(), String> {
    emit_status(app, "nodejs", "config", "正在清理旧版 Node.js...");

    uninstall_node_msi(app);
    path_sanitizer::remove_path_entries_for_exe("node.exe", None);
    env_config::remove_env("NODE_HOME");

    emit_status(app, "nodejs", "config", "旧版 Node.js 清理完成");
    Ok(())
}

/// 从注册表 Uninstall 键查找 Node.js 的 MSI 产品代码，
/// 然后调用 `msiexec /x {ProductCode} /qn` 静默卸载。
///
/// 扫描路径：
/// - HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall
/// - HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall
/// - HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall
fn uninstall_node_msi(app: &AppHandle) {
    let uninstall_paths: &[(isize, &str)] = &[
        (
            HKEY_LOCAL_MACHINE as isize,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_LOCAL_MACHINE as isize,
            r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_CURRENT_USER as isize,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
    ];

    for &(root_key, path) in uninstall_paths {
        let hkey = match root_key as u32 {
            0x80000001 => RegKey::predef(HKEY_CURRENT_USER),
            _ => RegKey::predef(HKEY_LOCAL_MACHINE),
        };

        let Ok(key) = hkey.open_subkey_with_flags(path, KEY_READ) else {
            continue;
        };

        for name in key.enum_keys().filter_map(|k| k.ok()) {
            let Ok(subkey) = key.open_subkey_with_flags(&name, KEY_READ) else {
                continue;
            };

            let display: String = subkey.get_value("DisplayName").unwrap_or_default();
            if !display.to_lowercase().contains("node.js") {
                continue;
            }

            let uninstall_string: String =
                subkey.get_value("UninstallString").unwrap_or_default();
            let product_code = extract_msi_product_code(&uninstall_string);

            if let Some(code) = product_code {
                emit_status(
                    app,
                    "nodejs",
                    "config",
                    &format!("正在卸载旧版 Node.js ({display})..."),
                );

                let result =
                    hide_window(Command::new("msiexec.exe").args(["/x", &code, "/qn"]))
                        .output();

                match result {
                    Ok(output) if output.status.success() => {
                        emit_status(
                            app,
                            "nodejs",
                            "config",
                            &format!("旧版 Node.js ({display}) 卸载成功"),
                        );
                        std::thread::sleep(std::time::Duration::from_secs(3));
                    }
                    Ok(output) => {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        emit_status(
                            app,
                            "nodejs",
                            "config",
                            &format!("旧版 Node.js 卸载返回非零: {stderr}"),
                        );
                    }
                    Err(e) => {
                        emit_status(
                            app,
                            "nodejs",
                            "config",
                            &format!("执行 msiexec 失败: {e}"),
                        );
                    }
                }
                return;
            }
        }
    }
}

/// 从 UninstallString 中提取 MSI 产品代码。
///
/// 典型格式：`MsiExec.exe /I{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}`
/// 或 `MsiExec.exe /X{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}`
fn extract_msi_product_code(uninstall_string: &str) -> Option<String> {
    let start = uninstall_string.find('{')?;
    let end = uninstall_string.find('}')?;
    if end > start {
        Some(uninstall_string[start..=end].to_string())
    } else {
        None
    }
}
