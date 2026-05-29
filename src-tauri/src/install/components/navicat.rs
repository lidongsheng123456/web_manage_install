//! Navicat Premium 自动激活器
//!
//! 等价于手动操作：
//! 1. 运行 `无限试用Navicat.bat`（清理注册表试用记录）
//! 2. 将 `winmm.dll` 复制到 Navicat 安装目录

use crate::detection::finder::find_uninstall::find_install_location;
use crate::install::{emit_done, emit_status};
use std::path::Path;
use tauri::AppHandle;
use winreg::enums::*;
use winreg::RegKey;

/// winmm.dll 嵌入二进制（避免杀软拦截资源文件复制）。
static WINMM_DLL_BYTES: &[u8] =
    include_bytes!("../../../../public/navicat激活/winmm.dll");

/// Navicat 可能的进程名列表。
const NAVICAT_PROCESS_NAMES: &[&str] = &["navicat.exe", "Navicat Premium.exe"];

/// 注册表中 Navicat 配置根键路径。
const REG_NAVICAT_ROOT: &str = r"Software\PremiumSoft\NavicatPremium";

/// 执行 Navicat 完整激活流程。
pub fn activate(app: &AppHandle) -> Result<(), String> {
    emit_status(app, "navicat", "config", "正在执行 Navicat 自动激活...");

    // Step 1: 关闭正在运行的 Navicat 进程（避免文件占用）
    kill_navicat_processes(app);

    // Step 2: 清理注册表试用记录
    emit_status(app, "navicat", "config", "正在清理 Navicat 注册表记录...");
    clean_navicat_registry()?;

    // Step 3: 写入 winmm.dll 到安装目录
    emit_status(app, "navicat", "config", "正在部署激活 DLL...");
    let install_dir = find_navicat_install_dir()?;
    let dll_dst = Path::new(&install_dir).join("winmm.dll");

    // 尝试写入，如果文件被占用则等待后重试
    write_dll_with_retry(&dll_dst)?;

    emit_done(
        app,
        "navicat",
        true,
        &format!("Navicat 激活完成（DLL 已部署到 {install_dir}）"),
    );
    Ok(())
}

/// 关闭所有 Navicat 相关进程。
fn kill_navicat_processes(app: &AppHandle) {
    use crate::common::process::hide_window;

    emit_status(app, "navicat", "config", "正在关闭 Navicat 进程...");
    for name in NAVICAT_PROCESS_NAMES {
        let mut cmd = std::process::Command::new("taskkill");
        cmd.args(["/F", "/IM", name, "/T"]);
        hide_window(&mut cmd);
        let _ = cmd.output();
    }
    std::thread::sleep(std::time::Duration::from_millis(500));
}

/// 写入 DLL 文件，如果文件被占用则关闭进程后重试（最多 3 次）。
fn write_dll_with_retry(dst: &Path) -> Result<(), String> {
    for attempt in 0..3 {
        match std::fs::write(dst, WINMM_DLL_BYTES) {
            Ok(_) => return Ok(()),
            Err(_) if attempt < 2 => {
                std::thread::sleep(std::time::Duration::from_secs(1));
                for name in NAVICAT_PROCESS_NAMES {
                    let mut cmd = std::process::Command::new("taskkill");
                    cmd.args(["/F", "/IM", name, "/T"]);
                    crate::common::process::hide_window(&mut cmd);
                    let _ = cmd.output();
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            Err(e) => {
                return Err(format!(
                    "写入 winmm.dll 失败（请手动关闭 Navicat 后重试）: {e}"
                ));
            }
        }
    }
    Ok(())
}

/// 清理 Navicat 注册表中的试用记录。
/// 等价于 `无限试用Navicat.bat` 的逻辑。
fn clean_navicat_registry() -> Result<(), String> {
    // 清理 NavicatPremium 下的 Registration 子键
    clean_registration_keys()?;

    // 清理 CLSID 下的 Info 子键
    clean_clsid_info_keys()?;

    Ok(())
}

/// 删除 NavicatPremium 下所有 Registration 子键的值。
fn clean_registration_keys() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let nav_key = match hkcu.open_subkey_with_flags(REG_NAVICAT_ROOT, KEY_READ) {
        Ok(k) => k,
        Err(_) => return Ok(()),
    };

    let subkeys: Vec<String> = nav_key
        .enum_keys()
        .filter_map(|k| k.ok())
        .filter(|name| name.contains("Registration"))
        .collect();

    for sub_name in subkeys {
        let full_path = format!("{REG_NAVICAT_ROOT}\\{sub_name}");
        if let Ok(sub) = hkcu.open_subkey_with_flags(&full_path, KEY_ALL_ACCESS) {
            let values: Vec<String> = sub
                .enum_values()
                .filter_map(|v| v.ok())
                .map(|(name, _)| name)
                .collect();
            for val_name in values {
                let _ = sub.delete_value(&val_name);
            }
        }
        let _ = hkcu.delete_subkey_all(&full_path);
    }

    Ok(())
}

/// 删除 HKCU\Software\Classes\CLSID 下所有以 Info 结尾的子键值。
fn clean_clsid_info_keys() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let clsid_path = r"Software\Classes\CLSID";

    let clsid_key = match hkcu.open_subkey_with_flags(clsid_path, KEY_READ) {
        Ok(k) => k,
        Err(_) => return Ok(()),
    };

    let guid_keys: Vec<String> = clsid_key
        .enum_keys()
        .filter_map(|k| k.ok())
        .collect();

    for guid in &guid_keys {
        let guid_path = format!("{clsid_path}\\{guid}");
        if let Ok(guid_key) = hkcu.open_subkey_with_flags(&guid_path, KEY_READ) {
            let sub_names: Vec<String> = guid_key
                .enum_keys()
                .filter_map(|k| k.ok())
                .filter(|name| name.ends_with("Info"))
                .collect();

            for info_name in sub_names {
                let info_path = format!("{guid_path}\\{info_name}");
                if let Ok(info_key) =
                    hkcu.open_subkey_with_flags(&info_path, KEY_ALL_ACCESS)
                {
                    let values: Vec<String> = info_key
                        .enum_values()
                        .filter_map(|v| v.ok())
                        .map(|(name, _)| name)
                        .collect();
                    for val_name in values {
                        let _ = info_key.delete_value(&val_name);
                    }
                }
                let _ = hkcu.delete_subkey_all(&info_path);
            }
        }
    }

    Ok(())
}

/// 查找 Navicat Premium 的安装目录。
fn find_navicat_install_dir() -> Result<String, String> {
    // 优先通过注册表 Uninstall 键查找
    let locations = find_install_location("Navicat Premium");
    if let Some(loc) = locations.first() {
        return Ok(loc.clone());
    }

    // 常见安装路径回退
    let common_paths = [
        r"C:\Program Files\PremiumSoft\Navicat Premium 17",
        r"C:\Program Files\PremiumSoft\Navicat Premium 16",
        r"C:\Program Files\PremiumSoft\Navicat Premium 15",
        r"C:\Program Files (x86)\PremiumSoft\Navicat Premium 17",
        r"C:\Program Files (x86)\PremiumSoft\Navicat Premium 16",
        r"C:\Program Files (x86)\PremiumSoft\Navicat Premium 15",
    ];

    for path in &common_paths {
        if Path::new(path).is_dir() {
            return Ok(path.to_string());
        }
    }

    Err("未找到 Navicat Premium 安装目录，请确认已安装 Navicat".into())
}
