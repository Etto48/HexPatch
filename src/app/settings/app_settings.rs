use mlua::UserDataRegistry;
use serde::{Deserialize, Serialize};

use super::{locale::Locale, theme_preference::ThemePreference, verbosity::Verbosity, Settings};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub history_limit: usize,
    pub log_limit: usize,
    pub log_level: Verbosity,
    pub theme: ThemePreference,
    pub locale: Locale,
}

impl AppSettings {
    pub fn register_userdata(data: &mut UserDataRegistry<Settings>) {
        mlua::UserDataFields::add_field_method_get(data, "app_history_limit", |_lua, settings| {
            Ok(settings.app.history_limit)
        });
        mlua::UserDataFields::add_field_method_set(
            data,
            "app_history_limit",
            |_lua, settings, value| {
                settings.app.history_limit = value;
                Ok(())
            },
        );
        mlua::UserDataFields::add_field_method_get(data, "app_log_limit", |_lua, settings| {
            Ok(settings.app.log_limit)
        });
        mlua::UserDataFields::add_field_method_set(
            data,
            "app_log_limit",
            |_lua, settings, value| {
                settings.app.log_limit = value;
                Ok(())
            },
        );
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            history_limit: 1024,
            log_limit: 1024,
            log_level: Verbosity::default(),
            theme: ThemePreference::default(),
            locale: Locale::default(),
        }
    }
}
