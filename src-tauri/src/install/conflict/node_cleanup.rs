//! Node.js 冲突清理
//!
//! 安装新版 Node.js 前，清理旧版本残留：
//! 1. 从注册表查找旧版 Node.js 的 MSI 产品代码并静默卸载
//! 2. 删除旧安装目录中的残留文件
//! 3. 移除系统 PATH 中所有指向旧 Node.js 的条目
//! 4. 清除旧的 NODE_HOME 环境变量
//!
//! 参考旧 PowerShell 脚本 AutoSetup_EN_v3_Concurrent.ps1 第 220-251 行。

use super::{file_cleaner, path_cleaner, path_scanner};
use crate::common::process::hide_window;
use crate::install::emit_status;
use crate::system::env_config;
use std::process::Command;
use tauri::AppHandle;
use winreg::enums::*;
use winreg::RegKey;

/// 执行 Node.js 安装前的冲突清理。
///
/// 安全流程：扫描 PATH → 终止旧进程 → MSI 卸载 → 等待 MSI 空闲 → 删除残留 → 清理 PATH → 重置环境变量。
///
/// **关键时序**：进程终止必须在 MSI 卸载之前执行。如果 node.exe 进程仍在运行：
/// - MSI 卸载无法正常操作被锁文件，会返回非零退出码或在后台挂起
/// - 后续新版本 MSI 安装会遇到 exit 1618（另一个 MSI 操作正在进行）
pub fn cleanup(app: &AppHandle) -> Result<(), String> {
    emit_status(app, "nodejs", "config", "正在清理旧版 Node.js...");

    // 第 1 步：扫描并保存所有信息（进程终止和 MSI 卸载后 exe 不存在，无法再扫描）
    let path_entries = path_scanner::find_path_entries_for_exe("node.exe");
    let install_roots = path_scanner::find_install_roots_from_path("node.exe", 0);

    // 收集所有旧安装目录
    let mut all_roots = Vec::new();
    if let Some(node_home) = path_scanner::read_env_var("NODE_HOME") {
        all_roots.push(node_home);
    }
    all_roots.extend(install_roots.iter().cloned());

    // 第 2 步：终止旧目录中的 Node.js 进程（必须在 MSI 卸载前，释放文件锁）
    for root in &all_roots {
        emit_status(
            app,
            "nodejs",
            "config",
            &format!("正在终止旧 Node.js 目录中的进程: {root}"),
        );
        file_cleaner::kill_processes_from_dir(root);
    }

    // 第 3 步：MSI 静默卸载（进程已终止，MSI 可正常操作）
    uninstall_node_msi(app);

    // 第 4 步：等待 MSI 操作完全结束（防止新安装 MSI 遇到 exit 1618）
    wait_for_msi_idle(app);

    // 第 5 步：删除残留文件
    for root in &all_roots {
        emit_status(
            app,
            "nodejs",
            "config",
            &format!("正在删除旧 Node.js 目录: {root}"),
        );
        file_cleaner::remove_dir_safe(root);
    }

    // 第 6 步：移除已采集的 PATH 条目
    path_cleaner::remove_entries(&path_entries, None);

    // 第 7 步：清除旧的 NODE_HOME
    env_config::remove_env("NODE_HOME");

    emit_status(app, "nodejs", "config", "旧版 Node.js 清理完成");
    Ok(())
}

/// 等待 Windows Installer 服务空闲（无 msiexec 进程在运行）。
///
/// MSI 安装和卸载是互斥的，Windows 同一时间只能运行一个 MSI 操作。
/// 如果上一个 MSI 卸载操作仍在后台运行，新的 MSI 安装会返回 exit 1618。
/// 最多等待 30 秒。
fn wait_for_msi_idle(app: &AppHandle) {
    for i in 0..30 {
        let output = hide_window(Command::new("cmd").args([
            "/C",
            "tasklist /FI \"IMAGENAME eq msiexec.exe\" /NH 2>nul",
        ]))
        .output();

        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                if !stdout.to_lowercase().contains("msiexec.exe") {
                    return;
                }
                if i == 0 {
                    emit_status(
                        app,
                        "nodejs",
                        "config",
                        "正在等待上一个 MSI 操作完成...",
                    );
                }
            }
            Err(_) => return,
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
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
