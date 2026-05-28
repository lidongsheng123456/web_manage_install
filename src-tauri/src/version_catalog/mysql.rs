//! MySQL 版本 provider。
//!
//! 当前安装器的服务名、检测和回滚逻辑都围绕 MySQL 8.0，因此版本目录只从
//! 可实时访问的镜像目录解析 8.0.x Windows ZIP。

use crate::common::types::VersionOption;
use crate::version_catalog::{
    compare_semver_desc, dedup_by_value, limit_keep_default, mark_default, option,
};
use regex_lite::Regex;

const DEFAULT_MYSQL: &str = "8.0.36";
const MAX_MYSQL_OPTIONS: usize = 14;
const DEFAULT_MYSQL_ARCHIVE_URL: &str =
    "https://cdn.mysql.com/archives/mysql-8.0/mysql-8.0.36-winx64.zip";

pub async fn load(client: &reqwest::Client) -> Result<Vec<VersionOption>, String> {
    let mut errors = Vec::new();
    let mut all_items = Vec::new();
    for (url, source) in [
        (
            "https://mirrors.aliyun.com/mysql/MySQL-8.0/",
            "阿里云 MySQL 镜像",
        ),
        (
            "https://mirrors.huaweicloud.com/mysql/Downloads/MySQL-8.0/",
            "华为云 MySQL 镜像",
        ),
    ] {
        match fetch_html(client, url, source).await {
            Ok(items) if !items.is_empty() => all_items.extend(items),
            Ok(_) => errors.push(format!("{source}: 未解析到 MySQL 8.0 Windows ZIP")),
            Err(e) => errors.push(format!("{source}: {e}")),
        }
    }

    if all_items.is_empty() {
        return Err(format!("实时获取 MySQL 版本失败: {}", errors.join("; ")));
    }

    all_items = dedup_by_value(all_items);
    all_items.sort_by(|a, b| compare_semver_desc(&a.value, &b.value));

    // 国内目录经常只保留较旧 MySQL 8.0 包；默认版本必须通过真实 URL 校验后才加入列表。
    if !all_items.iter().any(|item| item.value == DEFAULT_MYSQL)
        && is_url_reachable(client, DEFAULT_MYSQL_ARCHIVE_URL).await
    {
        all_items.push(option(
            DEFAULT_MYSQL,
            format!("MySQL {DEFAULT_MYSQL}"),
            true,
            false,
            "MySQL 官方 Archives",
        ));
        all_items.sort_by(|a, b| compare_semver_desc(&a.value, &b.value));
    }

    let items = limit_keep_default(all_items, MAX_MYSQL_OPTIONS, DEFAULT_MYSQL);
    Ok(mark_default(items, DEFAULT_MYSQL))
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
    let re = Regex::new(r"mysql-(8\.0\.\d+)-winx64\.zip").expect("valid mysql regex");
    let mut versions = re
        .captures_iter(text)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect::<Vec<_>>();

    versions.sort_by(|a, b| compare_semver_desc(a, b));
    versions.dedup();

    versions
        .into_iter()
        .map(|value| {
            option(
                &value,
                format!("MySQL {value}"),
                value == DEFAULT_MYSQL,
                false,
                source,
            )
        })
        .collect::<Vec<_>>()
}

async fn is_url_reachable(client: &reqwest::Client, url: &str) -> bool {
    match client.head(url).send().await {
        Ok(resp) if resp.status().is_success() => true,
        _ => client
            .get(url)
            .header(reqwest::header::RANGE, "bytes=0-0")
            .send()
            .await
            .map(|resp| resp.status().is_success())
            .unwrap_or(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_mysql_80_winx64_zip() {
        let html = r#"
          mysql-8.0.36-winx64.zip
          mysql-8.0.37-winx64.zip
          mysql-8.4.0-winx64.zip
          mysql-8.0.37-winx64-debug-test.zip
        "#;
        let items = parse_html(html, "test");
        assert_eq!(items[0].value, "8.0.37");
        assert!(items.iter().any(|item| item.value == "8.0.36"));
        assert!(!items.iter().any(|item| item.value.starts_with("8.4")));
    }

    #[test]
    fn marks_default_mysql_after_limiting() {
        let items = (20..45)
            .map(|patch| {
                let value = format!("8.0.{patch}");
                option(&value, format!("MySQL {value}"), false, false, "test")
            })
            .chain(std::iter::once(option(
                DEFAULT_MYSQL,
                format!("MySQL {DEFAULT_MYSQL}"),
                false,
                false,
                "test",
            )))
            .collect::<Vec<_>>();

        let items = mark_default(
            limit_keep_default(items, MAX_MYSQL_OPTIONS, DEFAULT_MYSQL),
            DEFAULT_MYSQL,
        );

        assert_eq!(items.len(), MAX_MYSQL_OPTIONS);
        assert!(items
            .iter()
            .any(|item| item.value == DEFAULT_MYSQL && item.default));
    }
}
