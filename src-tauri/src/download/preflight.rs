use crate::common::types::PreflightResult;
use crate::download::service;

/// 预检测试：用 HEAD 请求验证所有组件的镜像 URL 是否可达。
///
/// 不下载文件，仅检查 HTTP 状态码和 Content-Length，
/// 用于在不影响用户环境的情况下验证网络和链接有效性。
#[tauri::command]
pub async fn preflight_check() -> Result<Vec<PreflightResult>, String> {
    let components = [
        "nodejs", "jdk", "maven", "mysql", "idea", "navicat", "redis",
    ];
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let mut results = Vec::new();
    for comp in &components {
        let mirrors = service::get_mirrors(comp);
        for url in &mirrors.urls {
            results.push(check_url(&client, comp, url).await);
        }
    }

    Ok(results)
}

async fn check_url(client: &reqwest::Client, component: &str, url: &str) -> PreflightResult {
    let (reachable, status, size) = match client.head(url).send().await {
        Ok(resp) => {
            let ok = resp.status().is_success();
            let len = resp.content_length().unwrap_or(0);
            (ok, format!("HTTP {}", resp.status().as_u16()), len)
        }
        Err(e) => {
            let msg = if e.is_connect() {
                "连接失败".into()
            } else if e.is_timeout() {
                "超时".into()
            } else {
                format!("{e}")
            };
            (false, msg, 0)
        }
    };

    PreflightResult {
        component: component.to_string(),
        url: url.to_string(),
        reachable,
        status,
        file_size: size,
    }
}
