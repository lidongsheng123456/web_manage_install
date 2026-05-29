use crate::common::version_policy::defaults;
use serde::{Deserialize, Serialize};

/// 安装过程中的状态事件，通过 Tauri Event 推送到前端。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallEvent {
    /// 组件标识。
    pub component: String,
    /// 当前阶段：download / install / config / complete / error。
    pub phase: String,
    /// 人类可读的状态消息。
    pub message: String,
    /// 当前步骤是否成功。
    pub success: bool,
    /// 该组件是否已完成（无论成功或失败）。
    pub done: bool,
}

/// 用户在 Step 1 配置的安装参数。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallConfig {
    /// 安装根目录，如 `D:\develop\software`。
    pub install_root: String,
    /// MySQL root 用户初始密码。
    pub mysql_password: String,
    /// 是否安装 Node.js。
    pub install_nodejs: bool,
    /// 是否安装 JDK。
    pub install_jdk: bool,
    /// 是否安装 Maven。
    pub install_maven: bool,
    /// 是否安装 MySQL。
    pub install_mysql: bool,
    /// 模拟测试模式：仅验证下载链接可用性，不执行实际安装。
    #[serde(default)]
    pub dry_run: bool,
    /// 用户选择的 Node.js 版本（如 "20.19.0"、"22.13.1"）。
    #[serde(default = "default_node_ver")]
    pub node_version: String,
    /// 用户选择的 JDK 版本（如 "17"、"21"）。
    #[serde(default = "default_jdk_ver")]
    pub jdk_version: String,
    /// 用户选择的 Maven 版本（如 "3.9.6"、"3.9.9"）。
    #[serde(default = "default_maven_ver")]
    pub maven_version: String,
    /// 用户选择的 MySQL 版本（如 "8.0.36"、"8.0.37"）。
    #[serde(default = "default_mysql_ver")]
    pub mysql_version: String,
    /// 是否安装 IntelliJ IDEA。
    #[serde(default)]
    pub install_idea: bool,
    /// 是否安装 Navicat Premium。
    #[serde(default)]
    pub install_navicat: bool,
    /// 是否解压 Redis。
    #[serde(default)]
    pub install_redis: bool,
}

fn default_node_ver() -> String {
    defaults::NODEJS.into()
}
fn default_jdk_ver() -> String {
    defaults::JDK.into()
}
fn default_maven_ver() -> String {
    defaults::MAVEN.into()
}
fn default_mysql_ver() -> String {
    defaults::MYSQL.into()
}

/// 单个组件的安装结果。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallResult {
    /// 组件标识。
    pub component: String,
    /// 是否安装成功。
    pub success: bool,
    /// 结果描述信息。
    pub message: String,
}
