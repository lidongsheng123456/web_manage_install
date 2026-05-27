//! DevEnv Installer - 开发环境可视化安装器
//!
//! 基于 Tauri 2 的 Windows 桌面应用，自动从国内镜像下载并安装
//! Node.js v20.19.0、JDK 17、Maven 3.9.6、MySQL 8.0.36。

mod detect;
mod download;
mod env_config;
mod installers;
mod types;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            detect::detect_environment,
            detect::verify::run_verify,
            installers::install_all,
            download::preflight_check,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
