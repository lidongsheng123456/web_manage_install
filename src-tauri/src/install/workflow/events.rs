use crate::common::types::InstallEvent;
use tauri::{AppHandle, Emitter};

/// 向前端推送安装阶段状态事件。
pub(crate) fn emit_status(app: &AppHandle, component: &str, phase: &str, msg: &str) {
    let _ = app.emit(
        "install-status",
        InstallEvent {
            component: component.into(),
            phase: phase.into(),
            message: msg.into(),
            success: true,
            done: false,
        },
    );
}

/// 向前端推送组件安装完成（成功或失败）事件。
pub(crate) fn emit_done(app: &AppHandle, component: &str, success: bool, msg: &str) {
    let _ = app.emit(
        "install-status",
        InstallEvent {
            component: component.into(),
            phase: if success { "complete" } else { "error" }.into(),
            message: msg.into(),
            success,
            done: true,
        },
    );
}
