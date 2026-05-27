//! DevEnv Installer - 开发环境可视化安装器
//!
//! 基于 Tauri 2 的 Windows 桌面应用，自动从国内镜像下载并安装
//! Node.js、JDK、Maven、MySQL，以及本地打包的 IDEA、Navicat、Redis。

mod detect;
mod download;
mod env_config;
mod installers;
mod types;

use types::CancelToken;

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
    installers::rollback(&app, &components, &install_root)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    download::configure_proxy_bypass();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(CancelToken::new())
        .invoke_handler(tauri::generate_handler![
            detect::detect_environment,
            detect::verify::run_verify,
            installers::install_all,
            download::preflight_check,
            cancel_install,
            rollback_install,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
