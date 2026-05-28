use crate::common::process::hide_window;
use crate::detection::env::build_fresh_path;
use std::path::Path;
use std::process::Command;

/// 通过 `where` 命令搜索实时 PATH 中的可执行文件。
pub fn find_via_where(exe_name: &str) -> Vec<String> {
    let fresh_path = build_fresh_path();
    let mut cmd = Command::new("where");
    cmd.arg(exe_name)
        .env("PATH", &fresh_path)
        .stderr(std::process::Stdio::null());
    hide_window(&mut cmd);

    match cmd.output() {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && Path::new(l).exists())
            .collect(),
        _ => Vec::new(),
    }
}
