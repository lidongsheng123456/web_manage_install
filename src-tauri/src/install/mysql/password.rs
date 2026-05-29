use crate::common::process::hide_window;
use crate::install::emit_status;
use std::process::Command;
use tauri::AppHandle;

/// 设置 root 用户密码。
///
/// 优先尝试直接连接设置密码；若失败则回退到 skip-grant-tables 模式。
pub fn set_root_password(app: &AppHandle, mysql_home: &str, password: &str, service_name: &str) {
    emit_status(app, "mysql", "config", "正在设置 root 密码...");
    let mysql_exe = format!("{mysql_home}\\bin\\mysql.exe");
    let mysqld_exe = format!("{mysql_home}\\bin\\mysqld.exe");

    let sql = format!(
        "ALTER USER 'root'@'localhost' IDENTIFIED BY '{}'; FLUSH PRIVILEGES;",
        password
    );

    let direct_ok =
        hide_window(Command::new(&mysql_exe).args(["-u", "root", "--skip-password", "-e", &sql]))
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
    reset_password_with_skip_grants(app, &mysql_exe, &mysqld_exe, password, service_name);
}

fn reset_password_with_skip_grants(
    app: &AppHandle,
    mysql_exe: &str,
    mysqld_exe: &str,
    password: &str,
    service_name: &str,
) {
    let _ = hide_window(Command::new("net").args(["stop", service_name])).output();
    std::thread::sleep(std::time::Duration::from_secs(3));

    let child =
        hide_window(Command::new(mysqld_exe).args(["--skip-grant-tables", "--shared-memory"]))
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
    let _ = hide_window(Command::new(mysql_exe).args(["-u", "root", "-e", &safe_sql])).output();

    let pid = child.id();
    let _ = hide_window(Command::new("taskkill").args(["/F", "/PID", &pid.to_string()])).output();
    std::thread::sleep(std::time::Duration::from_secs(2));

    let _ = hide_window(Command::new("net").args(["start", service_name])).output();
    std::thread::sleep(std::time::Duration::from_secs(3));
}
