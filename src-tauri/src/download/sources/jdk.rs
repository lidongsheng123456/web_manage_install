use crate::common::types::MirrorSource;
use crate::common::version_policy::jdk as jdk_policy;

pub fn mirrors(version: &str) -> MirrorSource {
    let major = jdk_policy::major_from_version(version);
    if major == "8" {
        return jdk8_mirrors();
    }

    let feature = jdk_policy::feature_version(&major);

    MirrorSource {
        urls: vec![
            format!("https://repo.huaweicloud.com/openjdk/{feature}/openjdk-{feature}_windows-x64_bin.zip"),
            format!("https://download.java.net/java/GA/jdk{feature}/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-{feature}_windows-x64_bin.zip"),
            format!("https://github.com/adoptium/temurin{major}-binaries/releases/latest/download/OpenJDK{major}U-jdk_x64_windows_hotspot.zip"),
        ],
        filename: format!("openjdk-{major}_windows-x64_bin.zip"),
    }
}

fn jdk8_mirrors() -> MirrorSource {
    MirrorSource {
        urls: vec![
            jdk_policy::ADOPTIUM_8_TUNA_URL.into(),
            jdk_policy::ADOPTIUM_8_API_URL.into(),
            jdk_policy::ADOPTIUM_8_GITHUB_URL.into(),
        ],
        filename: jdk_policy::ADOPTIUM_8_FILENAME.into(),
    }
}
