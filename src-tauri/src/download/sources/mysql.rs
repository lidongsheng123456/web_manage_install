use crate::common::types::MirrorSource;

/// MySQL 8.0 ZIP：只生成当前安装逻辑支持的 winx64 绿色包地址。
pub fn mirrors(version: &str) -> MirrorSource {
    MirrorSource {
        urls: vec![
            format!("https://mirrors.sustech.edu.cn/mysql/downloads/MySQL-8.0/mysql-{version}-winx64.zip"),
            format!("https://mirrors.ustc.edu.cn/mysql/downloads/MySQL-8.0/mysql-{version}-winx64.zip"),
            format!("https://mirrors.aliyun.com/mysql/MySQL-8.0/mysql-{version}-winx64.zip"),
            format!("https://mirrors.huaweicloud.com/mysql/Downloads/MySQL-8.0/mysql-{version}-winx64.zip"),
            format!("https://mirrors.tuna.tsinghua.edu.cn/mysql/downloads/MySQL-8.0/mysql-{version}-winx64.zip"),
            format!("https://cdn.mysql.com/archives/mysql-8.0/mysql-{version}-winx64.zip"),
            format!("https://cdn.mysql.com/Downloads/MySQL-8.0/mysql-{version}-winx64.zip"),
        ],
        filename: format!("mysql-{version}-winx64.zip"),
    }
}
