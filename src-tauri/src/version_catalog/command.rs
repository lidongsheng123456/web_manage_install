//! 版本目录 IPC 命令。

use crate::common::types::VersionCatalog;

/// 前端调用的 IPC：实时获取四个核心环境的版本下拉框数据。
#[tauri::command]
pub async fn get_version_catalog() -> Result<VersionCatalog, String> {
    super::service::load_catalog().await
}
