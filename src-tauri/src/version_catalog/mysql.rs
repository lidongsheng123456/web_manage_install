//! MySQL 版本 provider。
//!
//! 版本列表从可实时访问的镜像目录解析，并补齐安装器需要保留的经典版本。
//! 这里仅负责版本发现、去重、排序和默认标记，下载目录/服务名策略统一放在公共策略层。

use crate::common::types::VersionOption;
use crate::common::version_policy::{defaults, mysql as mysql_policy};
use crate::version_catalog::{
    compare_semver_desc, dedup_by_value, limit_keep_values, mark_default, merge_options, option,
};
use regex_lite::Regex;

pub async fn load(client: &reqwest::Client) -> Result<Vec<VersionOption>, String> {
    let mut errors = Vec::new();
    let mut all_items = Vec::new();
    for (url, source) in catalog_sources() {
        match fetch_html(client, &url, source).await {
            Ok(items) if !items.is_empty() => all_items.extend(items),
            Ok(_) => errors.push(format!("{source}: 未解析到 Windows ZIP 版本")),
            Err(e) => errors.push(format!("{source}: {e}")),
        }
    }

    all_items = merge_options(all_items, required_mysql_options());
    if all_items.is_empty() {
        return Err(format!("实时获取 MySQL 版本失败: {}", errors.join("; ")));
    }

    all_items = dedup_by_value(all_items);
    all_items.sort_by(|a, b| compare_semver_desc(&a.value, &b.value));

    let items = limit_keep_values(
        all_items,
        mysql_policy::MAX_OPTIONS,
        mysql_policy::REQUIRED_VALUES,
    );
    Ok(mark_default(items, defaults::MYSQL))
}

fn catalog_sources() -> Vec<(String, &'static str)> {
    mysql_policy::SUPPORTED_SERIES
        .iter()
        .flat_map(|&series| {
            let dir = mysql_policy::directory_name(series);
            [
                (
                    format!("https://mirrors.aliyun.com/mysql/{dir}/"),
                    "阿里云 MySQL 镜像",
                ),
                (
                    format!("https://mirrors.huaweicloud.com/mysql/Downloads/{dir}/"),
                    "华为云 MySQL 镜像",
                ),
            ]
        })
        .collect()
}

async fn fetch_html(
    client: &reqwest::Client,
    url: &str,
    source: &str,
) -> Result<Vec<VersionOption>, String> {
    let text = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("请求版本目录失败: {e}"))?
        .text()
        .await
        .map_err(|e| format!("读取版本目录失败: {e}"))?;
    Ok(parse_html(&text, source))
}

fn parse_html(text: &str, source: &str) -> Vec<VersionOption> {
    let re = Regex::new(&mysql_policy::catalog_regex_pattern()).expect("valid mysql regex");
    let mut versions = re
        .captures_iter(text)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect::<Vec<_>>();

    versions.sort_by(|a, b| compare_semver_desc(a, b));
    versions.dedup();

    versions
        .into_iter()
        .map(|value| mysql_option(&value, source))
        .collect::<Vec<_>>()
}

fn required_mysql_options() -> Vec<VersionOption> {
    mysql_policy::REQUIRED_VALUES
        .iter()
        .map(|value| mysql_option(value, "MySQL 官方 Archives"))
        .collect()
}

fn mysql_option(value: &str, source: &str) -> VersionOption {
    option(
        value,
        format!("MySQL {value}"),
        value == defaults::MYSQL,
        false,
        source,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_mysql_80_and_57_winx64_zip() {
        let html = r#"
          mysql-8.0.36-winx64.zip
          mysql-8.0.37-winx64.zip
          mysql-8.4.0-winx64.zip
          mysql-5.7.38-winx64.zip
          mysql-5.6.51-winx64.zip
          mysql-8.0.37-winx64-debug-test.zip
        "#;
        let items = parse_html(html, "test");
        assert!(items.iter().any(|item| item.value == "8.0.37"));
        assert!(items.iter().any(|item| item.value == "8.0.36"));
        assert!(items.iter().any(|item| item.value == "5.7.38"));
        assert!(!items.iter().any(|item| item.value.starts_with("8.4")));
        assert!(!items.iter().any(|item| item.value.starts_with("5.6")));
    }

    #[test]
    fn marks_default_and_keeps_mysql_5_when_limiting() {
        let items = (20..45)
            .map(|patch| {
                let value = format!("8.0.{patch}");
                mysql_option(&value, "test")
            })
            .chain(required_mysql_options())
            .collect::<Vec<_>>();

        let items = mark_default(
            limit_keep_values(
                items,
                mysql_policy::MAX_OPTIONS,
                mysql_policy::REQUIRED_VALUES,
            ),
            defaults::MYSQL,
        );

        assert_eq!(items.len(), mysql_policy::MAX_OPTIONS);
        assert!(items
            .iter()
            .any(|item| item.value == defaults::MYSQL && item.default));
        assert!(items.iter().any(|item| item.value == "5.7.44"));
        assert!(items.iter().any(|item| item.value == "5.7.38"));
    }
}
