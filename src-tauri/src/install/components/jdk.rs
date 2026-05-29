//! JDK (OpenJDK/Temurin) 安装器
//!
//! 按选择版本从国内镜像下载 JDK ZIP 包，解压到安装目录，
//! 然后设置 JAVA_HOME 和 PATH 环境变量。

use crate::common::types::DownloadProgress;
use crate::common::version_policy::jdk as jdk_policy;
use crate::download;
use crate::install::utils;
use crate::install::{emit_done, emit_status};
use crate::system::env_config;
use tauri::ipc::Channel;
use tauri::AppHandle;

/// 执行 JDK 完整安装流程：下载 → ZIP 解压 → JAVA_HOME 环境变量。
pub async fn install(
    app: &AppHandle,
    install_root: &str,
    temp_dir: &str,
    version: &str,
    on_progress: &Channel<DownloadProgress>,
) -> Result<(), String> {
    emit_status(
        app,
        "jdk",
        "download",
        &format!("正在下载 JDK {version}..."),
    );
    let zip_path = download::download_with_version("jdk", version, temp_dir, on_progress).await?;

    emit_status(app, "jdk", "install", "正在解压 JDK...");
    let major = jdk_policy::major_from_version(version);
    let target = utils::extract_and_move(
        &zip_path,
        install_root,
        "jdk",
        &jdk_policy::install_dir_name(&major),
    )?;

    emit_status(app, "jdk", "config", "正在配置 JAVA_HOME 环境变量...");
    let java_bin = format!("{target}\\bin");
    env_config::set_system_env("JAVA_HOME", &target)?;
    env_config::append_to_path(&java_bin)?;

    emit_done(app, "jdk", true, &format!("JDK {version} 安装完成"));
    Ok(())
}
