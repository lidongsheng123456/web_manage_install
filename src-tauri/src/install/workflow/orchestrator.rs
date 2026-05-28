use crate::common::types::{CancelToken, DownloadProgress, InstallConfig, InstallResult};
use crate::install::components::{bundled, jdk, maven, mysql, node};
use crate::install::workflow::cancel::check_cancel;
use crate::install::workflow::dry_run::dry_run_download;
use crate::install::workflow::events::emit_status;
use crate::install::workflow::privilege::is_elevated;
use crate::install::workflow::result::record_install_result;
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, Manager};

/// 统一安装入口：根据用户配置依次安装选中的组件。
///
/// 这里仅负责流程编排：取消检查、测试模式分支、结果汇总和事件收口；
/// 具体安装细节下沉到各组件安装器，避免命令入口承载业务实现。
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

    // 每个组件安装前都检查取消信号，已取消时不再启动后续安装任务。
    macro_rules! install_component {
        ($flag:expr, $name:expr, $install_expr:expr) => {
            if $flag && !cancelled {
                if check_cancel(&app).is_err() {
                    cancelled = true;
                    emit_status(&app, $name, "error", "安装已取消");
                } else {
                    record_install_result(&app, $name, $install_expr, &mut results);
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

    install_component!(
        config.install_idea,
        "idea",
        if dry {
            dry_run_download(&app, "idea", "2023.3.8", &temp, &on_progress).await
        } else {
            bundled::download_idea(&app, &root, &temp, &on_progress).await
        }
    );

    install_component!(
        config.install_navicat,
        "navicat",
        if dry {
            dry_run_download(&app, "navicat", "17", &temp, &on_progress).await
        } else {
            bundled::download_navicat(&app, &root, &temp, &on_progress).await
        }
    );

    install_component!(
        config.install_redis,
        "redis",
        if dry {
            dry_run_download(&app, "redis", "5.0.14.1", &temp, &on_progress).await
        } else {
            bundled::download_redis(&app, &root, &temp, &on_progress).await
        }
    );

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
