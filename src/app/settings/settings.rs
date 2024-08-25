#![allow(clippy::module_inception)]
use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
};

use ratatui::style::Style;
use serde::de::Visitor;
use crate::detect_theme::Theme;

use super::{
    app_settings::AppSettings, color_settings::ColorSettings, key_settings::KeySettings,
    settings_value::SettingsValue,
};

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Settings {
    pub color: ColorSettings,
    pub key: KeySettings,
    pub app: AppSettings,
    pub custom: HashMap<String, SettingsValue>,
}

impl Settings {
    pub fn load(path: Option<&Path>, terminal_theme: Theme) -> Result<Settings, io::Error> {
        let path = match path {
            Some(path) => path.to_path_buf(),
            None => Self::get_default_settings_path().ok_or(io::Error::new(
                io::ErrorKind::Other,
                "Could not get default settings path",
            ))?,
        };

        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Settings file not found",
            ));
        }

        let settings = match std::fs::read_to_string(&path) {
            Ok(settings) => settings,
            Err(e) => return Err(e),
        };

        let mut deserializer = serde_json::Deserializer::from_str(&settings);

        Ok(match Settings::custom_deserialize(&mut deserializer, terminal_theme) {
            Ok(settings) => settings,
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Could not parse settings file: {}", e),
                ))
            }
        })
    }

    fn get_default_settings_path() -> Option<PathBuf> {
        let config = dirs::config_dir()?;
        Some(config.join("HexPatch").join("settings.json"))
    }

    pub fn load_or_create(path: Option<&Path>, terminal_theme: Theme) -> Result<Settings, String> {
        match Self::load(path, terminal_theme) {
            Ok(settings) => Ok(settings),
            Err(e) => {
                if e.kind() != io::ErrorKind::NotFound {
                    Err(format!("Could not load settings: {}", e))
                } else {
                    let settings = Settings::default();
                    if path.is_some() {
                        settings
                        .save(path)
                        .ok_or(format!("Could not save default settings: {}", e))?;
                    }
                    Ok(settings)
                }
            }
        }
    }

    pub fn save(&self, path: Option<&Path>) -> Option<()> {
        let path = match path {
            Some(path) => path.to_path_buf(),
            None => Self::get_default_settings_path()?,
        };

        let settings = serde_json::to_string_pretty(self).ok()?;
        std::fs::create_dir_all(path.parent()?).ok()?;
        std::fs::write(&path, settings).ok()?;
        Some(())
    }
}

struct SettingsVisitor {
    theme: Theme,
}

impl<'de> Visitor<'de> for SettingsVisitor {
    type Value = Settings;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid Settings struct")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>, {
        let mut color_settings: Option<HashMap<String, Style>> = None;
        let mut key_settings: Option<KeySettings> = None;
        let mut app_settings: Option<AppSettings> = None;
        let mut custom_settings: Option<HashMap<String, SettingsValue>> = None;

        while let Some(key) = map.next_key()? {
            match key {
                "color" => {
                    if color_settings.is_some() {
                        return Err(serde::de::Error::duplicate_field("color"));
                    }
                    color_settings = Some(map.next_value()?);
                },
                "key" => {
                    if key_settings.is_some() {
                        return Err(serde::de::Error::duplicate_field("key"));
                    }
                    key_settings = Some(map.next_value()?);
                },
                "app" => {
                    if app_settings.is_some() {
                        return Err(serde::de::Error::duplicate_field("app"));
                    }
                    app_settings = Some(map.next_value()?);
                },
                "custom" => {
                    if custom_settings.is_some() {
                        return Err(serde::de::Error::duplicate_field("custom"));
                    }
                    custom_settings = Some(map.next_value()?);
                },
                _ => {
                    return Err(serde::de::Error::unknown_field(key, &["color", "key", "app", "custom"]));
                },
            }
        }
        let key_settings = key_settings.unwrap_or_default();
        let app_settings = app_settings.unwrap_or_default();
        let custom_settings = custom_settings.unwrap_or_default();
        let color_settings = ColorSettings::from_map(&color_settings.unwrap_or_default(), &app_settings, self.theme).map_err(
            |e| serde::de::Error::custom(e),
        )?;

        Ok(Self::Value {
            color: color_settings,
            key: key_settings,
            app: app_settings,
            custom: custom_settings,
        })
    }

    
}

impl Settings {
    fn custom_deserialize<'de, D>(deserializer: D, theme: Theme) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        deserializer.deserialize_map(SettingsVisitor{theme})
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            color: ColorSettings::get_default_theme(Theme::Dark),
            key: KeySettings::default(),
            app: AppSettings::default(),
            custom: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::style::{Color, Style};

    use super::*;

    #[test]
    fn test_settings_load() {
        let settings = Settings::load(Some(Path::new("test/default_settings.json")), Theme::Dark);
        if let Err(e) = settings {
            panic!("Could not load settings: {}", e);
        }
        assert_eq!(settings.unwrap(), Settings::default());
    }

    #[test]
    fn test_settings_partial_load() {
        let settings = Settings::load(Some(Path::new("test/partial_default_settings.json")), Theme::Dark);
        if let Err(e) = settings {
            panic!("Could not load settings: {}", e);
        }
        assert_eq!(settings.unwrap(), Settings::default());
    }

    #[test]
    fn test_settings_load_custom() {
        let settings = Settings::load(Some(Path::new("test/custom_settings.json")), Theme::Dark);
        if let Err(e) = settings {
            panic!("Could not load settings: {}", e);
        }
        let mut expected = Settings::default();
        expected
            .custom
            .insert("plugin1.value1".to_string(), SettingsValue::from("value1"));
        expected
            .custom
            .insert("plugin1.value2".to_string(), SettingsValue::from(2));
        expected
            .custom
            .insert("plugin2.value1".to_string(), SettingsValue::from(3.0));
        expected
            .custom
            .insert("plugin2.value2".to_string(), SettingsValue::from(true));
        expected.custom.insert(
            "plugin3.value1".to_string(),
            SettingsValue::from(Style::default().fg(Color::Red)),
        );
        expected.custom.insert(
            "plugin3.value2".to_string(),
            SettingsValue::from(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        );
    }
}
