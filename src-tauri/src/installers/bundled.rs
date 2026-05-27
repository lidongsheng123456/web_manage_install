//! 附加工具安装器
//!
//! - IntelliJ IDEA / Navicat Premium — 从网络下载安装包 + 静默安装
//! - Redis — 从打包的 ZIP 解压即用
//! - 激活工具 (Idea激活.7z / navicat激活.7z) — 打包在 exe 中，复制到安装目录

use crate::download;
use crate::installers::{emit_done, emit_status};
use crate::installers::utils;
use crate::types::DownloadProgress;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::ipc::Channel;
use tauri::AppHandle;

const IDEA_CRACK: &str = "idea-activation.7z";
const NAVICAT_CRACK: &str = "navicat-activation.7z";
const REDIS_FILENAME: &str = "Redis-x64-3.2.100.zip";

/// 打包在 exe 内的小文件列表（随 tauri bundle resources 分发）
const BUNDLED_SMALL_FILES: &[(&str, &str)] = &[
    ("idea-activation.7z", "IDEA 激活工具"),
    ("navicat-activation.7z", "Navicat 激活工具"),
    ("Redis-x64-3.2.100.zip", "Redis 压缩包"),
];

/// 在多个候选位置搜索打包的本地资源文件。
fn find_resource(filename: &str) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(filename));
            candidates.push(dir.join("resources").join(filename));
            candidates.push(dir.join("_up_").join("public").join(filename));

            let mut ancestor = dir.to_path_buf();
            for _ in 0..6 {
                candidates.push(ancestor.join("public").join(filename));
                if !ancestor.pop() {
                    break;
                }
            }
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join(filename));
        candidates.push(cwd.join("public").join(filename));
        if let Some(parent) = cwd.parent() {
            candidates.push(parent.join("public").join(filename));
        }
    }

    candidates.into_iter().find(|p| p.exists())
}

// ─── 资源可用性检查 ─────────────────────────────────────────

/// 检查附加工具是否可安装。
/// IDEA/Navicat 从网络下载（始终可用），Redis 依赖本地 zip。
#[tauri::command]
pub fn check_bundled_resources() -> Vec<(String, bool)> {
    vec![
        ("idea".into(), true),
        ("navicat".into(), true),
        ("redis".into(), find_resource(REDIS_FILENAME).is_some()),
    ]
}

// ─── 复制小文件到安装目录 ────────────────────────────────────

/// 将打包的小文件（激活工具 + Redis）复制到安装根目录。
pub fn copy_bundled_to_root(app: &AppHandle, install_root: &str) -> Result<(), String> {
    std::fs::create_dir_all(install_root)
        .map_err(|e| format!("创建安装目录失败: {e}"))?;

    for (filename, label) in BUNDLED_SMALL_FILES {
        if let Some(src) = find_resource(filename) {
            let dest = Path::new(install_root).join(filename);

            if dest.exists() {
                let src_size = std::fs::metadata(&src).map(|m| m.len()).unwrap_or(0);
                let dst_size = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
                if src_size == dst_size && src_size > 0 {
                    emit_status(app, "bundled", "config", &format!("{label} 已存在，跳过"));
                    continue;
                }
            }

            emit_status(app, "bundled", "install", &format!("正在复制 {label}: {filename}..."));
            std::fs::copy(&src, &dest)
                .map_err(|e| format!("复制 {filename} 失败: {e}"))?;
        }
    }

    emit_done(app, "bundled", true, "本地资源文件已复制到安装目录");
    Ok(())
}

// ─── IntelliJ IDEA — 下载 + 安装 ───────────────────────────

