use crate::common::types::DownloadProgress;
use crate::download;
use crate::install::workflow::events::{emit_done, emit_status};
use tauri::ipc::Channel;
use tauri::AppHandle;

/// 模拟测试模式：仅执行下载验证，不做解压、安装和环境变量修改。
pub async fn dry_run_download(
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
    download::download_with_version(component, version, temp_dir, on_progress).await?;
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
