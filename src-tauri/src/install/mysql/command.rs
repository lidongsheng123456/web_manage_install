use std::process::Command;

/// 创建一个设置了 UTF-8 代码页的 Command，通过 cmd /C 执行。
///
/// 中文 Windows 默认代码页为 936 (GBK)，切换到 UTF-8 可减少 mysqld
/// 初始化输出乱码对判断逻辑的影响。
pub fn cmd_with_utf8(program: &str, args: &[&str]) -> Command {
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
