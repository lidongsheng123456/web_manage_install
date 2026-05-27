//! 组件安装器模块
//!
//! 每个组件有独立的安装子模块，统一通过 `install_all` 命令入口调度。
//! 支持在组件之间检查取消信号，以及对已完成组件执行回滚。

pub mod bundled;
pub mod jdk;
pub mod maven;
pub mod mysql;
pub mod node;
mod utils;

use crate::env_config;
use crate::types::{CancelToken, DownloadProgress, InstallConfig, InstallEvent, InstallResult};
use std::process::Command;
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, Manager};

/// 检查当前进程是否以管理员身份运行。
fn is_elevated() -> bool {
    use winreg::enums::*;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    hklm.open_subkey_with_flags(
        r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
        KEY_SET_VALUE,
    )
    .is_ok()
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

/// 检查取消令牌，如果已取消则返回 Err。
fn check_cancel(app: &AppHandle) -> Result<(), String> {
    let cancel = app.state::<CancelToken>();
    if cancel.is_cancelled() {
        Err("用户取消安装".into())
    } else {
        Ok(())
    }
}

/// 统一安装入口：根据用户配置依次安装选中的组件。
///
/// 每个组件之间会检查取消信号，取消后停止剩余组件安装。
/// 结果通过 `install-status` 事件实时推送，全部完成后通过
/// `install-complete` 事件发送汇总。
#[tauri::command]
pub async fn install_all(
    app: AppHandle,
    config: InstallConfig,
    on_progress: Channel<DownloadProgress>,
) -> Result<Vec<InstallResult>, String> {
    let cancel = app.state::<CancelToken>();
    cancel.reset();

    let root = config.install_root.clone();
    let temp = format!("{root}\\_temp");
    std::fs::create_dir_all(&temp).ok();

    if !config.dry_run && !is_elevated() {
        emit_status(
            &app,
            "nodejs",
            "config",
            "提示: 未以管理员身份运行，环境变量将写入用户级别（HKCU）",
        );
    }

    let mut results = Vec::new();
    let dry = config.dry_run;
    let ver = &config;
    let mut cancelled = false;

    // ─── 宏：安装前检查取消信号 ───
    macro_rules! install_component {
        ($flag:expr, $name:expr, $install_expr:expr) => {
            if $flag && !cancelled {
                if check_cancel(&app).is_err() {
                    cancelled = true;
                    emit_status(&app, $name, "error", "安装已取消");
                } else {
                    run_install(&app, $name, $install_expr, &mut results);
                }
            }
        };
    }

    install_component!(
        config.install_nodejs,
        "nodejs",
        if dry {
            dry_run_download(&app, "nodejs", &ver.node_version, &temp, &on_progress).await
        } else {
            node::install(&app, &root, &temp, &ver.node_version, &on_progress).await
        }
    );

    install_component!(
        config.install_jdk,
        "jdk",
        if dry {
            dry_run_download(&app, "jdk", &ver.jdk_version, &temp, &on_progress).await
        } else {
            jdk::install(&app, &root, &temp, &ver.jdk_version, &on_progress).await
        }
    );

    install_component!(
        config.install_maven,
        "maven",
        if dry {
            dry_run_download(&app, "maven", &ver.maven_version, &temp, &on_progress).await
        } else {
            maven::install(&app, &root, &temp, &ver.maven_version, &on_progress).await
        }
    );

    install_component!(
        config.install_mysql,
        "mysql",
        if dry {
            dry_run_download(&app, "mysql", &ver.mysql_version, &temp, &on_progress).await
        } else {
            mysql::install(
                &app,
                &root,
                &temp,
                &ver.mysql_version,
                &config.mysql_password,
                &on_progress,
            )
            .await
        }
    );

    // ─── IDEA / Navicat（网络下载 + 安装）───
    install_component!(
        config.install_idea,
        "idea",
        if dry {
            dry_run_download(&app, "idea", "2023.3.8", &temp, &on_progress).await
        } else {
            bundled::install_idea(&app, &root, &temp, &on_progress).await
        }
    );

    install_component!(
        config.install_navicat,
        "navicat",
        if dry {
            dry_run_download(&app, "navicat", "16.2", &temp, &on_progress).await
        } else {
            bundled::install_navicat(&app, &root, &temp, &on_progress).await
        }
    );

    // ─── 本地小文件（打包在 exe 中）───
    let has_local = config.install_idea || config.install_navicat || config.install_redis;
    if has_local && !cancelled && !dry {
        emit_status(&app, "bundled", "install", "正在复制激活工具和 Redis 到安装目录...");
        if let Err(e) = bundled::copy_bundled_to_root(&app, &root) {
            emit_status(&app, "bundled", "error", &format!("资源复制警告: {e}"));
        }
    }

    // ─── Redis（本地解压 / dry run 验证本地资源）───
    install_component!(
        config.install_redis,
        "redis",
        if dry {
            check_redis_available(&app)
        } else {
            bundled::install_redis(&app, &root)
        }
    );

    // 清理临时下载目录
    if !dry {
        let _ = std::fs::remove_dir_all(&temp);
    }

    if cancelled {
        let cancelled_comps: Vec<String> = results.iter().map(|r| r.component.clone()).collect();
        let _ = app.emit(
            "install-cancelled",
            serde_json::json!({
                "completedComponents": cancelled_comps,
                "message": "安装已取消，可选择回滚已完成的组件"
            }),
        );
    }

    let _ = app.emit("install-complete", &results);
    Ok(results)
}

/// 回滚已安装的组件：删除安装目录、移除环境变量、清理服务。
pub fn rollback(
    app: &AppHandle,
    components: &[String],
    install_root: &str,
) -> Result<Vec<String>, String> {
    let mut rolled_back = Vec::new();

    for comp in components {
        emit_status(app, comp, "config", &format!("正在回滚 {comp}..."));

        match comp.as_str() {
            "nodejs" => {
                let dir = format!("{install_root}\\nodejs");
                env_config::remove_from_path(&format!("{dir}"));
                env_config::remove_env("NODE_HOME");
                remove_dir_safe(&dir);
                rolled_back.push("Node.js".into());
            }
            "jdk" => {
                for major in ["17", "21"] {
                    let dir = format!("{install_root}\\jdk{major}");
                    if std::path::Path::new(&dir).exists() {
                        env_config::remove_from_path(&format!("{dir}\\bin"));
                        remove_dir_safe(&dir);
                    }
                }
                env_config::remove_env("JAVA_HOME");
                rolled_back.push("JDK".into());
            }
            "maven" => {
                let dir = format!("{install_root}\\maven");
                env_config::remove_from_path(&format!("{dir}\\bin"));
                env_config::remove_env("MAVEN_HOME");
                remove_dir_safe(&dir);
                rolled_back.push("Maven".into());
            }
            "mysql" => {
                let _ = Command::new("net").args(["stop", "MySQL80"]).output();
                std::thread::sleep(std::time::Duration::from_secs(2));
                let _ = Command::new("sc").args(["delete", "MySQL80"]).output();
                let dir = format!("{install_root}\\mysql");
                env_config::remove_from_path(&format!("{dir}\\bin"));
                env_config::remove_env("MYSQL_HOME");
                remove_dir_safe(&dir);
                rolled_back.push("MySQL".into());
            }
            "idea" => {
                let dir = format!("{install_root}\\IDEA");
                remove_dir_safe(&dir);
                rolled_back.push("IDEA".into());
            }
            "navicat" => {
                let dir = format!("{install_root}\\Navicat");
                remove_dir_safe(&dir);
                rolled_back.push("Navicat".into());
            }
            "redis" => {
                let dir = format!("{install_root}\\redis");
                remove_dir_safe(&dir);
                rolled_back.push("Redis".into());
            }
            _ => {}
        }

        emit_done(app, comp, true, &format!("{comp} 已回滚"));
    }

    Ok(rolled_back)
}

fn remove_dir_safe(dir: &str) {
    if std::path::Path::new(dir).exists() {
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// 模拟测试模式：验证 Redis 本地 ZIP 是否可用（返回 Result 以统一走 install_component! 宏）。
fn check_redis_available(app: &AppHandle) -> Result<(), String> {
    emit_status(app, "redis", "download", "[测试模式] 正在检查 Redis 本地资源...");

    let resources = bundled::check_bundled_resources();
    let found = resources.iter().any(|(name, avail)| name == "redis" && *avail);

    if found {
        emit_done(app, "redis", true, "[测试模式] Redis 资源验证通过");
        Ok(())
    } else {
        Err("[测试模式] Redis 压缩包未找到".into())
    }
}

/// 模拟测试模式：仅执行下载验证。
async fn dry_run_download(
    app: &AppHandle,
    component: &str,
    version: &str,
    temp_dir: &str,
    on_progress: &Channel<DownloadProgress>,
) -> Result<(), String> {
    emit_status(
        app,
        component,
        "download",
        &format!("[测试模式] 正在下载 {component}..."),
    );
    crate::download::download_with_version(component, version, temp_dir, on_progress).await?;
    emit_status(
        app,
        component,
        "config",
        &format!("[测试模式] {component} 下载验证成功，跳过安装步骤"),
    );
    emit_done(
        app,
        component,
        true,
        &format!("[测试模式] {component} 下载验证通过"),
    );
    Ok(())
}

/// 将单个组件的安装结果录入结果列表。
fn run_install(
    app: &AppHandle,
    name: &str,
    result: Result<(), String>,
    results: &mut Vec<InstallResult>,
) {
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
