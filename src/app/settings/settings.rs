#![allow(clippy::module_inception)]
use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
};

use super::{
    app_settings::AppSettings, color_settings::ColorSettings, key_settings::KeySettings,
    settings_value::SettingsValue,
};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct Settings {
    pub color: ColorSettings,
    pub key: KeySettings,
    pub app: AppSettings,
    pub custom: HashMap<String, SettingsValue>,
}

impl Settings {
    pub fn load(path: Option<&Path>) -> Result<Settings, io::Error> {
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

        Ok(match serde_json::from_str(&settings) {
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

    pub fn load_or_create(path: Option<&Path>) -> Result<Settings, String> {
        match Self::load(path) {
            Ok(settings) => Ok(settings),
            Err(e) => {
                if e.kind() != io::ErrorKind::NotFound {
                    Err(format!("Could not load settings: {}", e))
                } else {
                    let settings = Settings::default();
                    settings
                        .save(path)
                        .ok_or(format!("Could not save default settings: {}", e))?;
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

#[cfg(test)]
mod test {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::style::{Color, Style};

    use super::*;

    #[test]
    fn test_settings_load() {
        let settings = Settings::load(Some(Path::new("test/default_settings.json")));
        if let Err(e) = settings {
            panic!("Could not load settings: {}", e);
        }
        assert_eq!(settings.unwrap(), Settings::default());
    }

    #[test]
    fn test_settings_partial_load() {
        let settings = Settings::load(Some(Path::new("test/partial_default_settings.json")));
        if let Err(e) = settings {
            panic!("Could not load settings: {}", e);
        }
        assert_eq!(settings.unwrap(), Settings::default());
    }

    #[test]
    fn test_settings_load_custom() {
        let settings = Settings::load(Some(Path::new("test/custom_settings.json")));
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
