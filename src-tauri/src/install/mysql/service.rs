use crate::common::process::hide_window;
use crate::install::emit_status;
use crate::install::mysql::command::cmd_with_utf8;
use std::path::Path;
use std::process::Command;
use tauri::AppHandle;

/// 停止并删除旧的 MySQL Windows 服务。
pub fn cleanup_old_service(app: &AppHandle) {
    emit_status(app, "mysql", "config", "正在清理旧 MySQL 服务...");
    let _ = hide_window(Command::new("sc").args(["stop", "MySQL80"])).output();
    std::thread::sleep(std::time::Duration::from_secs(2));
    let _ = hide_window(Command::new("sc").args(["delete", "MySQL80"])).output();
    std::thread::sleep(std::time::Duration::from_secs(1));
}

/// 使用 `mysqld --initialize-insecure` 初始化数据目录。
pub fn initialize_data_dir(app: &AppHandle, mysql_home: &str) -> Result<(), String> {
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
pub fn register_service(app: &AppHandle, mysql_home: &str) -> Result<(), String> {
    emit_status(app, "mysql", "config", "正在注册 MySQL 系统服务...");

    let mysqld = format!("{mysql_home}\\bin\\mysqld.exe");
    let output = hide_window(Command::new(&mysqld).args(["--install", "MySQL80"]))
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
pub fn start_service(app: &AppHandle) -> Result<(), String> {
    emit_status(app, "mysql", "config", "正在启动 MySQL 服务...");

    for attempt in 1..=3 {
        if let Ok(o) = hide_window(Command::new("net").args(["start", "MySQL80"])).output() {
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
