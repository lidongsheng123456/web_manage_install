use serde::Serialize;

/// 单个组件的环境检测结果。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentStatus {
    /// 组件显示名称，如 "Node.js"、"MySQL"。
    pub name: String,
    /// 是否在系统中检测到。
    pub installed: bool,
    /// 检测到的实际版本号（未安装时为空）。
    pub version: String,
    /// 期望安装的版本描述。
    pub expected_version: String,
    /// 实际版本是否与期望版本匹配。
    pub version_match: bool,
}
