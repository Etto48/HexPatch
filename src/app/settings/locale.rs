use serde::{Deserialize, Serialize};

#[allow(non_camel_case_types)]
#[non_exhaustive]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Locale {
    #[default]
    #[serde(rename = "en")]
    en,
    #[serde(rename = "it-IT")]
    it_IT,
    #[serde(rename = "fr-FR")]
    fr_FR,
    #[serde(rename = "es-ES")]
    es_ES,
    #[serde(rename = "ja-JP")]
    ja_JP,
    #[serde(rename = "zh-CN")]
    zh_CN,
    #[serde(rename = "zh-TW")]
    zh_TW,
    #[serde(rename = "zh-HK")]
    zh_HK,
}

impl Locale {
    pub fn name(&self) -> &'static str {
        match self {
            Locale::en => "English",
            Locale::it_IT => "Italiano (Italia)",
            Locale::fr_FR => "Français (France)",
            Locale::es_ES => "Español (España)",
            Locale::ja_JP => "日本語 (日本)",
            Locale::zh_CN => "中文 (中国)",
            Locale::zh_TW => "中文 (台灣)",
            Locale::zh_HK => "中文 (香港)",
        }
    }
    pub fn code(&self) -> &'static str {
        match self {
            Locale::en => "en",
            Locale::it_IT => "it-IT",
            Locale::fr_FR => "fr-FR",
            Locale::es_ES => "es-ES",
            Locale::ja_JP => "ja-JP",
            Locale::zh_CN => "zh-CN",
            Locale::zh_TW => "zh-TW",
            Locale::zh_HK => "zh-HK",
        }
    }
}

