use crate::common::types::MirrorSource;

/// Redis Windows 社区包发布在 GitHub；未找到稳定中国镜像时，用加速入口优先。
pub fn mirrors(version: &str) -> MirrorSource {
    MirrorSource {
        urls: vec![
            format!("https://gh-proxy.com/https://github.com/tporadowski/redis/releases/download/v{version}/Redis-x64-{version}.zip"),
            format!("https://github.com/tporadowski/redis/releases/download/v{version}/Redis-x64-{version}.zip"),
        ],
        filename: format!("Redis-x64-{version}.zip"),
    }
}
