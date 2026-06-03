/// 生成 my.ini 配置文件。
///
/// 路径统一使用正斜杠，避免 MySQL 将 `\b`、`\t` 等片段解析成转义字符。
pub fn write_my_ini(mysql_home: &str, port: u16) -> Result<(), String> {
    let base = mysql_home.replace('\\', "/");
    let data = format!("{base}/data");

    let lines = [
        "[mysqld]",
        &format!("port={port}"),
        &format!("basedir=\"{base}\""),
        &format!("datadir=\"{data}\""),
        "max_connections=200",
        "character-set-server=utf8mb4",
        "default-storage-engine=INNODB",
        "",
        "[mysql]",
        "default-character-set=utf8mb4",
        "",
        "[client]",
        &format!("port={port}"),
        "default-character-set=utf8mb4",
    ];

    let content = lines.join("\r\n");
    let path = format!("{mysql_home}\\my.ini");
    std::fs::write(&path, content.as_bytes()).map_err(|e| format!("写入 my.ini 失败: {e}"))
}
