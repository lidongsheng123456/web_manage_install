//! Maven 版本 provider。
//!
//! 实时读取 Maven 元数据并过滤 alpha/beta/rc 等预发布版本，只展示 3.x 稳定版。

use crate::common::types::VersionOption;
use crate::version_catalog::{compare_semver_desc, limit_keep_default, mark_default, option};
use regex_lite::Regex;

const DEFAULT_MAVEN: &str = "3.9.6";
const MAVEN_METADATA_URL: &str =
    "https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/maven-metadata.xml";

pub async fn load(client: &reqwest::Client) -> Result<Vec<VersionOption>, String> {
    let text = client
        .get(MAVEN_METADATA_URL)
        .send()
        .await
        .map_err(|e| format!("请求 Maven metadata 失败: {e}"))?
        .text()
        .await
        .map_err(|e| format!("读取 Maven metadata 失败: {e}"))?;

    let items = parse_metadata(&text);
    if items.is_empty() {
        Err("实时获取 Maven 版本失败: 未解析到 Maven 3 稳定版".into())
    } else {
        Ok(items)
    }
}

fn parse_metadata(text: &str) -> Vec<VersionOption> {
    let re = Regex::new(r"<version>([^<]+)</version>").expect("valid maven version regex");
    let mut versions = re
        .captures_iter(text)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .filter(|version| is_stable_maven3(version))
        .collect::<Vec<_>>();
    versions.sort_by(|a, b| compare_semver_desc(a, b));

    let items = versions
        .into_iter()
        .map(|value| {
            option(
                &value,
                format!("Maven {value}"),
                value == DEFAULT_MAVEN,
                false,
                "Apache Maven metadata",
            )
        })
        .collect::<Vec<_>>();

    let items = limit_keep_default(items, 16, DEFAULT_MAVEN);
    mark_default(items, DEFAULT_MAVEN)
}

fn is_stable_maven3(version: &str) -> bool {
    version.starts_with("3.")
        && !version.contains('-')
        && version.split('.').all(|part| part.parse::<u32>().is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_prerelease_and_maven4() {
        let xml = r#"
          <versions>
            <version>3.9.6</version>
            <version>3.9.9</version>
            <version>4.0.0-rc-1</version>
            <version>4.0.0</version>
            <version>3.9.10-alpha</version>
          </versions>
        "#;
        let items = parse_metadata(xml);
        assert_eq!(items[0].value, "3.9.9");
        assert!(items
            .iter()
            .any(|item| item.value == "3.9.6" && item.default));
        assert!(!items.iter().any(|item| item.value.starts_with('4')));
    }

    #[test]
    fn keeps_default_maven_when_truncating() {
        let mut xml = String::from("<versions>");
        for patch in 7..30 {
            xml.push_str(&format!("<version>3.9.{patch}</version>"));
        }
        xml.push_str("<version>3.9.6</version></versions>");

        let items = parse_metadata(&xml);

        assert_eq!(items.len(), 16);
        assert!(items
            .iter()
            .any(|item| item.value == "3.9.6" && item.default));
    }
}
