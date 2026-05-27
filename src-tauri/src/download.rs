//! 多镜像源下载模块
//!
//! 提供从国内镜像源下载组件安装包的功能，支持：
//! - 多镜像自动故障转移（清华 → npmmirror → 华为云 → 官方）
//! - 流式下载 + 实时进度推送（通过 Tauri Channel）
//! - 已下载文件缓存检测（>1MB 视为有效缓存）

use crate::types::{DownloadProgress, MirrorSource, PreflightResult};
use futures_util::StreamExt;
use std::path::Path;
use tauri::ipc::Channel;

/// 根据组件标识和用户选择的版本号，返回对应的国内镜像 URL 列表和下载文件名。
///
/// 镜像按速度和稳定性排序，国内源优先。
/// `version` 参数由前端版本选择器传入，支持多版本切换。
pub fn get_mirrors_versioned(component: &str, version: &str) -> MirrorSource {
    match component {
        "nodejs" => MirrorSource {
            urls: vec![
                format!("https://mirrors.tuna.tsinghua.edu.cn/nodejs-release/v{version}/node-v{version}-x64.msi"),
                format!("https://npmmirror.com/mirrors/node/v{version}/node-v{version}-x64.msi"),
                format!("https://nodejs.org/dist/v{version}/node-v{version}-x64.msi"),
            ],
            filename: format!("node-v{version}-x64.msi"),
        },
        "jdk" => {
            let (major, _minor) = parse_jdk_version(version);
            MirrorSource {
                urls: vec![
                    format!("https://repo.huaweicloud.com/openjdk/{major}.0.2/openjdk-{major}.0.2_windows-x64_bin.zip"),
                    format!("https://download.java.net/java/GA/jdk{major}.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-{major}.0.2_windows-x64_bin.zip"),
                    format!("https://github.com/adoptium/temurin{major}-binaries/releases/download/jdk-{major}.0.13%2B11/OpenJDK{major}U-jdk_x64_windows_hotspot_{major}.0.13_11.zip"),
                ],
                filename: format!("openjdk-{major}_windows-x64_bin.zip"),
            }
        },
        "mysql" => {
            let minor = version.rsplit('.').next().unwrap_or("36");
            let minor_num: u32 = minor.parse().unwrap_or(36);
            let mut urls = vec![
                format!("https://cdn.mysql.com/archives/mysql-8.0/mysql-{version}-winx64.zip"),
            ];
            if minor_num < 37 {
                urls.push("https://cdn.mysql.com/Downloads/MySQL-8.0/mysql-8.0.37-winx64.zip".into());
            }
            MirrorSource { urls, filename: format!("mysql-{version}-winx64.zip") }
        },
        "maven" => MirrorSource {
            urls: vec![
                format!("https://repo.huaweicloud.com/apache/maven/maven-3/{version}/binaries/apache-maven-{version}-bin.zip"),
                format!("https://archive.apache.org/dist/maven/maven-3/{version}/binaries/apache-maven-{version}-bin.zip"),
            ],
            filename: format!("apache-maven-{version}-bin.zip"),
        },
        _ => MirrorSource {
            urls: vec![],
            filename: String::new(),
        },
    }
}

/// 解析 JDK 版本号，提取主版本号。"17" → (17, ""), "21" → (21, "")
fn parse_jdk_version(ver: &str) -> (u32, &str) {
    let major: u32 = ver.split('.').next().and_then(|s| s.parse().ok()).unwrap_or(17);
    (major, ver)
}

/// 使用默认版本号获取镜像（兼容旧调用）
pub fn get_mirrors(component: &str) -> MirrorSource {
    let default_ver = match component {
        "nodejs" => "20.19.0",
        "jdk"    => "17",
        "maven"  => "3.9.6",
        "mysql"  => "8.0.36",
        _        => "",
    };
    get_mirrors_versioned(component, default_ver)
}

/// 下载指定组件的安装包，自动尝试多个镜像并实时推送进度。
///
/// - 如果目标文件已存在且 >1MB，直接返回缓存路径
/// - 依次尝试每个镜像 URL，成功即返回
/// - 所有镜像都失败时返回最后一个错误
/// 带版本号的下载入口，version 为空则使用默认版本。
pub async fn download_with_version(
    component: &str,
    version: &str,
    dest_dir: &str,
    on_progress: &Channel<DownloadProgress>,
) -> Result<String, String> {
    let mirrors = if version.is_empty() {
        get_mirrors(component)
    } else {
        get_mirrors_versioned(component, version)
    };
    if mirrors.urls.is_empty() {
        return Err(format!("未知组件: {component}"));
    }

    let dest_path = Path::new(dest_dir).join(&mirrors.filename);

    if let Some(cached) = check_cache(&dest_path, component, on_progress) {
        return Ok(cached);
    }

    std::fs::create_dir_all(dest_dir)
        .map_err(|e| format!("创建目录失败: {e}"))?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .connect_timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let mut last_error = String::new();

    for (idx, url) in mirrors.urls.iter().enumerate() {
        let _ = on_progress.send(DownloadProgress {
            component: component.into(),
            downloaded: 0, total: 0, percent: 0.0,
            speed: String::new(),
            status: format!("尝试镜像 {}/{}", idx + 1, mirrors.urls.len()),
        });

        match stream_download(&client, url, &dest_path, component, on_progress).await {
            Ok(()) => return Ok(dest_path.to_string_lossy().to_string()),
            Err(e) => {
                last_error = format!("镜像 {} 失败: {e}", idx + 1);
                let _ = std::fs::remove_file(&dest_path);
            }
        }
    }

    Err(format!("{component} 所有镜像均下载失败。最后错误: {last_error}"))
}

