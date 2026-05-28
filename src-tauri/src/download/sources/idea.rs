use crate::common::types::MirrorSource;

/// IDEA 安装包：中国站会跳转中国区 CDN，因此把可用中国域名排在官方 CDN 前面。
pub fn mirrors(version: &str) -> MirrorSource {
    MirrorSource {
        urls: vec![
            format!("https://download.jetbrains.com/idea/ideaIU-{version}.exe"),
            format!("https://download-cdn.clf.jetbrains.com.cn/idea/ideaIU-{version}.exe"),
            format!("https://download-cdn.jetbrains.com.cn/idea/ideaIU-{version}.exe"),
            format!("https://download.jetbrains.com.cn/idea/ideaIU-{version}.exe"),
            format!("https://download-cdn.jetbrains.com/idea/ideaIU-{version}.exe"),
        ],
        filename: format!("ideaIU-{version}.exe"),
    }
}
