use crate::common::types::MirrorSource;

/// Maven ZIP：华为云和中科大镜像优先，Apache archive 兜底。
pub fn mirrors(version: &str) -> MirrorSource {
    MirrorSource {
        urls: vec![
            format!("https://repo.huaweicloud.com/apache/maven/maven-3/{version}/binaries/apache-maven-{version}-bin.zip"),
            format!("https://mirrors.ustc.edu.cn/apache/maven/maven-3/{version}/binaries/apache-maven-{version}-bin.zip"),
            format!("https://archive.apache.org/dist/maven/maven-3/{version}/binaries/apache-maven-{version}-bin.zip"),
        ],
        filename: format!("apache-maven-{version}-bin.zip"),
    }
}
