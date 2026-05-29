//! IntelliJ IDEA 自动激活器
//!
//! 所有激活资源以 include_bytes! 嵌入到 exe 中，无需外部文件。
//! 功能等价于 `IDEA激活.vbs` 脚本：
//! 1. 将 config、plugins、active-agt.jar 写入 %APPDATA%\IntelliJIdea\
//! 2. 在 %APPDATA%\JetBrains\ 下找到所有 IntelliJIdea 版本目录
//! 3. 写入 vmoptions 和 key 文件，修改 vmoptions 注入 javaagent

use crate::install::{emit_done, emit_status};
use std::path::{Path, PathBuf};
use tauri::AppHandle;

/// JetBrains 版本目录中包含此关键字的目录被识别为 IDEA 版本。
const IDEA_DIR_KEYWORD: &str = "IntelliJIdea";

// ── 嵌入激活资源文件 ──

static ACTIVE_AGT_JAR: &[u8] =
    include_bytes!("../../../../public/Idea激活/active-agt.jar");
static IDEA_KEY: &[u8] =
    include_bytes!("../../../../public/Idea激活/idea.key");
static IDEA_VMOPTIONS: &[u8] =
    include_bytes!("../../../../public/Idea激活/idea64.exe.vmoptions");

// config 目录
static CONFIG_DNS: &[u8] =
    include_bytes!("../../../../public/Idea激活/config/dns.conf");
static CONFIG_POWER: &[u8] =
    include_bytes!("../../../../public/Idea激活/config/power.conf");
static CONFIG_URL: &[u8] =
    include_bytes!("../../../../public/Idea激活/config/url.conf");

// plugins 目录
static PLUGIN_DNS: &[u8] =
    include_bytes!("../../../../public/Idea激活/plugins/dns.jar");
static PLUGIN_HIDEME: &[u8] =
    include_bytes!("../../../../public/Idea激活/plugins/hideme.jar");
static PLUGIN_POWER: &[u8] =
    include_bytes!("../../../../public/Idea激活/plugins/power.jar");
static PLUGIN_URL: &[u8] =
    include_bytes!("../../../../public/Idea激活/plugins/url.jar");

/// 执行 IDEA 激活全流程。
pub fn activate(app: &AppHandle) -> Result<(), String> {
    emit_status(app, "idea", "config", "正在执行 IDEA 激活...");

    let appdata = std::env::var("APPDATA").map_err(|_| "无法获取 APPDATA 环境变量")?;
    let jet_path = PathBuf::from(&appdata).join("JetBrains");

    if !jet_path.is_dir() {
        return Err(
            "未找到 JetBrains 配置目录，请先启动一次 IDEA 再执行激活".into(),
        );
    }

    let crack_dir = PathBuf::from(&appdata).join(IDEA_DIR_KEYWORD);
    deploy_crack_files(&crack_dir)?;

    let patched = patch_idea_versions(&jet_path, &crack_dir)?;

    if patched == 0 {
        return Err(
            "未找到任何 IntelliJIdea 版本目录，请先启动一次 IDEA".into(),
        );
    }

    emit_done(
        app,
        "idea",
        true,
        &format!("IDEA 激活完成，已处理 {patched} 个版本"),
    );
    Ok(())
}

/// 将嵌入的 config/plugins/active-agt.jar 写出到 %APPDATA%\IntelliJIdea\
fn deploy_crack_files(crack_dir: &Path) -> Result<(), String> {
    if crack_dir.exists() {
        std::fs::remove_dir_all(crack_dir)
            .map_err(|e| format!("清除旧激活目录失败: {e}"))?;
    }
    std::fs::create_dir_all(crack_dir)
        .map_err(|e| format!("创建激活目录失败: {e}"))?;

    // active-agt.jar
    write_file(&crack_dir.join("active-agt.jar"), ACTIVE_AGT_JAR)?;

    // config/
    let config_dir = crack_dir.join("config");
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("创建 config 目录失败: {e}"))?;
    write_file(&config_dir.join("dns.conf"), CONFIG_DNS)?;
    write_file(&config_dir.join("power.conf"), CONFIG_POWER)?;
    write_file(&config_dir.join("url.conf"), CONFIG_URL)?;

    // plugins/
    let plugins_dir = crack_dir.join("plugins");
    std::fs::create_dir_all(&plugins_dir)
        .map_err(|e| format!("创建 plugins 目录失败: {e}"))?;
    write_file(&plugins_dir.join("dns.jar"), PLUGIN_DNS)?;
    write_file(&plugins_dir.join("hideme.jar"), PLUGIN_HIDEME)?;
    write_file(&plugins_dir.join("power.jar"), PLUGIN_POWER)?;
    write_file(&plugins_dir.join("url.jar"), PLUGIN_URL)?;

    Ok(())
}

/// 在 JetBrains 目录下查找所有 IntelliJIdea 版本并注入激活配置。
fn patch_idea_versions(jet_path: &Path, crack_dir: &Path) -> Result<usize, String> {
    let mut patched = 0;

    let entries = std::fs::read_dir(jet_path)
        .map_err(|e| format!("读取 JetBrains 目录失败: {e}"))?;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.contains(IDEA_DIR_KEYWORD) {
            continue;
        }
        let version_dir = entry.path();
        if !version_dir.is_dir() {
            continue;
        }

        // 写入 vmoptions 和 key
        write_file(&version_dir.join("idea64.exe.vmoptions"), IDEA_VMOPTIONS)?;
        write_file(&version_dir.join("idea.key"), IDEA_KEY)?;

        // 修改 vmoptions 注入 javaagent
        let vmoptions_path = version_dir.join("idea64.exe.vmoptions");
        patch_vmoptions(&vmoptions_path, crack_dir)?;

        patched += 1;
    }

    Ok(patched)
}

/// 修改 vmoptions 文件：移除旧的 javaagent 行，追加新的激活参数。
fn patch_vmoptions(path: &Path, crack_dir: &Path) -> Result<(), String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("读取 vmoptions 失败: {e}"))?;

    let agent_jar = crack_dir.join("active-agt.jar");
    let agent_path_str = agent_jar.to_string_lossy().replace('/', "\\");

    let filtered: Vec<&str> = content
        .lines()
        .filter(|line| {
            !line.contains("-javaagent:")
                && !line.contains("--add-opens=java.base/jdk.internal.org.objectweb.asm")
        })
        .collect();

    let mut new_content = filtered.join("\n");
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push_str("--add-opens=java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED\n");
    new_content.push_str("--add-opens=java.base/jdk.internal.org.objectweb.asm.tree=ALL-UNNAMED\n");
    new_content.push_str(&format!("-javaagent:{agent_path_str}\n"));

    std::fs::write(path, &new_content)
        .map_err(|e| format!("写入 vmoptions 失败: {e}"))?;

    Ok(())
}

fn write_file(path: &Path, data: &[u8]) -> Result<(), String> {
    std::fs::write(path, data)
        .map_err(|e| format!("写入文件 {} 失败: {e}", path.display()))
}
