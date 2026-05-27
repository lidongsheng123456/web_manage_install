//! MySQL 绿色版安装器
//!
//! 从镜像下载 MySQL ZIP 绿色版，解压后完成：
//! 1. 校验安装路径（拒绝含非 ASCII 字符的路径，避免中文系统乱码问题）
//! 2. 生成 `my.ini` 配置文件（纯 ASCII，路径统一用正斜杠）
//! 3. 自动检测 3306 端口占用，必要时切换到 3307
//! 4. 停止并删除旧的 MySQL 服务
//! 5. `mysqld --initialize-insecure` 初始化数据目录（UTF-8 代码页）
//! 6. `mysqld --install` 注册 Windows 服务
//! 7. 启动服务并设置 root 密码
//! 8. 配置 MYSQL_HOME 和 PATH 环境变量

use crate::download;
use crate::env_config;
use crate::installers::utils;
use crate::installers::{emit_done, emit_status};
use crate::types::DownloadProgress;
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
    validate_install_path(install_root)?;

    emit_status(app, "mysql", "download", &format!("正在下载 MySQL {version}..."));
    let zip_path =
        download::download_with_version("mysql", version, temp_dir, on_progress).await?;

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

/// 校验安装路径：拒绝含非 ASCII 字符的路径。
///
/// MySQL 的 my.ini 解析器将反斜杠序列（\b \n \r \t \s）视为转义字符，
/// 且非 ASCII 路径（如中文）在 GBK 编码的 Windows 上可能导致 mysqld
/// 初始化失败。强制要求纯 ASCII 路径可避免这两类问题。
fn validate_install_path(path: &str) -> Result<(), String> {
    if !path.is_ascii() {
        return Err(format!(
            "安装路径包含非英文字符: {}\n\
             MySQL 不支持中文路径，请选择纯英文路径（如 D:\\develop\\software）",
            path
        ));
    }

    let dangerous_seqs = [r"\b", r"\n", r"\r", r"\t", r"\s", r"\0"];
    let lower = path.to_lowercase();
    for seq in &dangerous_seqs {
        if lower.contains(seq) {
            return Err(format!(
                "安装路径包含 MySQL 转义字符 '{seq}': {path}\n\
                 请避免路径中出现 \\b \\n \\r \\t \\s 等组合"
            ));
        }
    }
    Ok(())
}

/// 检查 Visual C++ 运行库是否已安装。
///
/// MySQL 8.0 依赖 VC++ 2015-2022 Redistributable (vcruntime140.dll)。
/// 缺少时 mysqld 会报 0xc000007b 或 DLL not found 错误。
fn check_vcruntime(app: &AppHandle) {
    let sys32 = std::env::var("SYSTEMROOT").unwrap_or_else(|_| r"C:\Windows".into());
    let dll = format!(r"{}\System32\vcruntime140.dll", sys32);
    if !Path::new(&dll).exists() {
        emit_status(
            app,
            "mysql",
            "config",
            "⚠ 未检测到 vcruntime140.dll (Visual C++ 运行库)，MySQL 可能无法启动。\
             请安装 Microsoft Visual C++ 2015-2022 Redistributable",
        );
    }
}

/// 检测 3306 端口是否被占用，被占用则使用 3307。
fn select_port() -> u16 {
    if std::net::TcpListener::bind(("127.0.0.1", 3306)).is_err() {
        3307
    } else {
        3306
    }
}

/// 生成 my.ini 配置文件。
///
/// - 路径统一使用正斜杠，避免 MySQL 解析反斜杠转义
/// - 显式指定 lc-messages-dir，避免中文系统找不到错误消息文件
/// - 显式设置字符集为 utf8mb4
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

/// 停止并删除旧的 MySQL Windows 服务。
fn cleanup_old_service(app: &AppHandle) {
    emit_status(app, "mysql", "config", "正在清理旧 MySQL 服务...");
    let _ = Command::new("sc").args(["stop", "MySQL80"]).output();
    std::thread::sleep(std::time::Duration::from_secs(2));
    let _ = Command::new("sc").args(["delete", "MySQL80"]).output();
    std::thread::sleep(std::time::Duration::from_secs(1));
}

