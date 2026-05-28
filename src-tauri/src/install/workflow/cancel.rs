use crate::common::types::CancelToken;
use tauri::{AppHandle, Manager};

/// 组件安装前统一检查取消信号，避免每个安装器重复读取状态。
pub fn check_cancel(app: &AppHandle) -> Result<(), String> {
    let cancel = app.state::<CancelToken>();
    if cancel.is_cancelled() {
        Err("用户取消安装".into())
    } else {
        Ok(())
    }
}
