//! 安装器公共工具函数

use std::path::Path;

/// 在指定目录下查找第一个子目录（ZIP 解压后的顶层目录可能名称不确定）。
fn find_first_dir(dir: &str) -> Option<String> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries {
        if let Ok(e) = entry {
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                return Some(e.path().to_string_lossy().to_string());
            }
        }
    }
    None
}

/// 检查目录下是否直接包含文件（用于判断 ZIP 是否为扁平结构）。
fn has_direct_files(dir: &str) -> bool {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|e| {
                e.file_type().map(|t| t.is_file()).unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// 解压 ZIP 文件到目标目录，返回 ZIP 内的顶层目录名。
///
/// 大多数组件 ZIP 包的结构为 `顶层目录/bin/...`，
/// 解压后需要将顶层目录重命名到最终安装路径。
pub fn extract_zip(zip_path: &str, dest_dir: &str) -> Result<String, String> {
    let file = std::fs::File::open(zip_path)
        .map_err(|e| format!("打开 ZIP 失败: {e}"))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("读取 ZIP 失败: {e}"))?;

    std::fs::create_dir_all(dest_dir)
        .map_err(|e| format!("创建目标目录失败: {e}"))?;

    let mut top_dir = String::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("读取 ZIP 条目失败: {e}"))?;
        let outpath = Path::new(dest_dir).join(entry.mangled_name());

        if i == 0 {
            if let Some(first) = entry.mangled_name().components().next() {
                top_dir = first.as_os_str().to_string_lossy().to_string();
            }
        }

        if entry.is_dir() {
            std::fs::create_dir_all(&outpath).ok();
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p).ok();
            }
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("创建文件失败: {e}"))?;
            std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("写入文件失败: {e}"))?;
        }
    }

    Ok(top_dir)
}

/// 递归复制整个目录树（`fs::rename` 跨盘符时的 fallback）。
pub fn copy_dir_recursive(src: &str, dst: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = Path::new(dst).join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&src_path.to_string_lossy(), &dst_path.to_string_lossy())?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// 解压 ZIP 并将顶层目录移动到最终安装路径。
///
/// `target_name` 为最终目录名。ZIP 内的顶层目录名可能不同
/// （如 `jdk-17.0.2` 或 `jdk-17.0.13+11`），此函数会自动处理。
/// 如果目标路径已存在会先删除。跨盘符 rename 失败时自动回退到递归复制。
///
/// 支持两种 ZIP 结构：
/// - 标准结构：`顶层目录/bin/...`（如 MySQL、Maven）
/// - 扁平结构：文件直接在 ZIP 根目录（如 tporadowski 的 Redis Windows 包）
pub fn extract_and_move(
    zip_path: &str,
    install_root: &str,
    extract_suffix: &str,
    target_name: &str,
) -> Result<String, String> {
    let extract_dir = format!("{install_root}\\_{extract_suffix}_extract");
    let top_dir = extract_zip(zip_path, &extract_dir)?;

    let candidate = format!("{extract_dir}\\{top_dir}");
    let source = if Path::new(&candidate).is_dir() {
        candidate
    } else if let Some(first_dir) = find_first_dir(&extract_dir) {
        first_dir
    } else if has_direct_files(&extract_dir) {
        extract_dir.clone()
    } else {
        return Err(format!("解压目录 {extract_dir} 中未找到有效内容"));
    };

    let target = format!("{install_root}\\{target_name}");

    if Path::new(&target).exists() {
        std::fs::remove_dir_all(&target).ok();
    }

    std::fs::rename(&source, &target)
        .or_else(|_| copy_dir_recursive(&source, &target))
        .map_err(|e| format!("移动目录失败: {e}"))?;

    if Path::new(&extract_dir).exists() {
        std::fs::remove_dir_all(&extract_dir).ok();
    }

    Ok(target)
}
