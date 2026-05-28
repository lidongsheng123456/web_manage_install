//! MySQL 绿色版安装编排
//!
//! 该文件只保留 MySQL 安装主流程；路径校验、my.ini、服务注册、
//! 密码设置和环境变量配置分别拆到同级模块，保持单文件单职责。

use crate::common::types::DownloadProgress;
use crate::download;
use crate::install::mysql::config::write_my_ini;
use crate::install::mysql::env::configure_env_vars;
use crate::install::mysql::password::set_root_password;
use crate::install::mysql::path::{select_port, validate_install_path};
use crate::install::mysql::runtime::check_vcruntime;
use crate::install::mysql::service::{
    cleanup_old_service, initialize_data_dir, register_service, start_service,
};
use crate::install::utils;
use crate::install::{emit_done, emit_status};
use tauri::ipc::Channel;
use tauri::AppHandle;

/// 执行 MySQL 完整安装流程。
pub async fn install(
    app: &AppHandle,
    install_root: &str,
    temp_dir: &str,
    version: &str,
    mysql_password: &str,
    on_progress: &Channel<DownloadProgress>,
) -> Result<(), String> {
    validate_install_path(install_root)?;

    emit_status(
        app,
        "mysql",
        "download",
        &format!("正在下载 MySQL {version}..."),
    );
    let zip_path = download::download_with_version("mysql", version, temp_dir, on_progress).await?;

    emit_status(
        app,
        "mysql",
        "install",
        "正在解压 MySQL（约 300MB，请耐心等待）...",
    );
    let target = utils::extract_and_move(&zip_path, install_root, "mysql", "mysql")?;

    let port = select_port();
    write_my_ini(&target, port)?;

    check_vcruntime(app);
    cleanup_old_service(app);
    initialize_data_dir(app, &target)?;
    register_service(app, &target)?;
    start_service(app)?;
    set_root_password(app, &target, mysql_password);
    configure_env_vars(&target)?;

    emit_done(
        app,
        "mysql",
        true,
        &format!("MySQL {version} 安装完成 (端口: {port})"),
    );
    Ok(())
}
