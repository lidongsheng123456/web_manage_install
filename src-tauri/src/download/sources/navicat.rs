use crate::common::types::MirrorSource;

/// Navicat 中国站直链优先，官方国际站仅作兜底。
pub fn mirrors() -> MirrorSource {
    MirrorSource {
        urls: vec![
            "https://dn.navicat.com.cn/download/navicat17_premium_cs_x64.exe".into(),
            "https://download.navicat.com/download/navicat17_premium_cs_x64.exe".into(),
        ],
        filename: "navicat17_premium_cs_x64.exe".into(),
    }
}
