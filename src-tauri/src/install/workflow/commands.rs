//! 安装流程命令。

use super::rollback;
use crate::common::types::CancelToken;
use tauri::{AppHandle, State};

/// 取消当前运行的安装工作流。
#[tauri::command]
pub fn cancel_install(state: State<CancelToken>) {
    state.cancel();
}

/// 回滚已安装的组件。
#[tauri::command]
pub async fn rollback_install(
    app: AppHandle,
    components: Vec<String>,
    install_root: String,
) -> Result<Vec<String>, String> {
    rollback::rollback(&app, &components, &install_root)
}