/// 下载并静默安装 IntelliJ IDEA Ultimate。
pub async fn install_idea(
    app: &AppHandle,
    install_root: &str,
    temp_dir: &str,
    on_progress: &Channel<DownloadProgress>,
) -> Result<(), String> {
    emit_status(app, "idea", "download", "正在下载 IntelliJ IDEA 2023.3.8...");
    let exe_path = download::download_with_version("idea", "2023.3.8", temp_dir, on_progress).await?;

    let target = format!("{install_root}\\IDEA");
    emit_status(app, "idea", "install", "正在静默安装 IntelliJ IDEA（约需 2-5 分钟）...");

    let output = Command::new(&exe_path)
        .args(["/S", &format!("/D={target}")])
        .output()
        .map_err(|e| format!("启动 IDEA 安装程序失败: {e}"))?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("IDEA 安装失败 (exit {code}): {stderr}"));
    }

    wait_for_install_complete(&target, "bin\\idea64.exe", 300)?;

    let crack_msg = if find_resource(IDEA_CRACK).is_some() {
        format!("。激活工具已复制到: {install_root}\\{IDEA_CRACK}")
    } else {
        String::new()
    };
    emit_done(app, "idea", true, &format!("IntelliJ IDEA 2023.3.8 安装完成{crack_msg}"));
    Ok(())
}

// ─── Navicat — 下载 + 安装 ──────────────────────────────────

/// 下载并静默安装 Navicat Premium。
pub async fn install_navicat(
    app: &AppHandle,
    install_root: &str,
    temp_dir: &str,
    on_progress: &Channel<DownloadProgress>,
) -> Result<(), String> {
    emit_status(app, "navicat", "download", "正在下载 Navicat Premium 16.2...");
    let exe_path = download::download_with_version("navicat", "16.2", temp_dir, on_progress).await?;

    let target = format!("{install_root}\\Navicat");
    emit_status(app, "navicat", "install", "正在静默安装 Navicat Premium（约需 1-3 分钟）...");

    let output = Command::new(&exe_path)
        .args(["/S", &format!("/D={target}")])
        .output()
        .map_err(|e| format!("启动 Navicat 安装程序失败: {e}"))?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Navicat 安装失败 (exit {code}): {stderr}"));
    }

    wait_for_install_complete(&target, "navicat.exe", 180)?;

    let crack_msg = if find_resource(NAVICAT_CRACK).is_some() {
        format!("。激活工具已复制到: {install_root}\\{NAVICAT_CRACK}")
    } else {
        String::new()
    };
    emit_done(app, "navicat", true, &format!("Navicat Premium 16.2 安装完成{crack_msg}"));
    Ok(())
}

// ─── Redis — 本地解压 ───────────────────────────────────────

/// 解压 Redis ZIP 到安装目录（绿色免安装），使用打包的本地资源。
pub fn install_redis(app: &AppHandle, install_root: &str) -> Result<(), String> {
    let zip_in_root = Path::new(install_root).join(REDIS_FILENAME);
    let zip = if zip_in_root.exists() {
        zip_in_root
    } else {
        find_resource(REDIS_FILENAME)
            .ok_or("未找到 Redis 压缩包")?
    };

    emit_status(app, "redis", "install", "正在解压 Redis...");

    let zip_str = zip.to_string_lossy().to_string();
    let target = utils::extract_and_move(&zip_str, install_root, "redis", "redis")?;

    let server = format!("{target}\\redis-server.exe");
    if !Path::new(&server).exists() {
        return Err("Redis 解压完成但未找到 redis-server.exe，请检查压缩包内容".into());
    }

    emit_done(
        app,
        "redis",
        true,
        &format!("Redis 3.2.100 已解压到 {target}（绿色免安装）"),
    );
    Ok(())
}

// ─── 工具函数 ───────────────────────────────────────────────

fn wait_for_install_complete(
    target_dir: &str,
    check_file: &str,
    max_secs: u64,
) -> Result<(), String> {
    let check = format!("{target_dir}\\{check_file}");
    let start = std::time::Instant::now();
    loop {
        if Path::new(&check).exists() {
            return Ok(());
        }
        if start.elapsed().as_secs() > max_secs {
            if Path::new(target_dir).is_dir() {
                return Ok(());
            }
            return Err(format!(
                "安装超时（等待 {max_secs}s），目标文件未出现: {check}"
            ));
        }
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}
