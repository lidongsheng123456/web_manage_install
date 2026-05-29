use crate::common::types::DownloadProgress;
use futures_util::StreamExt;
use std::io::Write;
use std::path::Path;
use tauri::ipc::Channel;

const MIN_DOWNLOAD_SPEED_AFTER_WARMUP: f64 = 128.0 * 1024.0;
const DOWNLOAD_SPEED_WARMUP_SECS: u64 = 20;

/// 从单个 URL 执行流式下载，每 100ms 推送一次进度。
pub async fn stream_download(
    client: &reqwest::Client,
    url: &str,
    dest_path: &Path,
    component: &str,
    on_progress: &Channel<DownloadProgress>,
) -> Result<(), String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let total = response.content_length().unwrap_or(0);
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut file = std::fs::File::create(dest_path).map_err(|e| format!("创建文件失败: {e}"))?;

    let start = std::time::Instant::now();
    let mut last_emit = std::time::Instant::now();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("下载中断: {e}"))?;
        if downloaded == 0 {
            validate_file_signature(url, &chunk)?;
        }
        file.write_all(&chunk)
            .map_err(|e| format!("写入失败: {e}"))?;
        downloaded += chunk.len() as u64;

        if last_emit.elapsed().as_millis() >= 100 || downloaded >= total {
            let elapsed = start.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 {
                downloaded as f64 / elapsed
            } else {
                0.0
            };
            if should_abort_slow_download(elapsed, speed, downloaded, total) {
                return Err(format!(
                    "下载速度过慢 ({})，自动尝试下一个源",
                    format_speed(speed)
                ));
            }
            let percent = if total > 0 {
                (downloaded as f64 / total as f64 * 100.0).min(100.0)
            } else {
                0.0
            };

            let _ = on_progress.send(DownloadProgress {
                component: component.into(),
                downloaded,
                total,
                percent,
                speed: format_speed(speed),
                status: "downloading".into(),
            });
            last_emit = std::time::Instant::now();
        }
    }

    file.flush().map_err(|e| format!("刷新失败: {e}"))?;
    Ok(())
}

/// 任意资源如果长时间低速，会占住当前源不切换；这里主动熔断并尝试下一个源。
fn should_abort_slow_download(
    elapsed_secs: f64,
    speed_bps: f64,
    downloaded: u64,
    total: u64,
) -> bool {
    total > 0
        && downloaded < total
        && elapsed_secs >= DOWNLOAD_SPEED_WARMUP_SECS as f64
        && speed_bps < MIN_DOWNLOAD_SPEED_AFTER_WARMUP
}

/// 校验首个数据块的文件头，避免把下载中转页 HTML 当成安装包保存。
fn validate_file_signature(url: &str, chunk: &[u8]) -> Result<(), String> {
    let lower = url.to_lowercase();
    if lower.contains(".exe") && !chunk.starts_with(b"MZ") {
        return Err("返回内容不是有效的 Windows EXE 文件".into());
    }
    if lower.contains(".zip") && !lower.contains("api.adoptium.net/") && !chunk.starts_with(b"PK") {
        return Err("返回内容不是有效的 ZIP 文件".into());
    }
    Ok(())
}

fn format_speed(bps: f64) -> String {
    if bps >= 1_048_576.0 {
        format!("{:.1} MB/s", bps / 1_048_576.0)
    } else if bps >= 1024.0 {
        format!("{:.0} KB/s", bps / 1024.0)
    } else {
        format!("{:.0} B/s", bps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_html_when_exe_expected() {
        let result = validate_file_signature(
            "https://example.com/navicat17_premium_cs_x64.exe",
            b"<!DOCTYPE html>",
        );
        assert!(result.is_err());
    }

    #[test]
    fn accepts_exe_signature() {
        let result = validate_file_signature(
            "https://example.com/navicat17_premium_cs_x64.exe",
            b"MZ\x90\x00",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn aborts_slow_download_after_warmup() {
        assert!(should_abort_slow_download(
            DOWNLOAD_SPEED_WARMUP_SECS as f64 + 1.0,
            MIN_DOWNLOAD_SPEED_AFTER_WARMUP - 1.0,
            1024,
            10 * 1024 * 1024,
        ));
    }
}
