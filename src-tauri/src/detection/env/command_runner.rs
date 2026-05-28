use super::env_vars::{build_fresh_path, read_fresh_env_vars};
use crate::common::process::hide_window;
use std::path::Path;
use std::process::Command;

/// 使用最新 PATH + 环境变量执行外部命令，返回合并的 stdout+stderr。
///
/// 通过 `cmd /C` 执行以正确解析 `.cmd`/`.bat`（如 mvn.cmd）和 PATHEXT。
pub fn run_cmd_fresh(program: &str, args: &[&str]) -> Option<String> {
    let fresh_path = build_fresh_path();
    let fresh_envs = read_fresh_env_vars();

    let full_cmd = std::iter::once(program.to_string())
        .chain(args.iter().map(|a| a.to_string()))
        .collect::<Vec<_>>()
        .join(" ");

    let mut cmd = Command::new("cmd");
    cmd.args(["/C", &full_cmd]).env("PATH", &fresh_path);
    for (k, v) in &fresh_envs {
        cmd.env(k, v);
    }

    collect_output(&mut cmd)
}

/// 在指定路径直接运行可执行文件并获取输出。
pub fn try_exe_at(exe_path: &str, args: &[&str]) -> Option<String> {
    if !Path::new(exe_path).exists() {
        return None;
    }
    let fresh_path = build_fresh_path();
    let fresh_envs = read_fresh_env_vars();

    let mut cmd = Command::new(exe_path);
    cmd.args(args).env("PATH", &fresh_path);
    for (k, v) in &fresh_envs {
        cmd.env(k, v);
    }

    collect_output(&mut cmd)
}

fn collect_output(cmd: &mut Command) -> Option<String> {
    hide_window(cmd);
    cmd.output().ok().and_then(|o| {
        let stdout = String::from_utf8_lossy(&o.stdout).to_string();
        let stderr = String::from_utf8_lossy(&o.stderr).to_string();
        let text = if stdout.trim().is_empty() {
            stderr
        } else {
            stdout
        };
        if text.trim().is_empty() {
            None
        } else {
            Some(text)
        }
    })
}
