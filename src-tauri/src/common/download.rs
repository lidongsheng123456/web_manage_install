use serde::Serialize;

/// 下载进度事件，通过 Tauri Channel 实时推送到前端。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    /// 正在下载的组件标识（nodejs / jdk / maven / mysql）。
    pub component: String,
    /// 已下载字节数。
    pub downloaded: u64,
    /// 文件总大小（可能为 0 表示未知）。
    pub total: u64,
    /// 下载百分比 0.0 ~ 100.0。
    pub percent: f64,
    /// 格式化的下载速度，如 "2.5 MB/s"。
    pub speed: String,
    /// 当前状态：downloading / cached / 尝试镜像 x/y。
    pub status: String,
}

/// 预检结果：单个镜像 URL 的连通性测试结果。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightResult {
    /// 组件标识。
    pub component: String,
    /// 测试的镜像 URL。
    pub url: String,
    /// 是否可达。
    pub reachable: bool,
    /// HTTP 状态码或错误信息。
    pub status: String,
    /// 文件大小（字节，0 表示未知）。
    pub file_size: u64,
}

/// 组件的下载源配置。
///
/// URL 已按优先级排序：可用中国源在前，官方源用于最后兜底。
pub struct MirrorSource {
    /// 按优先级排序的镜像 URL 列表（可用中国源在前）。
    pub urls: Vec<String>,
    /// 下载后保存的文件名。
    pub filename: String,
}
