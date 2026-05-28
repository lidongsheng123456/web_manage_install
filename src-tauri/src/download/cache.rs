use crate::common::types::DownloadProgress;
use std::path::Path;
use tauri::ipc::Channel;

/// 检查是否存在有效的下载缓存。
///
/// 对各组件设置最小有效大小阈值，避免使用下载中断导致的不完整文件。
pub fn check_cache(
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
        downloaded: meta.len(),
        total: meta.len(),
        percent: 100.0,
        speed: String::new(),
        status: "cached".into(),
    });
    Some(dest_path.to_string_lossy().to_string())
}

fn min_valid_size(component: &str) -> u64 {
    match component {
        "nodejs" => 20 * 1024 * 1024,
        "jdk" => 150 * 1024 * 1024,
        "maven" => 5 * 1024 * 1024,
        "mysql" => 200 * 1024 * 1024,
        "idea" => 500 * 1024 * 1024,
        "navicat" => 50 * 1024 * 1024,
        "redis" => 3 * 1024 * 1024,
        _ => 1024 * 1024,
    }
}
