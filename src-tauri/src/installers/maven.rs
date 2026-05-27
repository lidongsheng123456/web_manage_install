//! Maven 3.9.6 安装器
//!
//! 下载 Maven ZIP 包，解压后自动写入 `settings.xml`，
//! 配置阿里云公共仓库镜像和 localRepository 路径，
//! 最后设置 MAVEN_HOME 和 PATH 环境变量。

use crate::download;
use crate::types::DownloadProgress;
use crate::env_config;
use crate::installers::{emit_done, emit_status};
use crate::installers::utils;
use tauri::ipc::Channel;
use tauri::AppHandle;

/// 执行 Maven 完整安装流程：下载 → ZIP 解压 → settings.xml → 环境变量。
pub async fn install(
    app: &AppHandle,
    install_root: &str,
    temp_dir: &str,
    version: &str,
    on_progress: &Channel<DownloadProgress>,
) -> Result<(), String> {
    emit_status(app, "maven", "download", &format!("正在下载 Maven {version}..."));
    let zip_path = download::download_with_version("maven", version, temp_dir, on_progress).await?;

    emit_status(app, "maven", "install", "正在解压 Maven...");
    let target = utils::extract_and_move(&zip_path, install_root, "maven", "maven")?;

    emit_status(app, "maven", "config", "正在写入 settings.xml...");
    write_settings_xml(&target)?;

    emit_status(app, "maven", "config", "正在配置 MAVEN_HOME 环境变量...");
    let maven_bin = format!("{target}\\bin");
    env_config::set_system_env("MAVEN_HOME", &target)?;
    env_config::append_to_path(&maven_bin)?;

    emit_done(app, "maven", true, &format!("Maven {version} 安装完成"));
    Ok(())
}

/// 写入 Maven settings.xml，配置阿里云镜像和本地仓库路径。
///
/// `localRepository` 路径根据用户实际安装路径动态生成。
fn write_settings_xml(maven_home: &str) -> Result<(), String> {
    let repo_dir = format!("{maven_home}\\mvn_repo");
    std::fs::create_dir_all(&repo_dir).ok();

    let settings = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<settings xmlns="http://maven.apache.org/SETTINGS/1.2.0"
          xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
          xsi:schemaLocation="http://maven.apache.org/SETTINGS/1.2.0 https://maven.apache.org/xsd/settings-1.2.0.xsd">
    <localRepository>{repo}</localRepository>
    <mirrors>
        <mirror>
            <id>aliyunmaven</id>
            <mirrorOf>*</mirrorOf>
            <name>阿里云公共仓库</name>
            <url>https://maven.aliyun.com/repository/public</url>
        </mirror>
    </mirrors>
</settings>"#,
        repo = repo_dir
    );

    let path = format!("{maven_home}\\conf\\settings.xml");
    std::fs::write(&path, settings.as_bytes())
        .map_err(|e| format!("写入 settings.xml 失败: {e}"))
}
