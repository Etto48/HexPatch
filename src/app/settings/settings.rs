use std::path::{Path, PathBuf};

use super::color_settings::ColorSettings;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Settings
{
    pub color: ColorSettings
}

impl Settings
{
    pub fn load(path: Option<&Path>) -> Result<Settings, String>
    {
        let path = match path
        {
            Some(path) => path.to_path_buf(),
            None => Self::get_default_settings_path().ok_or("Could not get default settings path")?
        };

        if !path.exists()
        {
            return Err("Settings file does not exist".to_string())
        }

        let settings = match std::fs::read_to_string(&path)
        {
            Ok(settings) => settings,
            Err(e) => return Err(format!("Could not read settings file: {}", e))
        };

        Ok(match serde_json::from_str(&settings)
        {
            Ok(settings) => settings,
            Err(e) => return Err(format!("Could not parse settings file: {}", e))
        })
    }

    fn get_default_settings_path() -> Option<PathBuf>
    {
        let home = dirs::home_dir()?;
        Some(match std::env::consts::OS
        {
            "windows" => home.join("AppData").join("Local"),
            _ => home.join(".config")
        }.join("HexPatch").join("settings.toml"))
    }

    pub fn load_or_create(path: Option<&Path>) -> Result<Settings, String>
    {
        match Self::load(path)
        {
            Ok(settings) => Ok(settings),
            Err(e) => {
                let settings = Settings::default();
                settings.save(path).ok_or(format!("Could not save default settings: {}", e))?;
                Ok(settings)
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

impl Default for Settings
{
    fn default() -> Self
    {
        Self
        {
            color: ColorSettings::default()
        }
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
}