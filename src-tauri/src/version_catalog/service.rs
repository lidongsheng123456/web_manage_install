//! 版本目录业务编排。
//!
//! 这里不做缓存，也不补硬编码版本；所有版本都来自实时 HTTP 请求。

use crate::common::types::VersionCatalog;
use crate::version_catalog::{jdk, maven, mysql, nodejs, tomcat};

/// 实时获取核心环境版本。任一核心组件失败时返回错误，避免展示半真半假的版本列表。
pub async fn load_catalog() -> Result<VersionCatalog, String> {
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(8))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("创建版本请求客户端失败: {e}"))?;

    Ok(VersionCatalog {
        nodejs: nodejs::load(&client).await?,
        jdk: jdk::load(&client).await?,
        maven: maven::load(&client).await?,
        mysql: mysql::load(&client).await?,
        tomcat: tomcat::load(&client).await?,
    })
}
