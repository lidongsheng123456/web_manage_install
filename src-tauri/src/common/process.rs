//! 跨平台进程工具
//!
//! 在 Windows 上隐藏子进程控制台窗口，避免执行外部命令时弹出黑窗。

use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 为 Command 设置隐藏窗口标志。
///
/// Windows 上添加 `CREATE_NO_WINDOW`；其他平台为 no-op。
pub fn hide_window(cmd: &mut Command) -> &mut Command {
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}
