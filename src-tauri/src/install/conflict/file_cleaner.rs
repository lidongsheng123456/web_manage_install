//! 文件清理器
//!
//! 安全删除旧安装目录。提供多重路径合法性检查，
//! 防止误删系统关键目录。支持重试和定向终止占用进程。

use crate::common::process::hide_window;
use std::path::Path;
use std::process::Command;

/// 安全删除目录：多重检查路径合法性后递归删除。
///
/// 安全措施：
/// - 路径必须存在且为目录
/// - 路径长度必须大于 10 个字符（防止误删根目录如 `C:\`）
/// - 不能是 Windows 系统目录（`C:\Windows` 及其子目录）
/// - 不能是 Program Files 根目录（但其子目录允许删除）
/// - 路径层级必须 >= 3 层（如 `D:\dev\jdk`，排除 `D:\` 或 `D:\dev`）
///
/// 包含重试机制（最多 3 次，间隔 2 秒），应对文件暂时被占用的情况。
pub fn remove_dir_safe(dir: &str) {
    if !is_safe_to_delete(dir) {
        return;
    }
    for attempt in 0..3 {
        if std::fs::remove_dir_all(dir).is_ok() {
            return;
        }
        if attempt < 2 {
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }
}

/// 终止从指定目录内启动的所有进程。
///
/// 使用 PowerShell 精确查找路径前缀匹配的进程并强制终止。
/// 仅终止从目标目录启动的进程，不影响其他位置的同名程序。
/// 终止后等待 2 秒确保进程完全退出、文件锁释放。
pub fn kill_processes_from_dir(dir: &str) {
    let dir_normalized = dir.replace('/', "\\");
    let ps_script = format!(
        "Get-Process | Where-Object {{ $_.Path -and $_.Path.StartsWith('{}', [System.StringComparison]::OrdinalIgnoreCase) }} | Stop-Process -Force -ErrorAction SilentlyContinue",
        dir_normalized.replace('\'', "''")
    );

    let _ = hide_window(
        Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &ps_script]),
    )
    .output();

    std::thread::sleep(std::time::Duration::from_secs(2));
}

/// 删除目录下除指定子目录外的所有内容。
///
/// 用于 MySQL 等场景：需要清理安装文件但保留数据目录。
pub fn remove_dir_except(dir: &str, keep_names: &[&str]) {
    if !is_safe_to_delete(dir) {
        return;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    let keep_lower: Vec<String> = keep_names.iter().map(|n| n.to_lowercase()).collect();

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if keep_lower.contains(&name) {
            continue;
        }
        let entry_path = entry.path();
        if entry_path.is_dir() {
            let _ = std::fs::remove_dir_all(&entry_path);
        } else {
            let _ = std::fs::remove_file(&entry_path);
        }
    }
}

/// 检查目录路径是否安全可删除。
fn is_safe_to_delete(dir: &str) -> bool {
    let path = Path::new(dir);
    if !path.exists() || !path.is_dir() {
        return false;
    }
    if dir.len() <= 10 {
        return false;
    }

    let lower = dir.to_lowercase().replace('/', "\\");

    // 禁止删除 Windows 系统目录
    if lower.contains("\\windows\\") || lower.ends_with("\\windows") {
        return false;
    }

    // 禁止删除 Program Files 根目录本身（子目录允许）
    let pf_roots = ["\\program files", "\\program files (x86)"];
    for pf in &pf_roots {
        if lower.ends_with(pf) {
            return false;
        }
    }

    // 路径层级检查：至少 3 层（盘符 + 父目录 + 目标目录）
    let components: Vec<_> = path.components().collect();
    if components.len() < 3 {
        return false;
    }

    true
}
