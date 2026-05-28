use crate::common::types::InstallResult;
use crate::install::workflow::events::emit_done;
use tauri::AppHandle;

/// 将单个组件的安装结果录入结果列表。
pub fn record_install_result(
    app: &AppHandle,
    name: &str,
    result: Result<(), String>,
    results: &mut Vec<InstallResult>,
) {
    match result {
        Ok(()) => results.push(InstallResult {
            component: name.into(),
            success: true,
            message: format!("{name} 安装成功"),
        }),
        Err(e) => {
            emit_done(app, name, false, &e);
            results.push(InstallResult {
                component: name.into(),
                success: false,
                message: e,
            });
        }
    }
}
