//! 各组件版本来源解析。
//!
//! provider 只负责“从哪里拿版本、如何过滤和排序”，不直接参与下载和安装。

pub mod command;
mod jdk;
mod maven;
mod mysql;
mod nodejs;
mod service;

use crate::common::types::VersionOption;
use std::cmp::Ordering;

pub(crate) fn option(
    value: &str,
    label: String,
    default: bool,
    lts: bool,
    source: &str,
) -> VersionOption {
    VersionOption {
        value: value.to_string(),
        label,
        default,
        lts,
        source: source.to_string(),
    }
}

pub(crate) fn parse_semver(value: &str) -> Vec<u32> {
    value
        .trim_start_matches('v')
        .split(['.', '-', '+'])
        .filter_map(|part| part.parse::<u32>().ok())
        .collect()
}

pub(crate) fn compare_semver_desc(a: &str, b: &str) -> Ordering {
    let av = parse_semver(a);
    let bv = parse_semver(b);
    for i in 0..av.len().max(bv.len()) {
        let left = *av.get(i).unwrap_or(&0);
        let right = *bv.get(i).unwrap_or(&0);
        match right.cmp(&left) {
            Ordering::Equal => continue,
            ord => return ord,
        }
    }
    Ordering::Equal
}

pub(crate) fn dedup_by_value(items: Vec<VersionOption>) -> Vec<VersionOption> {
    let mut seen = std::collections::HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.value.clone()))
        .collect()
}

pub(crate) fn mark_default(
    mut items: Vec<VersionOption>,
    default_value: &str,
) -> Vec<VersionOption> {
    for item in &mut items {
        item.default = item.value == default_value;
    }
    items
}

pub(crate) fn merge_options(
    mut items: Vec<VersionOption>,
    pinned: Vec<VersionOption>,
) -> Vec<VersionOption> {
    for item in pinned {
        if !items.iter().any(|existing| existing.value == item.value) {
            items.push(item);
        }
    }
    items
}

pub(crate) fn limit_keep_default(
    mut items: Vec<VersionOption>,
    max_len: usize,
    default_value: &str,
) -> Vec<VersionOption> {
    if max_len == 0 || items.len() <= max_len {
        return items;
    }

    // 版本列表会限制展示数量，但项目约定的默认版本不能被最新版本挤掉。
    let default_item = items
        .iter()
        .find(|item| item.value == default_value)
        .cloned();
    items.truncate(max_len);

    if let Some(default_item) = default_item {
        if !items.iter().any(|item| item.value == default_value) {
            items.pop();
            items.push(default_item);
        }
    }

    items
}

pub(crate) fn limit_keep_values(
    mut items: Vec<VersionOption>,
    max_len: usize,
    required_values: &[&str],
) -> Vec<VersionOption> {
    if max_len == 0 || items.len() <= max_len {
        return items;
    }

    let required_items = required_values
        .iter()
        .filter_map(|value| items.iter().find(|item| item.value == *value).cloned())
        .collect::<Vec<_>>();

    items.truncate(max_len);

    for item in required_items {
        if items.iter().any(|existing| existing.value == item.value) {
            continue;
        }
        if items.len() >= max_len {
            if let Some(index) = (0..items.len())
                .rev()
                .find(|index| !required_values.contains(&items[*index].value.as_str()))
            {
                items.remove(index);
            } else {
                items.pop();
            }
        }
        items.push(item);
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_keeps_project_default_version() {
        let items = ["24.1.0", "22.1.0", "20.19.0"]
            .into_iter()
            .map(|value| option(value, value.to_string(), false, false, "test"))
            .collect::<Vec<_>>();

        let limited = limit_keep_default(items, 2, "20.19.0");

        assert_eq!(limited.len(), 2);
        assert!(limited.iter().any(|item| item.value == "20.19.0"));
    }
}
