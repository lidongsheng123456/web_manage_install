//! Tomcat 版本 provider。
//!
//! 从 Apache 归档目录获取各大版本的最新稳定版，
//! 支持 Tomcat 9 / 10 / 11 三个主版本线。

use crate::common::types::VersionOption;
use crate::common::version_policy::{defaults, tomcat as tomcat_policy};
use crate::version_catalog::{
    compare_semver_desc, dedup_by_value, limit_keep_values, mark_default, merge_options, option,
};
use regex_lite::Regex;

const ARCHIVE_URL_TEMPLATE: &str = "https://archive.apache.org/dist/tomcat/tomcat-{major}/";

pub async fn load(client: &reqwest::Client) -> Result<Vec<VersionOption>, String> {
    let mut all_versions = Vec::new();

    for &major in tomcat_policy::SUPPORTED_MAJORS {
        let url = ARCHIVE_URL_TEMPLATE.replace("{major}", &major.to_string());
        match fetch_versions_for_major(client, &url, major).await {
            Ok(versions) => all_versions.extend(versions),
            Err(e) => eprintln!("Tomcat {major} 版本获取失败: {e}"),
        }
    }

    if all_versions.is_empty() {
        return Ok(fallback_versions());
    }

    all_versions.sort_by(|a, b| compare_semver_desc(&a.value, &b.value));
    let all_versions = dedup_by_value(all_versions);
    let mut all_versions = merge_options(all_versions, fallback_versions());
    all_versions.sort_by(|a, b| compare_semver_desc(&a.value, &b.value));
    let all_versions =
        limit_keep_values(all_versions, tomcat_policy::MAX_OPTIONS, tomcat_policy::REQUIRED_VALUES);

    Ok(mark_default(all_versions, defaults::TOMCAT))
}

async fn fetch_versions_for_major(
    client: &reqwest::Client,
    url: &str,
    major: u32,
) -> Result<Vec<VersionOption>, String> {
    let text = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("请求 Tomcat {major} 目录失败: {e}"))?
        .text()
        .await
        .map_err(|e| format!("读取 Tomcat {major} 目录失败: {e}"))?;

    Ok(parse_directory_listing(&text, major))
}

fn parse_directory_listing(html: &str, major: u32) -> Vec<VersionOption> {
    let pattern = if major == 8 {
        format!(r#"href="v({major}\.\d+\.\d+)/""#)
    } else {
        format!(r#"href="v({major}\.\d+\.\d+)/""#)
    };
    let re = Regex::new(&pattern).expect("valid tomcat version regex");

    re.captures_iter(html)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .filter(|v| is_stable(v))
        .filter(|v| major != 8 || v.starts_with("8.5."))
        .map(|version| {
            let label = tomcat_policy::label(&version);
            let is_default = version == defaults::TOMCAT;
            option(&version, label, is_default, false, "Apache Archive")
        })
        .collect()
}

fn is_stable(version: &str) -> bool {
    !version.contains('-') && version.split('.').all(|p| p.parse::<u32>().is_ok())
}

fn fallback_versions() -> Vec<VersionOption> {
    [
        ("11.0.6", "Tomcat 11.0.6"),
        ("10.1.39", "Tomcat 10.1.39 (LTS)"),
        ("9.0.102", "Tomcat 9.0.102 (LTS)"),
        (defaults::TOMCAT, &format!("Tomcat {} (LTS)", defaults::TOMCAT)),
        ("7.0.109", "Tomcat 7.0.109 (EOL)"),
    ]
    .into_iter()
    .map(|(value, label)| {
        option(
            value,
            label.to_string(),
            value == defaults::TOMCAT,
            false,
            "内置版本",
        )
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tomcat9_versions_from_html() {
        let html = r#"
            <a href="v9.0.100/">v9.0.100/</a>
            <a href="v9.0.102/">v9.0.102/</a>
            <a href="v9.0.99/">v9.0.99/</a>
        "#;
        let items = parse_directory_listing(html, 9);
        assert_eq!(items.len(), 3);
        assert!(items.iter().any(|i| i.value == "9.0.102"));
    }

    #[test]
    fn fallback_contains_default_tomcat() {
        let items = fallback_versions();
        assert!(items.iter().any(|i| i.value == defaults::TOMCAT && i.default));
    }
}
