//! JDK 版本 provider。
//!
//! 版本列表实时读取 Adoptium 可用版本 API，并补齐经典 LTS 大版本。
//! 这里仅负责版本发现、LTS 标记和排序，下载路径由下载源模块处理。

use crate::common::types::VersionOption;
use crate::common::version_policy::{defaults, jdk as jdk_policy};
use crate::version_catalog::{
    compare_semver_desc, limit_keep_values, mark_default, merge_options, option,
};
use serde::Deserialize;

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
    // 版本目录只展示下载链路可覆盖的主版本范围。
    majors.retain(|major| jdk_policy::is_supported_major(*major));
    majors.sort_by(|a, b| compare_semver_desc(&a.to_string(), &b.to_string()));

    let items = majors
        .into_iter()
        .map(|major| {
            let value = major.to_string();
            let is_lts = lts_set.contains(&major) || jdk_policy::is_classic_lts(major);
            let label = jdk_policy::label(major, is_lts);
            option(
                &value,
                label,
                value == defaults::JDK,
                is_lts,
                "Adoptium API",
            )
        })
        .collect::<Vec<_>>();

    let mut items = merge_options(items, classic_jdk_options());
    items.sort_by(|a, b| compare_semver_desc(&a.value, &b.value));
    let items = limit_keep_values(items, jdk_policy::MAX_OPTIONS, jdk_policy::REQUIRED_VALUES);
    mark_default(items, defaults::JDK)
}

fn classic_jdk_options() -> Vec<VersionOption> {
    jdk_policy::CLASSIC_MAJORS
        .iter()
        .map(|major| {
            let value = major.to_string();
            let is_lts = true;
            option(
                &value,
                jdk_policy::label(*major, is_lts),
                value == defaults::JDK,
                is_lts,
                "Classic JDK versions",
            )
        })
        .collect()
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
        assert!(items.iter().any(|item| item.value == "8" && item.lts));
        assert!(!items.iter().any(|item| item.value == "7"));
    }

    #[test]
    fn keeps_classic_jdks_when_truncating() {
        let items = parse_releases(AdoptiumReleases {
            available_lts_releases: vec![17, 21, 25],
            available_releases: (8..40).collect(),
        });

        assert_eq!(items.len(), jdk_policy::MAX_OPTIONS);
        assert!(items.iter().any(|item| item.value == "8"));
        assert!(items.iter().any(|item| item.value == "11"));
        assert!(items.iter().any(|item| item.value == "17" && item.default));
        assert!(items.iter().any(|item| item.value == "21"));
    }
}
