use serde::{Deserialize, Serialize};

macro_rules! EnumValues {(
    $(#[$attr:meta])*
    $pub:vis enum $enum_name:ident {
        $(
            $(#[$field_attr:meta])*
            $field_name:ident,
        )*
    }) => {
        impl $enum_name {
            $pub const VALUES: &'static [$enum_name] = &[
                $(
                    $enum_name::$field_name,
                )*
            ];
        }
    }
}
macro_rules! LocaleCode {(
    $(#[$attr:meta])*
    $pub:vis enum $enum_name:ident {
        $(
            $(#[$field_attr:meta])*
            $field_name:ident,
        )*
    }) => {
        impl $enum_name {
            $pub const fn code(&self) -> &'static str {
                match self {
                    $(
                        $enum_name::$field_name => const_str::replace!(stringify!($field_name),"_", "-"),
                    )*
                }
            }
        }
    }
}

#[allow(non_camel_case_types)]
#[non_exhaustive]
#[derive(EnumValues!)]
#[derive(LocaleCode!)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Locale {
    #[default]
    auto, // "auto" has to be the first variant to make match_locale work correctly
    en,
    #[serde(rename = "it-IT")]
    it_IT,
    #[serde(rename = "fr-FR")]
    fr_FR,
    #[serde(rename = "es-ES")]
    es_ES,
    #[serde(rename = "de-DE")]
    de_DE,
    #[serde(rename = "ja-JP")]
    ja_JP,
    #[serde(rename = "zh-CN")]
    zh_CN,
    #[serde(rename = "zh-TW")]
    zh_TW,
    #[serde(rename = "zh-HK")]
    zh_HK,
    #[serde(rename = "tr-TR")]
    tr_TR,
}

impl Locale {
    pub const fn name(&self) -> &'static str {
        match self {
            Locale::auto => "Auto",
            Locale::en => "English",
            Locale::it_IT => "Italiano (Italia)",
            Locale::fr_FR => "Français (France)",
            Locale::es_ES => "Español (España)",
            Locale::de_DE => "Deutsch (Deutschland)",
            Locale::ja_JP => "日本語 (日本)",
            Locale::zh_CN => "中文 (中国)",
            Locale::zh_TW => "中文 (台灣)",
            Locale::zh_HK => "中文 (香港)",
            Locale::tr_TR => "Türkçe (Türkiye)",
        }
    }
    pub fn language(&self) -> &'static str {
        self.code().split('-').next().unwrap()
    }
    pub fn match_locale(&self, locale: &str) -> Option<Locale> {
        // Try to match exact locale first
        // Skip "auto" as it's not a valid locale code
        for l in &Locale::VALUES[1..] {
            if locale.starts_with(l.code()) {
                return Some(*l);
            }
        }
        // If no exact match, try to match by language code
        for l in &Locale::VALUES[1..] {
            if locale.starts_with(l.language()) {
                return Some(*l);
            }
        }
        None
    }
    pub fn apply(&self) {
        match self {
            Locale::auto => {
                let locale = 'found: {
                    for locale in sys_locale::get_locales() {
                        if let Some(locale) = self.match_locale(&locale) {
                            break 'found locale;
                        }
                    }
                    Locale::en // Fallback to English if no match found
                };
                rust_i18n::set_locale(locale.code());
            }
            _ => {
                rust_i18n::set_locale(self.code());
            }
        }
    }
}
