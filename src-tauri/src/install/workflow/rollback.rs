use crate::common::process::hide_window;
use crate::install::{emit_done, emit_status};
use crate::system::env_config;
use std::process::Command;
use tauri::AppHandle;

/// 回滚已安装的组件：删除安装目录、移除环境变量、清理服务。
pub fn rollback(
    app: &AppHandle,
    components: &[String],
    install_root: &str,
) -> Result<Vec<String>, String> {
    let mut rolled_back = Vec::new();

    for comp in components {
        emit_status(app, comp, "config", &format!("正在回滚 {comp}..."));

        match comp.as_str() {
            "nodejs" => rollback_node(install_root, &mut rolled_back),
            "jdk" => rollback_jdk(install_root, &mut rolled_back),
            "maven" => rollback_maven(install_root, &mut rolled_back),
            "mysql" => rollback_mysql(install_root, &mut rolled_back),
            "idea" => rollback_dir(install_root, "IDEA", "IDEA", &mut rolled_back),
            "navicat" => rollback_dir(install_root, "Navicat", "Navicat", &mut rolled_back),
            "redis" => rollback_dir(install_root, "redis", "Redis", &mut rolled_back),
            _ => {}
        }

        emit_done(app, comp, true, &format!("{comp} 已回滚"));
    }

    Ok(rolled_back)
}

fn rollback_node(install_root: &str, rolled_back: &mut Vec<String>) {
    let dir = format!("{install_root}\\nodejs");
    env_config::remove_from_path(&dir);
    env_config::remove_env("NODE_HOME");
    remove_dir_safe(&dir);
    rolled_back.push("Node.js".into());
}

fn rollback_jdk(install_root: &str, rolled_back: &mut Vec<String>) {
    for major in ["17", "21"] {
        let dir = format!("{install_root}\\jdk{major}");
        if std::path::Path::new(&dir).exists() {
            env_config::remove_from_path(&format!("{dir}\\bin"));
            remove_dir_safe(&dir);
        }
    }
    env_config::remove_env("JAVA_HOME");
    rolled_back.push("JDK".into());
}

fn rollback_maven(install_root: &str, rolled_back: &mut Vec<String>) {
    let dir = format!("{install_root}\\maven");
    env_config::remove_from_path(&format!("{dir}\\bin"));
    env_config::remove_env("MAVEN_HOME");
    remove_dir_safe(&dir);
    rolled_back.push("Maven".into());
}

fn rollback_mysql(install_root: &str, rolled_back: &mut Vec<String>) {
    let _ = hide_window(Command::new("net").args(["stop", "MySQL80"])).output();
    std::thread::sleep(std::time::Duration::from_secs(2));
    let _ = hide_window(Command::new("sc").args(["delete", "MySQL80"])).output();

    let dir = format!("{install_root}\\mysql");
    env_config::remove_from_path(&format!("{dir}\\bin"));
    env_config::remove_env("MYSQL_HOME");
    remove_dir_safe(&dir);
    rolled_back.push("MySQL".into());
}

fn rollback_dir(install_root: &str, dir_name: &str, label: &str, rolled_back: &mut Vec<String>) {
    remove_dir_safe(&format!("{install_root}\\{dir_name}"));
    rolled_back.push(label.into());
}

fn remove_dir_safe(dir: &str) {
    if std::path::Path::new(dir).exists() {
        let _ = std::fs::remove_dir_all(dir);
    }
}
