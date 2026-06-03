//! 下载源分发入口
//!
//! 每个资源的 URL 生成放在独立 source 文件里，本文件只负责按组件分发、
//! 维护默认版本，并用测试约束“中国源优先，官方源兜底”的排序规则。

use crate::common::types::MirrorSource;
use crate::common::version_policy::defaults;

mod idea;
mod jdk;
mod maven;
mod mysql;
mod navicat;
mod nodejs;
mod redis;
mod tomcat;

pub fn get_mirrors_versioned(component: &str, version: &str) -> MirrorSource {
    match component {
        "nodejs" => nodejs::mirrors(version),
        "jdk" => jdk::mirrors(version),
        "mysql" => mysql::mirrors(version),
        "maven" => maven::mirrors(version),
        "idea" => idea::mirrors(version),
        "navicat" => navicat::mirrors(),
        "redis" => redis::mirrors(version),
        "tomcat" => tomcat::mirrors(version),
        _ => MirrorSource {
            urls: vec![],
            filename: String::new(),
        },
    }
}

pub fn get_mirrors(component: &str) -> MirrorSource {
    get_mirrors_versioned(component, default_version(component))
}

fn default_version(component: &str) -> &str {
    defaults::component(component)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_download_sources_prefer_china_mirrors() {
        let source = get_mirrors_versioned("nodejs", "20.19.0");
        assert!(source.urls[0].contains("mirrors.cloud.tencent.com"));
        assert!(source.urls[1].contains("mirrors.tuna.tsinghua.edu.cn"));
        assert!(source.urls[2].contains("npmmirror.com"));
        assert!(source.urls.last().unwrap().contains("nodejs.org"));
    }

    #[test]
    fn mysql_download_sources_keep_official_source_last() {
        let source = get_mirrors_versioned("mysql", "8.0.36");
        assert!(source.urls[0].contains("mirrors.sustech.edu.cn"));
        assert!(source.urls[1].contains("mirrors.ustc.edu.cn"));
        assert!(source
            .urls
            .iter()
            .any(|url| url.contains("cdn.mysql.com/archives")));
    }

    #[test]
    fn jdk_download_sources_are_inline_and_prefer_huawei_mirror() {
        let source = get_mirrors_versioned("jdk", "21");
        assert!(source.urls[0].contains("repo.huaweicloud.com/openjdk/21.0.2"));
        assert_eq!(source.filename, "openjdk-21_windows-x64_bin.zip");
    }

    #[test]
    fn jdk8_download_sources_use_adoptium_zip() {
        let source = get_mirrors_versioned("jdk", "8");
        assert!(source.urls[0].contains("mirrors.tuna.tsinghua.edu.cn/Adoptium/8"));
        assert_eq!(
            source.filename,
            "OpenJDK8U-jdk_x64_windows_hotspot_8u492b09.zip"
        );
    }

    #[test]
    fn mysql57_download_sources_use_57_directories() {
        let source = get_mirrors_versioned("mysql", "5.7.44");
        assert!(source.urls[0].contains("MySQL-5.7"));
        assert!(source.urls.iter().any(|url| url.contains("mysql-5.7")));
        assert_eq!(source.filename, "mysql-5.7.44-winx64.zip");
    }

    #[test]
    fn maven_download_sources_prefer_verified_china_mirrors() {
        let source = get_mirrors_versioned("maven", "3.9.6");
        assert!(source.urls[0].contains("repo.huaweicloud.com"));
        assert!(source.urls[1].contains("mirrors.ustc.edu.cn"));
        assert!(source.urls.last().unwrap().contains("archive.apache.org"));
    }

    #[test]
    fn idea_download_sources_try_china_endpoints_first() {
        let source = get_mirrors_versioned("idea", "2023.3.8");
        assert!(source.urls[0].contains("download.jetbrains.com/idea"));
        assert!(source.urls[1].contains("download-cdn.clf.jetbrains.com.cn"));
        assert!(source
            .urls
            .last()
            .unwrap()
            .contains("download-cdn.jetbrains.com"));
    }

    #[test]
    fn navicat_download_sources_prefer_china_site_endpoint() {
        let source = get_mirrors_versioned("navicat", "17");
        assert!(source.urls[0].contains("dn.navicat.com.cn"));
        assert!(source.urls[1].contains("navicat17_premium_cs_x64.exe"));
        assert!(source.urls.last().unwrap().contains("download.navicat.com"));
    }

    #[test]
    fn redis_download_sources_keep_github_accelerator_before_official() {
        let source = get_mirrors_versioned("redis", "5.0.14.1");
        assert!(source.urls[0].contains("gh-proxy.com"));
        assert!(source.urls.last().unwrap().contains("github.com"));
    }

    #[test]
    fn tomcat9_download_sources_prefer_china_mirrors_with_windows_x64() {
        let source = get_mirrors_versioned("tomcat", "9.0.102");
        assert!(source.urls[0].contains("repo.huaweicloud.com"));
        assert!(source.urls[1].contains("mirrors.ustc.edu.cn"));
        assert!(source.urls.last().unwrap().contains("archive.apache.org"));
        assert_eq!(source.filename, "apache-tomcat-9.0.102-windows-x64.zip");
    }

    #[test]
    fn tomcat7_download_sources_use_platform_independent_zip() {
        let source = get_mirrors_versioned("tomcat", "7.0.109");
        assert_eq!(source.filename, "apache-tomcat-7.0.109.zip");
        assert!(!source.urls[0].contains("windows-x64"));
    }

    #[test]
    fn tomcat85_download_sources_use_platform_independent_zip() {
        let source = get_mirrors_versioned("tomcat", "8.5.100");
        assert_eq!(source.filename, "apache-tomcat-8.5.100.zip");
    }
}
