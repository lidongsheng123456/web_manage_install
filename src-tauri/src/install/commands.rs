//! 安装模块对外暴露的 Tauri 命令入口。
//!
//! 仅负责命令签名适配和调度，具体业务逻辑下沉到各组件安装器。

use super::components::{idea, navicat};
use tauri::AppHandle;

/// 前端点击按钮触发 IDEA 激活。
#[tauri::command]
pub fn activate_idea(app: AppHandle) -> Result<String, String> {
    idea::activate(&app)?;
    Ok("IDEA 激活成功".into())
}

/// 前端点击按钮触发 Navicat 激活。
#[tauri::command]
pub fn activate_navicat(app: AppHandle) -> Result<String, String> {
    navicat::activate(&app)?;
    Ok("Navicat 激活成功".into())
}
