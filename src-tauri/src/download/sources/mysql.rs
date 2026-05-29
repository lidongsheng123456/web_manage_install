use crate::common::types::MirrorSource;
use crate::common::version_policy::mysql as mysql_policy;

/// MySQL ZIP：按版本系列生成当前安装逻辑支持的 winx64 绿色包地址。
pub fn mirrors(version: &str) -> MirrorSource {
    let series = mysql_policy::series(version);

    MirrorSource {
        urls: mysql_urls(version, series),
        filename: format!("mysql-{version}-winx64.zip"),
    }
}

fn mysql_urls(version: &str, series: &str) -> Vec<String> {
    let dir = mysql_policy::directory_name(series);
    let archive_dir = mysql_policy::archive_directory_name(series);

    vec![
        format!("https://mirrors.sustech.edu.cn/mysql/downloads/{dir}/mysql-{version}-winx64.zip"),
        format!("https://mirrors.ustc.edu.cn/mysql/downloads/{dir}/mysql-{version}-winx64.zip"),
        format!("https://mirrors.aliyun.com/mysql/{dir}/mysql-{version}-winx64.zip"),
        format!("https://mirrors.huaweicloud.com/mysql/Downloads/{dir}/mysql-{version}-winx64.zip"),
        format!(
            "https://mirrors.tuna.tsinghua.edu.cn/mysql/downloads/{dir}/mysql-{version}-winx64.zip"
        ),
        format!("https://cdn.mysql.com/archives/{archive_dir}/mysql-{version}-winx64.zip"),
        format!("https://cdn.mysql.com/Downloads/{dir}/mysql-{version}-winx64.zip"),
    ]
}
