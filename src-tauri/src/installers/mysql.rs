//! MySQL 8.0.36 安装器
//!
//! 从国内镜像下载 MySQL ZIP 绿色版，解压后完成：
//! 1. 生成 `my.ini` 配置文件（ASCII 编码，避免 BOM）
//! 2. 自动检测 3306 端口占用，必要时切换到 3307
//! 3. 停止并删除旧的 MySQL80 服务
//! 4. `mysqld --initialize-insecure` 初始化数据目录
//! 5. `mysqld --install MySQL80` 注册 Windows 服务
//! 6. 启动服务并设置 root 密码
//! 7. 配置 MYSQL_HOME 和 PATH 环境变量

use crate::download;
use crate::types::DownloadProgress;
use crate::env_config;
use crate::installers::{emit_done, emit_status};
use crate::installers::utils;
use std::path::Path;
use std::process::Command;
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
    emit_status(app, "mysql", "download", &format!("正在下载 MySQL {version}..."));
    let zip_path = download::download_with_version("mysql", version, temp_dir, on_progress).await?;

    emit_status(app, "mysql", "install", "正在解压 MySQL（约 300MB，请耐心等待）...");
    let target = utils::extract_and_move(&zip_path, install_root, "mysql", "mysql")?;

    let port = select_port();
    write_my_ini(&target, port)?;

    cleanup_old_service(app);
    initialize_data_dir(app, &target)?;
    register_service(app, &target)?;
    start_service(app)?;
    set_root_password(app, &target, mysql_password);
    configure_env_vars(&target)?;

    emit_done(app, "mysql", true, &format!("MySQL {version} 安装完成 (端口: {port})"));
    Ok(())
}

/// 检测 3306 端口是否被占用，被占用则使用 3307。
fn select_port() -> u16 {
    if std::net::TcpListener::bind(("127.0.0.1", 3306)).is_err() { 3307 } else { 3306 }
}

/// 生成 my.ini 配置文件。
///
/// 注意：每行不能有前导空格，否则 mysqld 无法解析。
/// 使用 `\r\n` 行尾确保 Windows 兼容。
fn write_my_ini(mysql_home: &str, port: u16) -> Result<(), String> {
    let base = mysql_home.replace('\\', "/");
    let data = format!("{base}/data");

    let lines = [
        "[mysqld]",
        &format!("port={port}"),
        &format!("basedir={base}"),
        &format!("datadir={data}"),
        "max_connections=200",
        "character-set-server=utf8mb4",
        "default-storage-engine=INNODB",
        "default_authentication_plugin=mysql_native_password",
        "",
        "[mysql]",
        "default-character-set=utf8mb4",
        "",
        "[client]",
        &format!("port={port}"),
        "default-character-set=utf8mb4",
    ];

    let content = lines.join("\r\n");
    let path = format!("{mysql_home}\\my.ini");
    std::fs::write(&path, content.as_bytes())
        .map_err(|e| format!("写入 my.ini 失败: {e}"))
}

/// 停止并删除旧的 MySQL80 Windows 服务。
fn cleanup_old_service(app: &AppHandle) {
    emit_status(app, "mysql", "config", "正在清理旧 MySQL 服务...");
    let _ = Command::new("sc").args(["stop", "MySQL80"]).output();
    std::thread::sleep(std::time::Duration::from_secs(2));
    let _ = Command::new("sc").args(["delete", "MySQL80"]).output();
    std::thread::sleep(std::time::Duration::from_secs(1));
}

/// 使用 `mysqld --initialize-insecure` 初始化数据目录。
fn initialize_data_dir(app: &AppHandle, mysql_home: &str) -> Result<(), String> {
    emit_status(app, "mysql", "config", "正在初始化 MySQL 数据目录...");

    let data_dir = format!("{mysql_home}\\data");
    if Path::new(&data_dir).exists() {
        std::fs::remove_dir_all(&data_dir).ok();
    }
    std::fs::create_dir_all(&data_dir).ok();

    let mysqld = format!("{mysql_home}\\bin\\mysqld.exe");
    if !Path::new(&mysqld).exists() {
        return Err(format!("mysqld.exe 不存在: {mysqld}"));
    }

    let output = Command::new(&mysqld)
        .args(["--initialize-insecure", "--console"])
        .output()
        .map_err(|e| format!("MySQL 初始化失败: {e}"))?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let combined = format!("{stdout}{stderr}");

    // mysqld --initialize-insecure 成功时在 stderr 输出日志，
    // 关键标志是包含 "root@localhost is created" 或 "initializ" 字样
    let init_ok = output.status.success()
        || combined.contains("root@localhost is created")
        || combined.contains("initializ");

    if !init_ok {
        return Err(format!("MySQL 初始化失败: {stderr}"));
    }
    Ok(())
}

/// 将 mysqld 注册为 Windows 服务 MySQL80。
fn register_service(app: &AppHandle, mysql_home: &str) -> Result<(), String> {
    emit_status(app, "mysql", "config", "正在注册 MySQL 系统服务...");

    let mysqld = format!("{mysql_home}\\bin\\mysqld.exe");
    let output = Command::new(&mysqld)
        .args(["--install", "MySQL80"])
        .output()
        .map_err(|e| format!("注册服务失败: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("already exists") {
            return Err(format!("MySQL 服务注册失败: {stderr}"));
        }
    }
    Ok(())
}

/// 启动 MySQL80 服务，最多重试 3 次。
fn start_service(app: &AppHandle) -> Result<(), String> {
    emit_status(app, "mysql", "config", "正在启动 MySQL 服务...");

    for attempt in 1..=3 {
        if let Ok(o) = Command::new("net").args(["start", "MySQL80"]).output() {
            if o.status.success() {
                std::thread::sleep(std::time::Duration::from_secs(2));
                return Ok(());
            }
        }
        emit_status(app, "mysql", "config", &format!("启动服务重试 {attempt}/3..."));
        std::thread::sleep(std::time::Duration::from_secs(3));
    }

    Err("MySQL 服务启动失败，请手动检查".into())
}

/// 设置 root 用户密码（初始化时使用 --initialize-insecure 无密码）。
fn set_root_password(app: &AppHandle, mysql_home: &str, password: &str) {
    emit_status(app, "mysql", "config", "正在设置 root 密码...");
    let mysql_exe = format!("{mysql_home}\\bin\\mysql.exe");
    let sql = format!("ALTER USER 'root'@'localhost' IDENTIFIED BY '{password}';");
    let _ = Command::new(&mysql_exe)
        .args(["-u", "root", "--skip-password", "-e", &sql])
        .output();
}

/// 配置 MYSQL_HOME 和 PATH 环境变量。
fn configure_env_vars(mysql_home: &str) -> Result<(), String> {
    let mysql_bin = format!("{mysql_home}\\bin");
    env_config::set_system_env("MYSQL_HOME", mysql_home)?;
    env_config::append_to_path(&mysql_bin)
}
