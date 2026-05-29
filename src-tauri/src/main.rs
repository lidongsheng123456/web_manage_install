#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod common;
mod detection;
mod download;
mod install;
mod system;
mod version_catalog;

use common::types::CancelToken;

/// 前端调用此命令取消正在进行的安装。
#[tauri::command]
fn cancel_install(state: tauri::State<CancelToken>) {
    state.cancel();
}

/// 前端调用此命令回滚已安装的组件。
#[tauri::command]
async fn rollback_install(
    app: tauri::AppHandle,
    components: Vec<String>,
    install_root: String,
) -> Result<Vec<String>, String> {
    install::rollback(&app, &components, &install_root)
}

fn main() {
    download::configure_proxy_bypass();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(CancelToken::new())
        .invoke_handler(tauri::generate_handler![
            detection::detect_environment,
            detection::verify::run_verify,
            install::workflow::orchestrator::install_all,
            download::preflight::preflight_check,
            version_catalog::command::get_version_catalog,
            cancel_install,
            rollback_install,
            install::commands::activate_idea,
            install::commands::activate_navicat,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
