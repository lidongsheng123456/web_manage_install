//! 验证命令执行模块
//!
//! Step 4 结果页的 "点击验证" 功能。
//! 使用实时注册表 PATH + 环境变量执行白名单命令，
//! 带重试机制，确保刚安装完的工具能被检测到。

use super::env_reader::{build_fresh_path, read_fresh_env_vars};
use std::process::Command;

/// 允许前端执行的验证命令白名单
const ALLOWED_COMMANDS: &[(&str, &str, &[&str])] = &[
    ("node -v", "node", &["-v"]),
    ("java -version", "java", &["-version"]),
    ("mvn -v", "mvn", &["-v"]),
    ("mysql -V", "mysql", &["-V"]),
];

/// 运行一条验证命令并返回输出文本，带自动重试。
#[tauri::command]
pub async fn run_verify(cmd: String) -> Result<String, String> {
    let (program, args) = ALLOWED_COMMANDS
        .iter()
        .find(|(name, _, _)| *name == cmd.as_str())
        .map(|(_, prog, args)| (*prog, *args))
        .ok_or_else(|| "不允许的命令".to_string())?;

    for attempt in 0..2 {
        if attempt > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }

        let fresh_path = build_fresh_path();
        let fresh_envs = read_fresh_env_vars();

        let mut cmd_builder = Command::new(program);
        cmd_builder.args(args).env("PATH", &fresh_path);
        for (k, v) in &fresh_envs {
            cmd_builder.env(k, v);
        }

        match cmd_builder.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let text = if stdout.trim().is_empty() {
                    stderr
                } else {
                    stdout
                };
                if !text.trim().is_empty() {
                    return Ok(text.trim().lines().next().unwrap_or("").to_string());
                }
            }
            Err(_) if attempt == 0 => continue,
            Err(e) => return Err(format!("{program} 执行失败: {e}")),
        }
    }

    Err(format!(
        "{program} 未找到或无输出。请检查是否已正确安装并加入 PATH"
    ))
}
