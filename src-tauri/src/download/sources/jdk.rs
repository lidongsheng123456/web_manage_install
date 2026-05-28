use crate::common::types::MirrorSource;

/// OpenJDK ZIP：华为云镜像优先，官方 java.net 和 Adoptium 兜底。
pub fn mirrors(version: &str) -> MirrorSource {
    let major = version.split('.').next().unwrap_or("17");
    let feature = feature_version(major);

    MirrorSource {
        urls: vec![
            format!("https://repo.huaweicloud.com/openjdk/{feature}/openjdk-{feature}_windows-x64_bin.zip"),
            format!("https://download.java.net/java/GA/jdk{feature}/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-{feature}_windows-x64_bin.zip"),
            format!("https://github.com/adoptium/temurin{major}-binaries/releases/latest/download/OpenJDK{major}U-jdk_x64_windows_hotspot.zip"),
        ],
        filename: format!("openjdk-{major}_windows-x64_bin.zip"),
    }
}

fn feature_version(major: &str) -> String {
    match major {
        "9" => "9.0.4",
        "10" => "10.0.2",
        "11" => "11.0.2",
        "12" => "12.0.2",
        "13" => "13.0.2",
        "14" => "14.0.2",
        "15" => "15.0.2",
        "16" => "16.0.2",
        "17" => "17.0.2",
        "18" => "18.0.2",
        "19" => "19.0.2",
        "20" => "20.0.2",
        "21" => "21.0.2",
        "22" => "22.0.2",
        "23" => "23.0.2",
        "24" => "24.0.2",
        "25" => "25.0.2",
        "26" => "26.0.1",
        _ => "17.0.2",
    }
    .to_string()
}
