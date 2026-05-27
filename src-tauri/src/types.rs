//! 公共类型定义模块
//!
//! 定义了前后端 IPC 通信使用的所有数据结构，
//! 统一使用 `camelCase` 序列化以匹配 JavaScript 命名习惯。

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 环境检测
// ---------------------------------------------------------------------------

/// 单个组件的环境检测结果
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentStatus {
    /// 组件显示名称，如 "Node.js"、"MySQL"
    pub name: String,
    /// 是否在系统中检测到
    pub installed: bool,
    /// 检测到的实际版本号（未安装时为空）
    pub version: String,
    /// 期望安装的版本描述
    pub expected_version: String,
    /// 实际版本是否与期望版本匹配
    pub version_match: bool,
}

// ---------------------------------------------------------------------------
// 下载进度
// ---------------------------------------------------------------------------

/// 下载进度事件，通过 Tauri Channel 实时推送到前端
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    /// 正在下载的组件标识（nodejs / jdk / maven / mysql）
    pub component: String,
    /// 已下载字节数
    pub downloaded: u64,
    /// 文件总大小（可能为 0 表示未知）
    pub total: u64,
    /// 下载百分比 0.0 ~ 100.0
    pub percent: f64,
    /// 格式化的下载速度，如 "2.5 MB/s"
    pub speed: String,
    /// 当前状态：downloading / cached / 尝试镜像 x/y
    pub status: String,
}

// ---------------------------------------------------------------------------
// 安装状态事件
// ---------------------------------------------------------------------------

/// 安装过程中的状态事件，通过 Tauri Event 推送到前端
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallEvent {
    /// 组件标识
    pub component: String,
    /// 当前阶段：download / install / config / complete / error
    pub phase: String,
    /// 人类可读的状态消息
    pub message: String,
    /// 当前步骤是否成功
    pub success: bool,
    /// 该组件是否已完成（无论成功或失败）
    pub done: bool,
}

// ---------------------------------------------------------------------------
// 安装配置（前端 → 后端）
// ---------------------------------------------------------------------------

/// 用户在 Step 1 配置的安装参数
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallConfig {
    /// 安装根目录，如 `D:\develop\software`
    pub install_root: String,
    /// MySQL root 用户初始密码
    pub mysql_password: String,
    /// 是否安装 Node.js
    pub install_nodejs: bool,
    /// 是否安装 JDK
    pub install_jdk: bool,
    /// 是否安装 Maven
    pub install_maven: bool,
    /// 是否安装 MySQL
    pub install_mysql: bool,
    /// 模拟测试模式：仅验证下载链接可用性，不执行实际安装
    #[serde(default)]
    pub dry_run: bool,
    /// 用户选择的 Node.js 版本（如 "20.19.0"、"22.13.1"）
    #[serde(default = "default_node_ver")]
    pub node_version: String,
    /// 用户选择的 JDK 版本（如 "17"、"21"）
    #[serde(default = "default_jdk_ver")]
    pub jdk_version: String,
    /// 用户选择的 Maven 版本（如 "3.9.6"、"3.9.9"）
    #[serde(default = "default_maven_ver")]
    pub maven_version: String,
    /// 用户选择的 MySQL 版本（如 "8.0.36"、"8.0.37"）
    #[serde(default = "default_mysql_ver")]
    pub mysql_version: String,
}

fn default_node_ver() -> String { "20.19.0".into() }
fn default_jdk_ver() -> String { "17".into() }
fn default_maven_ver() -> String { "3.9.6".into() }
fn default_mysql_ver() -> String { "8.0.36".into() }

/// 预检结果：单个镜像 URL 的连通性测试结果
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightResult {
    /// 组件标识
    pub component: String,
    /// 测试的镜像 URL
    pub url: String,
    /// 是否可达
    pub reachable: bool,
    /// HTTP 状态码或错误信息
    pub status: String,
    /// 文件大小（字节，0 表示未知）
    pub file_size: u64,
}

// ---------------------------------------------------------------------------
// 安装结果（后端 → 前端）
// ---------------------------------------------------------------------------

/// 单个组件的安装结果
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallResult {
    /// 组件标识
    pub component: String,
    /// 是否安装成功
    pub success: bool,
    /// 结果描述信息
    pub message: String,
}

// ---------------------------------------------------------------------------
// 镜像源配置
// ---------------------------------------------------------------------------

/// 组件的国内镜像下载源配置
pub struct MirrorSource {
    /// 按优先级排序的镜像 URL 列表（国内源在前）
    pub urls: Vec<String>,
    /// 下载后保存的文件名
    pub filename: String,
}
