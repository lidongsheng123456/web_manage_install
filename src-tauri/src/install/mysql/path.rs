/// 校验 MySQL 安装路径：拒绝含非 ASCII 字符的路径。
///
/// 非 ASCII 路径（如中文）在 GBK 编码的 Windows 上可能导致 mysqld
/// 初始化失败。强制要求纯 ASCII 路径可避免此问题。
pub fn validate_install_path(path: &str) -> Result<(), String> {
    if !path.is_ascii() {
        return Err(format!(
            "安装路径包含非英文字符: {}\n\
             MySQL 不支持中文路径，请选择纯英文路径（如 D:\\develop\\software）",
            path
        ));
    }
    Ok(())
}

/// 检测 3306 端口是否被占用，被占用则使用 3307。
pub fn select_port() -> u16 {
    if std::net::TcpListener::bind(("127.0.0.1", 3306)).is_err() {
        3307
    } else {
        3306
    }
}
