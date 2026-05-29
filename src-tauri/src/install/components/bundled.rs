//! 附加工具处理器
//!
//! IDEA / Navicat / Redis 均从网络下载到安装目录，不自动安装。
//! 用户可在安装器完成后自行双击运行安装程序。
//! 激活工具（idea-activation.7z / navicat-activation.7z）如果存在则一并复制。

use crate::common::types::DownloadProgress;
use crate::common::version_policy::defaults;
use crate::download;
use crate::install::{emit_done, emit_status};
use std::path::Path;
use tauri::ipc::Channel;
use tauri::AppHandle;

/// 下载 IntelliJ IDEA 安装包到安装目录（不自动安装）。
pub async fn download_idea(
    app: &AppHandle,
    install_root: &str,
    temp_dir: &str,
    on_progress: &Channel<DownloadProgress>,
) -> Result<(), String> {
    emit_status(
        app,
        "idea",
        "download",
        &format!("正在下载 IntelliJ IDEA {}...", defaults::IDEA),
    );
    let src =
        download::download_with_version("idea", defaults::IDEA, temp_dir, on_progress).await?;

    std::fs::create_dir_all(install_root).ok();
    let filename = Path::new(&src)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let dest = format!("{install_root}\\{filename}");
    std::fs::copy(&src, &dest).map_err(|e| format!("复制 IDEA 安装包失败: {e}"))?;

    emit_done(
        app,
        "idea",
        true,
        &format!("IDEA 安装包已下载到 {dest}，请手动双击安装"),
    );
    Ok(())
}

/// 下载 Navicat Premium 安装包到安装目录（不自动安装）。
pub async fn download_navicat(
    app: &AppHandle,
    install_root: &str,
    temp_dir: &str,
    on_progress: &Channel<DownloadProgress>,
) -> Result<(), String> {
    emit_status(
        app,
        "navicat",
        "download",
        &format!("正在下载 Navicat Premium {}...", defaults::NAVICAT),
    );
    let src = download::download_with_version("navicat", defaults::NAVICAT, temp_dir, on_progress)
        .await?;

    std::fs::create_dir_all(install_root).ok();
    let filename = Path::new(&src)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let dest = format!("{install_root}\\{filename}");
    std::fs::copy(&src, &dest).map_err(|e| format!("复制 Navicat 安装包失败: {e}"))?;

    emit_done(
        app,
        "navicat",
        true,
        &format!("Navicat 安装包已下载到 {dest}，请手动双击安装"),
    );
    Ok(())
}

/// 下载 Redis ZIP 到安装目录并解压（绿色免安装）。
pub async fn download_redis(
    app: &AppHandle,
    install_root: &str,
    temp_dir: &str,
    on_progress: &Channel<DownloadProgress>,
) -> Result<(), String> {
    emit_status(
        app,
        "redis",
        "download",
        &format!("正在下载 Redis {}...", defaults::REDIS),
    );
    let zip_path =
        download::download_with_version("redis", defaults::REDIS, temp_dir, on_progress).await?;

    emit_status(app, "redis", "install", "正在解压 Redis...");
    let target =
        crate::install::utils::extract_and_move(&zip_path, install_root, "redis", "redis")?;

    let server = format!("{target}\\redis-server.exe");
    if !Path::new(&server).exists() {
        return Err("Redis 解压完成但未找到 redis-server.exe".into());
    }

    emit_done(
        app,
        "redis",
        true,
        &format!("Redis 已解压到 {target}（绿色免安装，双击 redis-server.exe 启动）"),
    );
    Ok(())
}
