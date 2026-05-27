//! 验证命令执行模块
//!
//! Step 4 结果页的 "点击验证" 功能。
//! 使用实时注册表 PATH + 环境变量执行白名单命令，
//! 返回第一行输出供前端展示。

use super::env_reader::{build_fresh_path, read_fresh_env_vars};
use std::process::Command;

/// 允许前端执行的验证命令白名单
const ALLOWED_COMMANDS: &[(&str, &str, &[&str])] = &[
    ("node -v", "node", &["-v"]),
    ("java -version", "java", &["-version"]),
    ("mvn -v", "mvn", &["-v"]),
    ("mysql -V", "mysql", &["-V"]),
];

/// 运行一条验证命令并返回输出文本。
#[tauri::command]
pub async fn run_verify(cmd: String) -> Result<String, String> {
    let (program, args) = ALLOWED_COMMANDS
        .iter()
        .find(|(name, _, _)| *name == cmd.as_str())
        .map(|(_, prog, args)| (*prog, *args))
        .ok_or_else(|| "不允许的命令".to_string())?;

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
            let text = if stdout.trim().is_empty() { stderr } else { stdout };
            if text.trim().is_empty() {
                Err(format!("{program} 未找到或无输出"))
            } else {
                Ok(text.trim().lines().next().unwrap_or("").to_string())
            }
        }
        Err(e) => Err(format!("{program} 执行失败: {e}")),
    }
}
