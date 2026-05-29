//! 资源版本策略集中定义。
//!
//! 下载源、版本目录、安装/检测流程只引用这里的默认值、保留版本和服务名，
//! 避免在各业务模块重复硬编码。

pub mod defaults {
    pub const NODEJS: &str = "20.19.0";
    pub const JDK: &str = "17";
    pub const MAVEN: &str = "3.9.6";
    pub const MYSQL: &str = "8.0.36";
    pub const IDEA: &str = "2023.3.8";
    pub const NAVICAT: &str = "17";
    pub const REDIS: &str = "5.0.14.1";

    pub fn component(component: &str) -> &'static str {
        match component {
            "nodejs" => NODEJS,
            "jdk" => JDK,
            "maven" => MAVEN,
            "mysql" => MYSQL,
            "idea" => IDEA,
            "navicat" => NAVICAT,
            "redis" => REDIS,
            _ => "",
        }
    }
}

pub mod nodejs {
    use super::defaults;

    pub struct LtsPin {
        pub version: &'static str,
        pub codename: &'static str,
    }

    pub const MAX_OPTIONS: usize = 24;
    pub const CLASSIC_LTS: &[LtsPin] = &[
        LtsPin {
            version: defaults::NODEJS,
            codename: "Iron",
        },
        LtsPin {
            version: "18.20.8",
            codename: "Hydrogen",
        },
        LtsPin {
            version: "16.20.2",
            codename: "Gallium",
        },
        LtsPin {
            version: "14.21.3",
            codename: "Fermium",
        },
        LtsPin {
            version: "12.22.12",
            codename: "Erbium",
        },
        LtsPin {
            version: "10.24.1",
            codename: "Dubnium",
        },
    ];
    pub const REQUIRED_VALUES: &[&str] = &[
        defaults::NODEJS,
        "18.20.8",
        "16.20.2",
        "14.21.3",
        "12.22.12",
        "10.24.1",
    ];
}

pub mod jdk {
    use super::defaults;

    pub const MAX_OPTIONS: usize = 16;
    pub const MIN_MAJOR: u32 = 8;
    pub const MAX_MAJOR: u32 = 26;
    pub const CLASSIC_MAJORS: &[u32] = &[8, 11, 17, 21];
    pub const REQUIRED_VALUES: &[&str] = &["8", "11", defaults::JDK, "21"];
    pub const INSTALL_DIR_PREFIX: &str = "jdk";
    pub const ADOPTIUM_8_FILENAME: &str = "OpenJDK8U-jdk_x64_windows_hotspot_8u492b09.zip";
    pub const ADOPTIUM_8_TUNA_URL: &str =
        "https://mirrors.tuna.tsinghua.edu.cn/Adoptium/8/jdk/x64/windows/OpenJDK8U-jdk_x64_windows_hotspot_8u492b09.zip";
    pub const ADOPTIUM_8_API_URL: &str =
        "https://api.adoptium.net/v3/binary/latest/8/ga/windows/x64/jdk/hotspot/normal/eclipse?project=jdk";
    pub const ADOPTIUM_8_GITHUB_URL: &str =
        "https://github.com/adoptium/temurin8-binaries/releases/latest/download/OpenJDK8U-jdk_x64_windows_hotspot_8u492b09.zip";

    pub fn major_from_version(version: &str) -> String {
        let mut parts = version.split('.');
        let first = parts.next().unwrap_or(defaults::JDK);
        if first == "1" {
            parts.next().unwrap_or(first).to_string()
        } else {
            first.to_string()
        }
    }

    pub fn is_classic_lts(major: u32) -> bool {
        CLASSIC_MAJORS.contains(&major)
    }

    pub fn is_supported_major(major: u32) -> bool {
        (MIN_MAJOR..=MAX_MAJOR).contains(&major)
    }

    pub fn supported_majors() -> std::ops::RangeInclusive<u32> {
        MIN_MAJOR..=MAX_MAJOR
    }

    pub fn install_dir_name(major: &str) -> String {
        format!("{INSTALL_DIR_PREFIX}{major}")
    }

    pub fn label(major: u32, is_lts: bool) -> String {
        if major == 8 && is_lts {
            "JDK 1.8 (LTS)".to_string()
        } else if is_lts {
            format!("JDK {major} (LTS)")
        } else {
            format!("JDK {major}")
        }
    }

    pub fn feature_version(major: &str) -> &'static str {
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
    }
}

pub mod mysql {
    use super::defaults;

    pub const MAX_OPTIONS: usize = 18;
    pub const SUPPORTED_SERIES: &[&str] = &["8.0", "5.7"];
    pub const REQUIRED_VALUES: &[&str] = &[defaults::MYSQL, "5.7.44", "5.7.38"];
    pub const MANAGED_SERVICE_NAMES: &[&str] = &["MySQL80", "MySQL57"];
    pub const DETECT_SERVICE_NAMES: &[&str] =
        &["MySQL80", "MySQL57", "MySQL81", "MySQL90", "MySQL", "MySql"];

    pub fn series(version: &str) -> &'static str {
        SUPPORTED_SERIES
            .iter()
            .copied()
            .find(|series| version.starts_with(&format!("{series}.")))
            .unwrap_or(SUPPORTED_SERIES[0])
    }

    pub fn service_name_for_version(version: &str) -> &'static str {
        match series(version) {
            "5.7" => "MySQL57",
            _ => "MySQL80",
        }
    }

    pub fn directory_name(series: &str) -> String {
        format!("MySQL-{series}")
    }

    pub fn archive_directory_name(series: &str) -> String {
        format!("mysql-{series}")
    }

    pub fn catalog_regex_pattern() -> String {
        let supported = SUPPORTED_SERIES
            .iter()
            .map(|series| series.replace('.', r"\."))
            .collect::<Vec<_>>()
            .join("|");
        format!(r"mysql-((?:{supported})\.\d+)-winx64\.zip")
    }
}
