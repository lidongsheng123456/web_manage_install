//! Tomcat 安装器
//!
//! 下载 Windows ZIP → 解压到安装目录 → 设置 CATALINA_HOME 环境变量。

use crate::common::types::DownloadProgress;
use crate::common::version_policy::tomcat as tomcat_policy;
use crate::download;
use crate::install::{emit_done, emit_status, utils};
use crate::system::env_config;
use tauri::ipc::Channel;
use tauri::AppHandle;

pub async fn install(
    app: &AppHandle,
    install_root: &str,
    temp_dir: &str,
    version: &str,
    on_progress: &Channel<DownloadProgress>,
) -> Result<(), String> {
    let dir_name = tomcat_policy::install_dir_name(version);

    emit_status(
        app,
        "tomcat",
        "download",
        &format!("正在下载 Apache Tomcat {version}..."),
    );
    let zip_path =
        download::download_with_version("tomcat", version, temp_dir, on_progress).await?;

    emit_status(app, "tomcat", "install", "正在解压 Tomcat...");
    let target = utils::extract_and_move(&zip_path, install_root, "tomcat", &dir_name)?;

    emit_status(app, "tomcat", "config", "正在配置 CATALINA_HOME 环境变量...");
    env_config::set_system_env("CATALINA_HOME", &target)?;

    emit_done(
        app,
        "tomcat",
        true,
        &format!("Tomcat {version} 已安装到 {target}，CATALINA_HOME 已配置"),
    );
    Ok(())
}
