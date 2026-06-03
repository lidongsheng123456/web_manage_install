use crate::common::types::{CancelToken, DownloadProgress, InstallConfig, InstallResult};
use crate::install::workflow::privilege::is_elevated;
use crate::install::workflow::task;
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, Manager};

/// 统一安装入口：根据用户配置并发安装选中的组件。
///
/// 这里仅负责流程编排：初始化、权限检查、委托并发任务执行、结果汇总和事件收口；
/// 具体安装细节下沉到各组件安装器，并发调度由 task 模块承担。
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
        crate::install::emit_status(
            &app,
            "nodejs",
            "config",
            "提示: 未以管理员身份运行，环境变量将写入用户级别（HKCU）",
        );
    }

    let results = task::run_all_concurrent(&app, &config, &on_progress).await;

    if !config.dry_run {
        let _ = std::fs::remove_dir_all(&temp);
    }

    let has_cancelled = results.iter().any(|r| r.message.contains("已取消"));
    if has_cancelled {
        let completed: Vec<String> = results
            .iter()
            .filter(|r| r.success)
            .map(|r| r.component.clone())
            .collect();
        let _ = app.emit(
            "install-cancelled",
            serde_json::json!({
                "completedComponents": completed,
                "message": "安装已取消，可选择回滚已完成的组件"
            }),
        );
    }

    let _ = app.emit("install-complete", &results);
    Ok(results)
}
