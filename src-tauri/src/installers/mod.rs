//! 组件安装器模块
//!
//! 每个组件有独立的安装子模块，统一通过 `install_all` 命令入口调度。

pub mod jdk;
pub mod maven;
pub mod mysql;
pub mod node;
mod utils;

use crate::types::{DownloadProgress, InstallConfig, InstallEvent, InstallResult};
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter};

/// 检查当前进程是否以管理员身份运行。
///
/// 通过尝试写入 HKLM 注册表来判断，避免依赖 Windows API。
fn is_elevated() -> bool {
    use winreg::enums::*;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    hklm.open_subkey_with_flags(
        r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
        KEY_SET_VALUE,
    ).is_ok()
}

/// 向前端推送安装阶段状态事件。
pub(crate) fn emit_status(app: &AppHandle, component: &str, phase: &str, msg: &str) {
    let _ = app.emit(
        "install-status",
        InstallEvent {
            component: component.into(),
            phase: phase.into(),
            message: msg.into(),
            success: true,
            done: false,
        },
    );
}

/// 向前端推送组件安装完成（成功或失败）事件。
pub(crate) fn emit_done(app: &AppHandle, component: &str, success: bool, msg: &str) {
    let _ = app.emit(
        "install-status",
        InstallEvent {
            component: component.into(),
            phase: if success { "complete" } else { "error" }.into(),
            message: msg.into(),
            success,
            done: true,
        },
    );
}

/// 统一安装入口：根据用户配置依次安装选中的组件。
///
/// 每个组件按 下载 → 解压/安装 → 配置环境变量 的流水线执行，
/// 结果通过 `install-status` 事件实时推送，全部完成后通过
/// `install-complete` 事件发送汇总。
#[tauri::command]
pub async fn install_all(
    app: AppHandle,
    config: InstallConfig,
    on_progress: Channel<DownloadProgress>,
) -> Result<Vec<InstallResult>, String> {
    let root = config.install_root.clone();
    let temp = format!("{root}\\_temp");
    std::fs::create_dir_all(&temp).ok();

    // 非测试模式下检查管理员权限
    if !config.dry_run && !is_elevated() {
        emit_status(&app, "nodejs", "config",
            "提示: 未以管理员身份运行，环境变量将写入用户级别（HKCU）");
    }

    let mut results = Vec::new();
    let dry = config.dry_run;
    let ver = &config;

    if config.install_nodejs {
        if dry {
            run_install(&app, "nodejs", dry_run_download(&app, "nodejs", &ver.node_version, &temp, &on_progress).await, &mut results);
        } else {
            run_install(&app, "nodejs", node::install(&app, &root, &temp, &ver.node_version, &on_progress).await, &mut results);
        }
    }
    if config.install_jdk {
        if dry {
            run_install(&app, "jdk", dry_run_download(&app, "jdk", &ver.jdk_version, &temp, &on_progress).await, &mut results);
        } else {
            run_install(&app, "jdk", jdk::install(&app, &root, &temp, &ver.jdk_version, &on_progress).await, &mut results);
        }
    }
    if config.install_maven {
        if dry {
            run_install(&app, "maven", dry_run_download(&app, "maven", &ver.maven_version, &temp, &on_progress).await, &mut results);
        } else {
            run_install(&app, "maven", maven::install(&app, &root, &temp, &ver.maven_version, &on_progress).await, &mut results);
        }
    }
    if config.install_mysql {
        if dry {
            run_install(&app, "mysql", dry_run_download(&app, "mysql", &ver.mysql_version, &temp, &on_progress).await, &mut results);
        } else {
            run_install(&app, "mysql", mysql::install(&app, &root, &temp, &ver.mysql_version, &config.mysql_password, &on_progress).await, &mut results);
        }
    }

    // 清理临时下载目录
    if !dry {
        let _ = std::fs::remove_dir_all(&temp);
    }

    let _ = app.emit("install-complete", &results);
    Ok(results)
}

/// 模拟测试模式：仅执行下载验证，完成后推送成功事件。
/// 不执行解压、安装、环境变量等操作，保护用户系统环境。
async fn dry_run_download(
    app: &AppHandle,
    component: &str,
    version: &str,
    temp_dir: &str,
    on_progress: &Channel<DownloadProgress>,
) -> Result<(), String> {
    emit_status(app, component, "download", &format!("[测试模式] 正在下载 {component}..."));
    crate::download::download_with_version(component, version, temp_dir, on_progress).await?;
    emit_status(app, component, "config", &format!("[测试模式] {component} 下载验证成功，跳过安装步骤"));
    emit_done(app, component, true, &format!("[测试模式] {component} 下载验证通过"));
    Ok(())
}

/// 将单个组件的安装结果录入结果列表，失败时额外推送 done 事件。
fn run_install(app: &AppHandle, name: &str, result: Result<(), String>, results: &mut Vec<InstallResult>) {
    match result {
        Ok(()) => results.push(InstallResult {
            component: name.into(),
            success: true,
            message: format!("{name} 安装成功"),
        }),
        Err(e) => {
            emit_done(app, name, false, &e);
            results.push(InstallResult {
                component: name.into(),
                success: false,
                message: e,
            });
        }
    }
}
