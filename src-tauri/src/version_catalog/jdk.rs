//! JDK 版本 provider。
//!
//! 版本列表实时读取 Adoptium 可用版本 API。下载阶段仍由镜像源模块优先使用华为云
//! OpenJDK 镜像；这里仅负责版本发现、LTS 标记和排序。

use crate::common::types::VersionOption;
use crate::version_catalog::{compare_semver_desc, limit_keep_default, mark_default, option};
use serde::Deserialize;

const DEFAULT_JDK: &str = "17";

#[derive(Debug, Deserialize)]
struct AdoptiumReleases {
    #[serde(default)]
    available_lts_releases: Vec<u32>,
    #[serde(default)]
    available_releases: Vec<u32>,
}

pub async fn load(client: &reqwest::Client) -> Result<Vec<VersionOption>, String> {
    let data = client
        .get("https://api.adoptium.net/v3/info/available_releases")
        .send()
        .await
        .map_err(|e| format!("请求 JDK 版本 API 失败: {e}"))?
        .json::<AdoptiumReleases>()
        .await
        .map_err(|e| format!("解析 JDK 版本 API 失败: {e}"))?;

    let items = parse_releases(data);
    if items.is_empty() {
        Err("实时获取 JDK 版本失败: 未解析到可用版本".into())
    } else {
        Ok(items)
    }
}

fn parse_releases(data: AdoptiumReleases) -> Vec<VersionOption> {
    let lts_set = data
        .available_lts_releases
        .into_iter()
        .collect::<std::collections::HashSet<_>>();
    let mut majors = data.available_releases;
    // 华为 OpenJDK 镜像当前从 9 开始提供 Windows x64 ZIP，版本目录只展示下载链路可覆盖的版本。
    majors.retain(|major| (9..=26).contains(major));
    majors.sort_by(|a, b| compare_semver_desc(&a.to_string(), &b.to_string()));

    let items = majors
        .into_iter()
        .map(|major| {
            let value = major.to_string();
            let is_lts = lts_set.contains(&major);
            let label = if is_lts {
                format!("JDK {major} (LTS)")
            } else {
                format!("JDK {major}")
            };
            option(&value, label, value == DEFAULT_JDK, is_lts, "Adoptium API")
        })
        .collect::<Vec<_>>();

    let items = limit_keep_default(items, 12, DEFAULT_JDK);
    mark_default(items, DEFAULT_JDK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marks_lts_and_default() {
        let items = parse_releases(AdoptiumReleases {
            available_lts_releases: vec![17, 21, 25],
            available_releases: vec![16, 17, 21, 25, 26],
        });
        assert!(items.iter().any(|item| item.value == "17" && item.default));
        assert!(items.iter().any(|item| item.value == "21" && item.lts));
        assert!(!items.iter().any(|item| item.value == "7"));
        assert!(!items.iter().any(|item| item.value == "8"));
    }

    #[test]
    fn keeps_default_jdk_when_truncating() {
        let items = parse_releases(AdoptiumReleases {
            available_lts_releases: vec![17, 21, 25],
            available_releases: (17..40).collect(),
        });

        assert_eq!(items.len(), 10);
        assert!(items.iter().any(|item| item.value == "17" && item.default));
    }
}
