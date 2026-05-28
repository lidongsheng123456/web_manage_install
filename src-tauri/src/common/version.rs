use serde::{Deserialize, Serialize};

/// 前端版本下拉框的一条候选项。
///
/// `source` 用于提示版本列表来自国内镜像或官方元数据，
/// 便于排查“为什么某个版本没有出现”这类资源问题。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionOption {
    /// 传给安装流程的真实版本号，如 `20.19.0`、`17`、`3.9.6`。
    pub value: String,
    /// 展示给用户看的版本名称。
    pub label: String,
    /// 是否为项目默认推荐版本。
    #[serde(default)]
    pub default: bool,
    /// 是否为长期支持版本；不适用的组件保持 false。
    #[serde(default)]
    pub lts: bool,
    /// 版本清单来源说明，如“清华镜像”“Adoptium API”。
    pub source: String,
}

/// 四个核心环境的动态版本目录。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionCatalog {
    pub nodejs: Vec<VersionOption>,
    pub jdk: Vec<VersionOption>,
    pub maven: Vec<VersionOption>,
    pub mysql: Vec<VersionOption>,
}
