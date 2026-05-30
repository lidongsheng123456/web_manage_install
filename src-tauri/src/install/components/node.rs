//! Node.js 安装器
//!
//! 通过 MSI 静默安装 Node.js 到指定目录，
//! 然后设置 NODE_HOME 环境变量并配置 npm 使用淘宝镜像。

use crate::common::process::hide_window;
use crate::common::types::DownloadProgress;
use crate::download;
use crate::install::{emit_done, emit_status};
use crate::system::env_config;
use std::path::Path;
use std::process::Command;
use tauri::ipc::Channel;
use tauri::AppHandle;

/// 执行 Node.js 完整安装流程：冲突清理 → 下载 → MSI 静默安装 → 环境变量 → npm 镜像。
pub async fn install(
    app: &AppHandle,
    install_root: &str,
    temp_dir: &str,
    version: &str,
    on_progress: &Channel<DownloadProgress>,
) -> Result<(), String> {
    // 安装前清理旧版本冲突：卸载旧 MSI、移除旧 PATH 条目、重置环境变量
    crate::install::conflict::resolve_conflicts(app, "nodejs")?;

    emit_status(
        app,
        "nodejs",
        "download",
        &format!("正在下载 Node.js v{version}..."),
    );
    let msi_path =
        download::download_with_version("nodejs", version, temp_dir, on_progress).await?;

    emit_status(app, "nodejs", "install", "正在静默安装 Node.js...");
    let node_dir = format!("{install_root}\\nodejs");
    std::fs::create_dir_all(&node_dir).ok();

    let output = hide_window(Command::new("msiexec").args([
        "/i",
        &msi_path,
        "/qn",
        "/norestart",
        &format!("INSTALLDIR={node_dir}"),
        "ADDLOCAL=ALL",
    ]))
    .output()
    .map_err(|e| format!("运行 msiexec 失败: {e}"))?;

    let exit_code = output.status.code().unwrap_or(-1);
    // msiexec exit 0 = 成功, 3010 = 成功但需要重启
    if exit_code != 0 && exit_code != 3010 {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Node.js MSI 安装失败 (exit {}): {stderr}",
            exit_code
        ));
    }

    std::thread::sleep(std::time::Duration::from_secs(3));

    emit_status(app, "nodejs", "config", "正在配置 NODE_HOME 环境变量...");
    env_config::set_system_env("NODE_HOME", &node_dir)?;
    env_config::append_to_path(&node_dir)?;

    emit_status(app, "nodejs", "config", "正在设置 npm 淘宝镜像...");
    let npm_cmd = format!("{node_dir}\\npm.cmd");
    if Path::new(&npm_cmd).exists() {
        let _ = hide_window(Command::new(&npm_cmd).args([
            "config",
            "set",
            "registry",
            "https://registry.npmmirror.com",
        ]))
        .output();
    }

    emit_done(app, "nodejs", true, &format!("Node.js v{version} 安装完成"));
    Ok(())
}
