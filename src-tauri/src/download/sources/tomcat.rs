use crate::common::types::MirrorSource;
use crate::common::version_policy::tomcat as tomcat_policy;

/// Tomcat ZIP：华为云、中科大、清华镜像优先，Apache archive 兜底。
///
/// Tomcat 9+ 提供 windows-x64 专用包，Tomcat 7/8 使用平台无关 zip。
pub fn mirrors(version: &str) -> MirrorSource {
    let major = tomcat_policy::major_from_version(version);
    let dir = tomcat_policy::major_dir(major);
    let (suffix, filename) = if tomcat_policy::has_windows_x64_zip(major) {
        ("-windows-x64.zip", format!("apache-tomcat-{version}-windows-x64.zip"))
    } else {
        (".zip", format!("apache-tomcat-{version}.zip"))
    };

    MirrorSource {
        urls: vec![
            format!("https://repo.huaweicloud.com/apache/tomcat/{dir}/v{version}/bin/apache-tomcat-{version}{suffix}"),
            format!("https://mirrors.ustc.edu.cn/apache/tomcat/{dir}/v{version}/bin/apache-tomcat-{version}{suffix}"),
            format!("https://mirrors.tuna.tsinghua.edu.cn/apache/tomcat/{dir}/v{version}/bin/apache-tomcat-{version}{suffix}"),
            format!("https://archive.apache.org/dist/tomcat/{dir}/v{version}/bin/apache-tomcat-{version}{suffix}"),
        ],
        filename,
    }
}
