//! 环境检测模块
//!
//! 检测系统中 Node.js / JDK / Maven / MySQL 的安装状态和版本号。
//! 核心改进：所有检测均使用注册表实时 PATH 而非进程启动时的旧 PATH，
//! 确保安装后立即能检测到新组件。
//!
//! 模块结构：
//! - `env_reader` — 注册表环境变量实时读取 + 命令执行工具
//! - `node`       — Node.js 检测
//! - `jdk`        — JDK 检测
//! - `maven`      — Maven 检测
//! - `mysql`      — MySQL 检测（含服务/注册表/路径多级回退）
//! - `verify`     — Step 4 验证命令执行

pub mod env_reader;
mod jdk;
mod maven;
mod mysql;
mod node;
pub mod verify;

use crate::types::ComponentStatus;

/// 检测系统中所有目标组件的安装状态。
///
/// 使用用户选择的版本号作为期望版本来判断匹配，
/// 返回 4 个组件的检测结果，前端根据状态显示绿/黄/红指示灯。
#[tauri::command]
pub async fn detect_environment(
    node_version: Option<String>,
    jdk_version: Option<String>,
    maven_version: Option<String>,
    mysql_version: Option<String>,
) -> Result<Vec<ComponentStatus>, String> {
    let nv = node_version.unwrap_or_else(|| "20.19.0".into());
    let jv = jdk_version.unwrap_or_else(|| "17".into());
    let mv = maven_version.unwrap_or_else(|| "3.9.6".into());
    let myv = mysql_version.unwrap_or_else(|| "8.0".into());

    Ok(vec![
        node::detect(&nv),
        jdk::detect(&jv),
        maven::detect(&mv),
        mysql::detect(&myv),
    ])
}
