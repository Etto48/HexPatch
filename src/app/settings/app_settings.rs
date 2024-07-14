use mlua::UserDataRegistry;
use serde::{Deserialize, Serialize};

use super::Settings;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub history_limit: usize,
    pub log_limit: usize,
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
        }
    }
}
