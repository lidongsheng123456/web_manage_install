use crate::common::types::MirrorSource;

/// Node.js Windows MSI：腾讯云、清华、npmmirror、阿里云优先，官方源兜底。
pub fn mirrors(version: &str) -> MirrorSource {
    MirrorSource {
        urls: vec![
            format!("https://mirrors.cloud.tencent.com/nodejs-release/v{version}/node-v{version}-x64.msi"),
            format!("https://mirrors.tuna.tsinghua.edu.cn/nodejs-release/v{version}/node-v{version}-x64.msi"),
            format!("https://npmmirror.com/mirrors/node/v{version}/node-v{version}-x64.msi"),
            format!("https://mirrors.aliyun.com/nodejs-release/v{version}/node-v{version}-x64.msi"),
            format!("https://nodejs.org/dist/v{version}/node-v{version}-x64.msi"),
        ],
        filename: format!("node-v{version}-x64.msi"),
    }
}
