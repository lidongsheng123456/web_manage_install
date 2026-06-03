//! 系统环境变量配置门面
//!
//! 对外保留安装器需要的四个操作：设置变量、追加 PATH、删除变量、
//! 从 PATH 移除条目。具体注册表写入、PATH 字符串处理和广播刷新拆到
//! 独立模块，便于后续测试和替换实现。
//!
//! **线程安全**：所有操作通过全局互斥锁序列化，防止并发安装时
//! 多线程同时读写注册表 PATH 导致条目丢失（TOCTOU 竞争条件）。

use crate::system::env_broadcast::broadcast_env_change;
use crate::system::env_registry;
use crate::system::path_entry::{append_entry, path_contains, remove_entry};
use std::sync::{Mutex, OnceLock};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// 设置环境变量，自动降级：HKLM → HKCU → setx。
pub fn set_system_env(key: &str, value: &str) -> Result<(), String> {
    let _guard = env_lock().lock().unwrap();

    if env_registry::try_set_hklm(key, value).is_ok() {
        broadcast_env_change();
        return Ok(());
    }

    if env_registry::try_set_hkcu(key, value).is_ok() {
        broadcast_env_change();
        return Ok(());
    }

    let result = env_registry::try_setx(key, value);
    if result.is_ok() {
        broadcast_env_change();
    }
    result
}

/// 向 PATH 追加路径条目，自动降级：HKLM → HKCU → setx。
pub fn append_to_path(new_entry: &str) -> Result<(), String> {
    let _guard = env_lock().lock().unwrap();

    let normalized = new_entry.trim_end_matches('\\').to_lowercase();

    if let Ok(current) = env_registry::read_path_from_hklm() {
        if path_contains(&current, &normalized) {
            return Ok(());
        }
        let updated = append_entry(&current, new_entry);
        if env_registry::try_write_path_hklm(&updated).is_ok() {
            broadcast_env_change();
            return Ok(());
        }
    }

    if let Ok(current) = env_registry::read_path_from_hkcu() {
        if path_contains(&current, &normalized) {
            return Ok(());
        }
        let updated = append_entry(&current, new_entry);
        if env_registry::try_write_path_hkcu(&updated).is_ok() {
            broadcast_env_change();
            return Ok(());
        }
    }

    let current_path = std::env::var("PATH").unwrap_or_default();
    if path_contains(&current_path, &normalized) {
        return Ok(());
    }
    env_registry::try_setx("PATH", &format!("{current_path};{new_entry}"))
}

/// 删除环境变量（用于回滚），尝试 HKLM → HKCU。
pub fn remove_env(key: &str) {
    let _guard = env_lock().lock().unwrap();

    env_registry::delete_env_value_hklm(key);
    env_registry::delete_env_value_hkcu(key);
    broadcast_env_change();
}

/// 从 PATH 中移除指定路径条目（用于回滚），大小写不敏感。
pub fn remove_from_path(entry: &str) {
    let _guard = env_lock().lock().unwrap();

    let normalized = entry.trim_end_matches('\\').to_lowercase();

    if let Ok(current) = env_registry::read_path_from_hklm() {
        let updated = remove_entry(&current, &normalized);
        if updated != current {
            let _ = env_registry::try_write_path_hklm(&updated);
        }
    }
    if let Ok(current) = env_registry::read_path_from_hkcu() {
        let updated = remove_entry(&current, &normalized);
        if updated != current {
            let _ = env_registry::try_write_path_hkcu(&updated);
        }
    }
    broadcast_env_change();
}