/// 创建一个设置了 UTF-8 代码页的 Command，通过 cmd /C 执行。
///
/// 中文 Windows 默认代码页为 936 (GBK)，mysqld 的控制台输出可能包含
/// UTF-8 字符导致乱码或路径解析异常。通过 chcp 65001 切换到 UTF-8。
fn cmd_with_utf8(program: &str, args: &[&str]) -> Command {
    let args_str = std::iter::once(program.to_string())
        .chain(args.iter().map(|a| {
            if a.contains(' ') {
                format!("\"{}\"", a)
            } else {
                a.to_string()
            }
        }))
        .collect::<Vec<_>>()
        .join(" ");

    let mut cmd = Command::new("cmd");
    cmd.args(["/C", &format!("chcp 65001 >nul 2>&1 && {}", args_str)]);
    cmd
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

    let output = cmd_with_utf8(&mysqld, &["--initialize-insecure", "--console"])
        .output()
        .map_err(|e| format!("MySQL 初始化失败: {e}"))?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let combined = format!("{stdout}{stderr}");

    let init_ok = output.status.success()
        || combined.contains("root@localhost is created")
        || combined.contains("initializ");

    if !init_ok {
        return Err(format!(
            "MySQL 初始化失败（可能原因：路径含特殊字符或缺少 VC++ 运行库）: {stderr}"
        ));
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
            let stderr = String::from_utf8_lossy(&o.stderr);
            emit_status(
                app,
                "mysql",
                "config",
                &format!("启动服务重试 {attempt}/3: {}", stderr.trim()),
            );
        }
        std::thread::sleep(std::time::Duration::from_secs(3));
    }

    Err("MySQL 服务启动失败，请手动检查。常见原因：端口被占用、缺少 VC++ 运行库、路径含中文".into())
}

/// 设置 root 用户密码。
///
/// 优先尝试直接连接设置密码（initialize-insecure 模式下 root 无密码）；
/// 若失败则回退到 skip-grant-tables 安全模式：停服 → 无鉴权启动 →
/// FLUSH PRIVILEGES → ALTER USER → 杀进程 → 重启正常服务。
fn set_root_password(app: &AppHandle, mysql_home: &str, password: &str) {
    emit_status(app, "mysql", "config", "正在设置 root 密码...");
    let mysql_exe = format!("{mysql_home}\\bin\\mysql.exe");
    let mysqld_exe = format!("{mysql_home}\\bin\\mysqld.exe");

    let sql = format!(
        "ALTER USER 'root'@'localhost' IDENTIFIED BY '{}'; FLUSH PRIVILEGES;",
        password
    );

    let direct_ok = Command::new(&mysql_exe)
        .args(["-u", "root", "--skip-password", "-e", &sql])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if direct_ok {
        return;
    }

    emit_status(
        app,
        "mysql",
        "config",
        "直接设置密码失败，切换到安全模式重置...",
    );

    let _ = Command::new("net").args(["stop", "MySQL80"]).output();
    std::thread::sleep(std::time::Duration::from_secs(3));

    let child = Command::new(&mysqld_exe)
        .args(["--skip-grant-tables", "--shared-memory"])
        .spawn();

    let child = match child {
        Ok(c) => c,
        Err(e) => {
            emit_status(
                app,
                "mysql",
                "config",
                &format!("⚠ 安全模式启动失败: {e}，请手动设置 root 密码"),
            );
            return;
        }
    };

    std::thread::sleep(std::time::Duration::from_secs(5));

    let safe_sql = format!(
        "FLUSH PRIVILEGES; ALTER USER 'root'@'localhost' IDENTIFIED BY '{}';",
        password
    );
    let _ = Command::new(&mysql_exe)
        .args(["-u", "root", "-e", &safe_sql])
        .output();

    let pid = child.id();
    let _ = Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .output();
    std::thread::sleep(std::time::Duration::from_secs(2));

    let _ = Command::new("net").args(["start", "MySQL80"]).output();
    std::thread::sleep(std::time::Duration::from_secs(3));
}

/// 配置 MYSQL_HOME 和 PATH 环境变量。
fn configure_env_vars(mysql_home: &str) -> Result<(), String> {
    let mysql_bin = format!("{mysql_home}\\bin");
    env_config::set_system_env("MYSQL_HOME", mysql_home)?;
    env_config::append_to_path(&mysql_bin)
}
