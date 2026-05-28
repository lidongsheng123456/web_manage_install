const CHINA_SOURCE_DOMAINS: &[&str] = &[
    "mirrors.cloud.tencent.com",
    "mirrors.tuna.tsinghua.edu.cn",
    "npmmirror.com",
    "repo.huaweicloud.com",
    "mirrors.aliyun.com",
    "mirrors.huaweicloud.com",
    "mirrors.sustech.edu.cn",
    "mirrors.ustc.edu.cn",
    "download-cdn.clf.jetbrains.com.cn",
    "download-cdn.jetbrains.com.cn",
    "download.jetbrains.com.cn",
    "dn.navicat.com.cn",
    "navicat-installers.oss-cn-shanghai.aliyuncs.com",
];

/// 在应用启动时调用，将中国源域名追加到 NO_PROXY 环境变量。
///
/// reqwest 默认读取 ALL_PROXY/HTTP_PROXY/HTTPS_PROXY 走代理，
/// 同时读取 NO_PROXY 跳过指定域名。此函数确保中国源绕过代理直连，
/// 国际镜像（nodejs.org、github.com 等）继续走用户配置的代理。
pub fn configure_proxy_bypass() {
    let has_proxy = std::env::var("ALL_PROXY").is_ok()
        || std::env::var("all_proxy").is_ok()
        || std::env::var("HTTP_PROXY").is_ok()
        || std::env::var("http_proxy").is_ok()
        || std::env::var("HTTPS_PROXY").is_ok()
        || std::env::var("https_proxy").is_ok();

    if !has_proxy {
        return;
    }

    let existing = std::env::var("NO_PROXY")
        .or_else(|_| std::env::var("no_proxy"))
        .unwrap_or_default()
        .to_lowercase();

    let new_entries: Vec<&&str> = CHINA_SOURCE_DOMAINS
        .iter()
        .filter(|d| !existing.contains(&d.to_lowercase()))
        .collect();

    if new_entries.is_empty() {
        return;
    }

    let additions = new_entries
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let combined = if existing.is_empty() {
        additions
    } else {
        format!("{},{}", existing.trim_end_matches(','), additions)
    };

    std::env::set_var("NO_PROXY", &combined);
}
