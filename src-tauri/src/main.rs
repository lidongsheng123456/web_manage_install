#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod common;
mod detection;
mod download;
mod install;
mod system;
mod version_catalog;

use common::types::CancelToken;

// Tauri 主入口，负责设置全局状态和注册命令。
fn main() {
    // 强制 WebView2 启用 GPU 硬件加速，解决部分 Intel/AMD 集显被 GPU blocklist
    // 屏蔽导致 CSS 动画卡顿（表现为云朵、状态灯等动效像放 PPT）的问题。
    // 必须在 tauri::Builder 之前设置，WebView2 初始化时读取该环境变量。
    std::env::set_var(
        "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
        "--ignore-gpu-blocklist --enable-gpu-rasterization --enable-zero-copy",
    );

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
            install::workflow::commands::cancel_install,
            install::workflow::commands::rollback_install,
            install::components::commands::activate_idea,
            install::components::commands::activate_navicat,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
