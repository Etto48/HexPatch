#![allow(clippy::module_inception)]
use std::{io, path::{Path, PathBuf}};

use super::{color_settings::ColorSettings, key_settings::KeySettings};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct Settings
{
    pub color: ColorSettings,
    pub key: KeySettings,
}

impl Settings
{
    pub fn load(path: Option<&Path>) -> Result<Settings, io::Error>
    {
        let path = match path
        {
            Some(path) => path.to_path_buf(),
            None => Self::get_default_settings_path().ok_or(
                io::Error::new(io::ErrorKind::Other, "Could not get default settings path")
            )?
        };

        if !path.exists()
        {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Settings file not found"));
        }

        let settings = match std::fs::read_to_string(&path)
        {
            Ok(settings) => settings,
            Err(e) => return Err(e)
        };

        Ok(match serde_json::from_str(&settings)
        {
            Ok(settings) => settings,
            Err(e) => return Err(io::Error::new(
                io::ErrorKind::InvalidData, 
                format!("Could not parse settings file: {}", e))
            )
        })
    }

    fn get_default_settings_path() -> Option<PathBuf>
    {
        let home = dirs::home_dir()?;
        Some(match std::env::consts::OS
        {
            "windows" => home.join("AppData").join("Local"),
            _ => home.join(".config")
        }.join("HexPatch").join("settings.json"))
    }

    pub fn load_or_create(path: Option<&Path>) -> Result<Settings, String>
    {
        match Self::load(path)
        {
            Ok(settings) => Ok(settings),
            Err(e) => {
                if e.kind() != io::ErrorKind::NotFound
                {
                    Err(format!("Could not load settings: {}", e))
                }
                else
                {
                    let settings = Settings::default();
                    settings.save(path).ok_or(format!("Could not save default settings: {}", e))?;
                    Ok(settings)
                }
            }
        }
    }

    pub fn save(&self, path: Option<&Path>) -> Option<()>
    {
        let path = match path
        {
            Some(path) => path.to_path_buf(),
            None => Self::get_default_settings_path()?
        };

        let settings = serde_json::to_string_pretty(self).ok()?;
        std::fs::create_dir_all(path.parent()?).ok()?;
        std::fs::write(&path, settings).ok()?;
        Some(())
    }

}

#[cfg(test)]
mod test
{
    use super::*;

    #[test]
    fn test_settings_load()
    {
        let settings = Settings::load(Some(Path::new("test/default_settings.json")));
        if let Err(e) = settings
        {
            panic!("Could not load settings: {}", e);
        }
        assert_eq!(settings.unwrap(), Settings::default());
    }

    #[test]
    fn test_settings_partial_load()
    {
        let settings = Settings::load(Some(Path::new("test/partial_default_settings.json")));
        if let Err(e) = settings
        {
            panic!("Could not load settings: {}", e);
        }
        assert_eq!(settings.unwrap(), Settings::default());
    }
}