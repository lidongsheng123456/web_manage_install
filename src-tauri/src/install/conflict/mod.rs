//! 环境冲突解决模块
//!
//! 在安装新版本组件之前，自动清理主机上旧版本的残留配置，
//! 确保用户选择的资源版本能正确生效，不被旧环境变量和 PATH 条目干扰。
//!
//! 模块结构（SpringBoot 风格分包 / 单文件单职责）：
//! - `path_scanner`   — PATH 扫描器：从注册表 PATH 中查找含指定 exe 的目录
//! - `path_cleaner`   — PATH 清理器：从注册表 PATH 中移除指定条目
//! - `file_cleaner`   — 文件清理器：安全删除旧安装目录
//! - `node_cleanup`   — Node.js 冲突清理：MSI 卸载 + 文件删除 + PATH 清理
//! - `jdk_cleanup`    — JDK 冲突清理：文件删除 + Oracle javapath 移除 + PATH 清理
//! - `maven_cleanup`  — Maven 冲突清理：文件删除 + PATH 清理 + M2_HOME 兼容
//! - `mysql_cleanup`  — MySQL 冲突清理：文件删除（保留 data）+ PATH 清理

mod file_cleaner;
mod jdk_cleanup;
mod maven_cleanup;
mod mysql_cleanup;
mod node_cleanup;
mod path_cleaner;
mod path_scanner;

use tauri::AppHandle;

/// 在安装前解决旧版本冲突：清理 PATH、卸载旧程序、删除旧文件、重置环境变量。
///
/// 根据组件标识分发到对应的清理器执行。每个清理器内部按照
/// "扫描 PATH → 卸载/删除旧文件 → 移除 PATH 条目 → 重置环境变量" 的顺序依次处理。
///
/// 对于未识别的组件标识（如附加工具 idea/navicat/redis），直接跳过。
pub fn resolve_conflicts(app: &AppHandle, component: &str) -> Result<(), String> {
    match component {
        "nodejs" => node_cleanup::cleanup(app),
        "jdk" => jdk_cleanup::cleanup(app),
        "maven" => maven_cleanup::cleanup(app),
        "mysql" => mysql_cleanup::cleanup(app),
        _ => Ok(()),
    }
}
