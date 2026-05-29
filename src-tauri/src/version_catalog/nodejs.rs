//! Node.js 版本 provider。
//!
//! 优先实时读取清华/npmmirror 的 Node 发布索引，官方 `nodejs.org/dist/index.json`
//! 只作为实时请求兜底。只保留有 Windows x64 MSI 的稳定版本。

use crate::common::types::VersionOption;
use crate::common::version_policy::{defaults, nodejs as node_policy};
use crate::version_catalog::{
    compare_semver_desc, dedup_by_value, limit_keep_values, mark_default, merge_options, option,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct NodeRelease {
    version: String,
    #[serde(default)]
    lts: serde_json::Value,
    #[serde(default)]
    files: Vec<String>,
}

pub async fn load(client: &reqwest::Client) -> Result<Vec<VersionOption>, String> {
    let mut errors = Vec::new();
    for (url, source) in [
        (
            "https://mirrors.tuna.tsinghua.edu.cn/nodejs-release/index.json",
            "清华 Node 镜像",
        ),
        (
            "https://npmmirror.com/mirrors/node/index.json",
            "npmmirror Node 镜像",
        ),
        ("https://nodejs.org/dist/index.json", "Node 官方索引"),
    ] {
        match fetch(client, url, source).await {
            Ok(items) if !items.is_empty() => return Ok(items),
            Ok(_) => errors.push(format!("{source}: 未解析到可用 Windows x64 MSI 版本")),
            Err(e) => errors.push(format!("{source}: {e}")),
        }
    }
    Err(format!("实时获取 Node.js 版本失败: {}", errors.join("; ")))
}

async fn fetch(
    client: &reqwest::Client,
    url: &str,
    source: &str,
) -> Result<Vec<VersionOption>, String> {
    let releases = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("请求版本索引失败: {e}"))?
        .json::<Vec<NodeRelease>>()
        .await
        .map_err(|e| format!("解析版本索引失败: {e}"))?;
    Ok(parse_releases(releases, source))
}

fn parse_releases(mut releases: Vec<NodeRelease>, source: &str) -> Vec<VersionOption> {
    releases.sort_by(|a, b| compare_semver_desc(&a.version, &b.version));

    let mut lts_items = Vec::new();
    let mut current_items = Vec::new();

    for release in releases {
        if !release.files.iter().any(|file| file == "win-x64-msi") {
            continue;
        }
        let value = release.version.trim_start_matches('v').to_string();
        if value.contains('-') {
            continue;
        }
        let is_lts = release.lts != serde_json::Value::Bool(false) && !release.lts.is_null();
        let label = if is_lts {
            format!("v{value} (LTS)")
        } else {
            format!("v{value}")
        };
        let item = option(&value, label, value == defaults::NODEJS, is_lts, source);
        if is_lts {
            lts_items.push(item);
        } else {
            current_items.push(item);
        }
    }

    let mut lts_items = merge_options(lts_items, classic_lts_options());
    lts_items = dedup_by_value(lts_items);
    lts_items.sort_by(|a, b| compare_semver_desc(&a.value, &b.value));
    current_items = dedup_by_value(current_items);

    let mut items = lts_items;
    items.extend(current_items);
    items = dedup_by_value(items);
    items = limit_keep_values(
        items,
        node_policy::MAX_OPTIONS,
        node_policy::REQUIRED_VALUES,
    );
    mark_default(items, defaults::NODEJS)
}

fn classic_lts_options() -> Vec<VersionOption> {
    node_policy::CLASSIC_LTS
        .iter()
        .map(|pin| {
            option(
                pin.version,
                format!("v{} (LTS)", pin.version),
                pin.version == defaults::NODEJS,
                true,
                &format!("Node.js {} LTS", pin.codename),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_lts_before_current_and_requires_windows_msi() {
        let releases = vec![
            NodeRelease {
                version: "v26.0.0".into(),
                lts: false.into(),
                files: vec!["win-x64-msi".into()],
            },
            NodeRelease {
                version: "v24.11.1".into(),
                lts: "Krypton".into(),
                files: vec!["win-x64-msi".into()],
            },
            NodeRelease {
                version: "v20.19.0".into(),
                lts: "Iron".into(),
                files: vec!["src".into()],
            },
        ];
        let items = parse_releases(releases, "test");
        assert_eq!(items[0].value, "24.11.1");
        assert!(items.iter().any(|item| item.value == "24.11.1" && item.lts));
        assert!(items.iter().any(|item| item.value == "26.0.0" && !item.lts));
    }

    #[test]
    fn keeps_default_node_when_truncating() {
        let mut releases = (0..node_policy::MAX_OPTIONS + 3)
            .map(|index| NodeRelease {
                version: format!("v24.{index}.0"),
                lts: "Krypton".into(),
                files: vec!["win-x64-msi".into()],
            })
            .collect::<Vec<_>>();
        releases.push(NodeRelease {
            version: "v20.19.0".into(),
            lts: "Iron".into(),
            files: vec!["win-x64-msi".into()],
        });

        let items = parse_releases(releases, "test");

        assert_eq!(items.len(), node_policy::MAX_OPTIONS);
        assert!(items
            .iter()
            .any(|item| item.value == "20.19.0" && item.default));
    }

    #[test]
    fn keeps_classic_lts_majors_when_limiting() {
        let releases = (0..node_policy::MAX_OPTIONS + 12)
            .map(|index| NodeRelease {
                version: format!("v24.{index}.0"),
                lts: "Krypton".into(),
                files: vec!["win-x64-msi".into()],
            })
            .collect::<Vec<_>>();

        let items = parse_releases(releases, "test");

        for required in node_policy::REQUIRED_VALUES {
            assert!(items.iter().any(|item| item.value == *required));
        }
    }
}
