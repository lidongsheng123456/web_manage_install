use crate::install::emit_status;
use std::path::Path;
use tauri::AppHandle;

/// 检查 Visual C++ 运行库是否已安装。
///
/// MySQL 8.0 依赖 VC++ 2015-2022 Redistributable (vcruntime140.dll)。
pub fn check_vcruntime(app: &AppHandle) {
    let sys32 = std::env::var("SYSTEMROOT").unwrap_or_else(|_| r"C:\Windows".into());
    let dll = format!(r"{}\System32\vcruntime140.dll", sys32);
    if !Path::new(&dll).exists() {
        emit_status(
            app,
            "mysql",
            "config",
            "⚠ 未检测到 vcruntime140.dll (Visual C++ 运行库)，MySQL 可能无法启动。\
             请安装 Microsoft Visual C++ 2015-2022 Redistributable",
        );
    }
}
