//! 组件命令。

use super::{idea, navicat};
use tauri::AppHandle;

/// 从前端命令激活IDEA。
#[tauri::command]
pub fn activate_idea(app: AppHandle) -> Result<String, String> {
    idea::activate(&app)?;
    Ok("IDEA 激活成功".into())
}

/// 从前端命令激活Navicat。
#[tauri::command]
pub fn activate_navicat(app: AppHandle) -> Result<String, String> {
    navicat::activate(&app)?;
    Ok("Navicat 激活成功".into())
}