/// 检查是否存在有效的下载缓存。
///
/// 对各组件设置最小有效大小阈值（根据实际安装包大小），
/// 避免使用下载中断导致的不完整文件。
fn min_valid_size(component: &str) -> u64 {
    match component {
        "nodejs" => 20 * 1024 * 1024,   // Node.js MSI ~30MB
        "jdk"    => 150 * 1024 * 1024,   // OpenJDK ZIP ~180MB
        "maven"  => 5 * 1024 * 1024,     // Maven ZIP ~10MB
        "mysql"  => 200 * 1024 * 1024,   // MySQL ZIP ~400MB
        _        => 1024 * 1024,
    }
}

fn check_cache(
    dest_path: &Path,
    component: &str,
    on_progress: &Channel<DownloadProgress>,
) -> Option<String> {
    let meta = std::fs::metadata(dest_path).ok()?;
    if meta.len() < min_valid_size(component) {
        return None;
    }
    let _ = on_progress.send(DownloadProgress {
        component: component.into(),
        downloaded: meta.len(), total: meta.len(),
        percent: 100.0, speed: String::new(),
        status: "cached".into(),
    });
    Some(dest_path.to_string_lossy().to_string())
}

/// 从单个 URL 执行流式下载，每 100ms 推送一次进度。
async fn stream_download(
    client: &reqwest::Client,
    url: &str,
    dest_path: &Path,
    component: &str,
    on_progress: &Channel<DownloadProgress>,
) -> Result<(), String> {
    let response = client.get(url).send().await
        .map_err(|e| format!("请求失败: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let total = response.content_length().unwrap_or(0);
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut file = std::fs::File::create(dest_path)
        .map_err(|e| format!("创建文件失败: {e}"))?;

    let start = std::time::Instant::now();
    let mut last_emit = std::time::Instant::now();

    use std::io::Write;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("下载中断: {e}"))?;
        file.write_all(&chunk).map_err(|e| format!("写入失败: {e}"))?;
        downloaded += chunk.len() as u64;

        if last_emit.elapsed().as_millis() >= 100 || downloaded >= total {
            let elapsed = start.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 { downloaded as f64 / elapsed } else { 0.0 };
            let percent = if total > 0 { (downloaded as f64 / total as f64 * 100.0).min(100.0) } else { 0.0 };

            let _ = on_progress.send(DownloadProgress {
                component: component.into(),
                downloaded, total, percent,
                speed: format_speed(speed),
                status: "downloading".into(),
            });
            last_emit = std::time::Instant::now();
        }
    }

    file.flush().map_err(|e| format!("刷新失败: {e}"))?;
    Ok(())
}

/// 将字节/秒转为人类可读的速度字符串。
fn format_speed(bps: f64) -> String {
    if bps >= 1_048_576.0 { format!("{:.1} MB/s", bps / 1_048_576.0) }
    else if bps >= 1024.0 { format!("{:.0} KB/s", bps / 1024.0) }
    else { format!("{:.0} B/s", bps) }
}

/// 预检测试：用 HEAD 请求验证所有组件的镜像 URL 是否可达。
///
/// 不下载文件，仅检查 HTTP 状态码和 Content-Length，
/// 用于在不影响用户环境的情况下验证网络和链接有效性。
#[tauri::command]
pub async fn preflight_check() -> Result<Vec<PreflightResult>, String> {
    let components = ["nodejs", "jdk", "maven", "mysql"];
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let mut results = Vec::new();

    for comp in &components {
        let mirrors = get_mirrors(comp);
        for url in &mirrors.urls {
            let (reachable, status, size) = match client.head(url).send().await {
                Ok(resp) => {
                    let ok = resp.status().is_success();
                    let len = resp.content_length().unwrap_or(0);
                    (ok, format!("HTTP {}", resp.status().as_u16()), len)
                }
                Err(e) => {
                    let msg = if e.is_connect() { "连接失败".into() }
                        else if e.is_timeout() { "超时".into() }
                        else { format!("{e}") };
                    (false, msg, 0)
                }
            };
            results.push(PreflightResult {
                component: comp.to_string(),
                url: url.clone(),
                reachable,
                status,
                file_size: size,
            });
        }
    }

    Ok(results)
}
