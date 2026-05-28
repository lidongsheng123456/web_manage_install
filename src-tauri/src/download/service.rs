use crate::common::types::DownloadProgress;
use crate::download::{cache, sources, stream};
use std::path::Path;
use tauri::ipc::Channel;

/// 根据组件标识和用户选择的版本号，返回对应的下载 URL 列表和文件名。
///
/// 镜像按速度和稳定性排序：可用中国源优先，无公开中国源的资源使用官方源兜底。
/// `version` 参数由前端版本选择器传入，支持多版本切换。
pub fn get_mirrors_versioned(component: &str, version: &str) -> crate::common::types::MirrorSource {
    sources::get_mirrors_versioned(component, version)
}

/// 使用默认版本号获取镜像。
pub fn get_mirrors(component: &str) -> crate::common::types::MirrorSource {
    sources::get_mirrors(component)
}

/// 下载指定组件的安装包，自动尝试多个镜像并实时推送进度。
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
    if let Some(cached) = cache::check_cache(&dest_path, component, on_progress) {
        return Ok(cached);
    }

    std::fs::create_dir_all(dest_dir).map_err(|e| format!("创建目录失败: {e}"))?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .connect_timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let mut last_error = String::new();
    for (idx, url) in mirrors.urls.iter().enumerate() {
        emit_attempt(component, idx, mirrors.urls.len(), on_progress);

        match stream::stream_download(&client, url, &dest_path, component, on_progress).await {
            Ok(()) => return Ok(dest_path.to_string_lossy().to_string()),
            Err(e) => {
                last_error = format!("镜像 {} 失败: {e}", idx + 1);
                emit_failure(component, &last_error, on_progress);
                let _ = std::fs::remove_file(&dest_path);
            }
        }
    }

    Err(format!(
        "{component} 所有镜像均下载失败。最后错误: {last_error}"
    ))
}

fn emit_attempt(
    component: &str,
    index: usize,
    total: usize,
    on_progress: &Channel<DownloadProgress>,
) {
    let _ = on_progress.send(DownloadProgress {
        component: component.into(),
        downloaded: 0,
        total: 0,
        percent: 0.0,
        speed: String::new(),
        status: format!("尝试镜像 {}/{}", index + 1, total),
    });
}

fn emit_failure(component: &str, message: &str, on_progress: &Channel<DownloadProgress>) {
    let _ = on_progress.send(DownloadProgress {
        component: component.into(),
        downloaded: 0,
        total: 0,
        percent: 0.0,
        speed: String::new(),
        status: message.to_string(),
    });
}
