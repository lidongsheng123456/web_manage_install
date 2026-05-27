//! 本地资源安装器
//!
//! 处理与应用一起分发的本地安装包（不需要下载）：
//! - IntelliJ IDEA Ultimate — EXE 静默安装 + 激活工具
//! - Navicat Premium — EXE 静默安装 + 激活工具
//! - Redis — ZIP 解压即用
//!
//! 所有资源文件会先复制到用户指定的安装路径下。

use crate::installers::{emit_done, emit_status};
use crate::installers::utils;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::AppHandle;

/// 所有 public/ 下需要管理的资源文件
const BUNDLED_FILES: &[(&str, &str)] = &[
    ("ideaIU-2023.3.8.exe", "IDEA 安装包"),
    ("Idea激活.7z", "IDEA 激活工具"),
    ("navicat162_premium_cs_x64.exe", "Navicat 安装包"),
    ("navicat激活.7z", "Navicat 激活工具"),
    ("Redis-x64-3.2.100.zip", "Redis 压缩包"),
];

const IDEA_FILENAME: &str = "ideaIU-2023.3.8.exe";
const IDEA_CRACK: &str = "Idea激活.7z";
const NAVICAT_FILENAME: &str = "navicat162_premium_cs_x64.exe";
const NAVICAT_CRACK: &str = "navicat激活.7z";
const REDIS_FILENAME: &str = "Redis-x64-3.2.100.zip";

/// 在多个候选位置搜索本地资源文件。
fn find_resource(filename: &str) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(filename));
            candidates.push(dir.join("resources").join(filename));

            let mut ancestor = dir.to_path_buf();
            for _ in 0..5 {
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

/// 检查本地资源文件可用性，返回每个资源的 (名称, 是否存在)。
#[tauri::command]
pub fn check_bundled_resources() -> Vec<(String, bool)> {
    vec![
        ("idea".into(), find_resource(IDEA_FILENAME).is_some()),
        ("navicat".into(), find_resource(NAVICAT_FILENAME).is_some()),
        ("redis".into(), find_resource(REDIS_FILENAME).is_some()),
    ]
}

// ─── 复制所有资源到安装目录 ─────────────────────────────────

/// 将所有可用的 public/ 资源文件复制到安装根目录。
///
/// 包括安装包和激活工具，方便用户在安装目录中直接找到。
/// 已存在且大小一致的文件会跳过。
pub fn copy_resources_to_root(app: &AppHandle, install_root: &str) -> Result<(), String> {
    std::fs::create_dir_all(install_root)
        .map_err(|e| format!("创建安装目录失败: {e}"))?;

    for (filename, label) in BUNDLED_FILES {
        if let Some(src) = find_resource(filename) {
            let dest = Path::new(install_root).join(filename);

            if dest.exists() {
                let src_size = std::fs::metadata(&src).map(|m| m.len()).unwrap_or(0);
                let dst_size = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
                if src_size == dst_size && src_size > 0 {
                    emit_status(
                        app,
                        "bundled",
                        "config",
                        &format!("{label} 已存在，跳过: {filename}"),
                    );
                    continue;
                }
            }

            emit_status(
                app,
                "bundled",
                "install",
                &format!("正在复制 {label}: {filename}..."),
            );

            std::fs::copy(&src, &dest)
                .map_err(|e| format!("复制 {filename} 失败: {e}"))?;
        }
    }

    emit_done(app, "bundled", true, "所有资源文件已复制到安装目录");
    Ok(())
}

// ─── IntelliJ IDEA 安装 ────────────────────────────────────

/// 静默安装 IntelliJ IDEA Ultimate，并复制激活工具到安装目录。
pub fn install_idea(app: &AppHandle, install_root: &str) -> Result<(), String> {
    let exe_in_root = Path::new(install_root).join(IDEA_FILENAME);
    let exe = if exe_in_root.exists() {
        exe_in_root
    } else {
        find_resource(IDEA_FILENAME)
            .ok_or("未找到 IDEA 安装包")?
    };

    let target = format!("{install_root}\\IDEA");
    emit_status(app, "idea", "install", "正在静默安装 IntelliJ IDEA（约需 2-5 分钟）...");

    let output = Command::new(exe.to_string_lossy().to_string())
        .args(["/S", &format!("/D={target}")])
        .output()
        .map_err(|e| format!("启动 IDEA 安装程序失败: {e}"))?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("IDEA 安装失败 (exit {code}): {stderr}"));
    }

    wait_for_install_complete(&target, "bin\\idea64.exe", 300)?;

    emit_done(app, "idea", true, &format!(
        "IntelliJ IDEA 2023.3.8 安装完成。激活工具: {install_root}\\{IDEA_CRACK}"
    ));
    Ok(())
}

// ─── Navicat 安装 ───────────────────────────────────────────

/// 静默安装 Navicat Premium，并复制激活工具到安装目录。
pub fn install_navicat(app: &AppHandle, install_root: &str) -> Result<(), String> {
    let exe_in_root = Path::new(install_root).join(NAVICAT_FILENAME);
    let exe = if exe_in_root.exists() {
        exe_in_root
    } else {
        find_resource(NAVICAT_FILENAME)
            .ok_or("未找到 Navicat 安装包")?
    };

    let target = format!("{install_root}\\Navicat");
    emit_status(
        app,
        "navicat",
        "install",
        "正在静默安装 Navicat Premium（约需 1-3 分钟）...",
    );

    let output = Command::new(exe.to_string_lossy().to_string())
        .args(["/S", &format!("/D={target}")])
        .output()
        .map_err(|e| format!("启动 Navicat 安装程序失败: {e}"))?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Navicat 安装失败 (exit {code}): {stderr}"));
    }

    wait_for_install_complete(&target, "navicat.exe", 180)?;

    emit_done(app, "navicat", true, &format!(
        "Navicat Premium 16.2 安装完成。激活工具: {install_root}\\{NAVICAT_CRACK}"
    ));
    Ok(())
}

// ─── Redis 解压 ─────────────────────────────────────────────

/// 解压 Redis ZIP 到安装目录（绿色免安装）。
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
