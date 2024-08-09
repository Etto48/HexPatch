use mlua::{FromLua, IntoLua, Lua, Table, UserDataFields, UserDataMethods};
use ratatui::{
    layout::Alignment,
    text::{Span, Text},
};

use crate::app::settings::{
    app_settings::AppSettings, color_settings::ColorSettings, key_settings::KeySettings,
    register_color_settings_macro::set_style, settings_value::SettingsValue, Settings,
};

pub fn register_vec_u8(lua: &Lua) -> mlua::Result<()> {
    lua.register_userdata_type(|data: &mut mlua::UserDataRegistry<Vec<u8>>| {
        data.add_field_method_get("len", |_lua, this| Ok(this.len()));
        data.add_method("get", |_lua, this, index: usize| {
            if let Some(byte) = this.get(index) {
                Ok(*byte)
            } else {
                Err(mlua::Error::RuntimeError("Index out of bounds".to_string()))
            }
        });
        data.add_method_mut("set", |_lua, this, (index, value): (usize, u8)| {
            if let Some(byte) = this.get_mut(index) {
                let old_value = *byte;
                *byte = value;
                Ok(old_value)
            } else {
                Err(mlua::Error::RuntimeError("Index out of bounds".to_string()))
            }
        });
    })?;
    Ok(())
}

pub fn register_text(lua: &Lua) -> mlua::Result<()> {
    lua.register_userdata_type(|data: &mut mlua::UserDataRegistry<Text>| {
        data.add_method_mut("push_line", |_lua, this, value: String| {
            let span = Span::styled(value, this.style);
            let line = match &this.alignment {
                Some(alignment) => match alignment {
                    Alignment::Left => span.into_left_aligned_line(),
                    Alignment::Center => span.into_centered_line(),
                    Alignment::Right => span.into_right_aligned_line(),
                },
                None => span.into(),
            };
            this.push_line(line);
            Ok(())
        });
        data.add_method_mut("push_span", |_lua, this, value: String| {
            this.push_span(Span::styled(value, this.style));
            Ok(())
        });
        data.add_method_mut("set_style", |lua, this, value: Table| {
            set_style(lua, &mut this.style, value)
        });
        data.add_method_mut("reset_style", |_lua, this, ()| {
            this.style = Default::default();
            Ok(())
        });
        data.add_method_mut("set_alignment", |_lua, this, value: String| {
            match value.as_str() {
                "left" => this.alignment = Some(Alignment::Left),
                "center" => this.alignment = Some(Alignment::Center),
                "right" => this.alignment = Some(Alignment::Right),
                _ => return Err(mlua::Error::RuntimeError("Invalid alignment".to_string())),
            }
            Ok(())
        });
        data.add_method_mut("reset_alignment", |_lua, this, ()| {
            this.alignment = None;
            Ok(())
        });
    })?;
    Ok(())
}

pub fn register_string(lua: &Lua) -> mlua::Result<()> {
    lua.register_userdata_type(|data: &mut mlua::UserDataRegistry<String>| {
        data.add_method("get", |_lua, this, ()| Ok(this.clone()));
        data.add_method_mut("set", |_lua, this, value: String| {
            let old_value = std::mem::replace(this, value);
            Ok(old_value)
        });
    })?;
    Ok(())
}

pub fn register_usize(lua: &Lua) -> mlua::Result<()> {
    lua.register_userdata_type(|data: &mut mlua::UserDataRegistry<usize>| {
        data.add_method("get", |_lua, this, ()| Ok(*this));
        data.add_method_mut("set", |_lua, this, value: usize| {
            let old_value = std::mem::replace(this, value);
            Ok(old_value)
        });
    })?;
    Ok(())
}

pub fn register_settings(lua: &Lua) -> mlua::Result<()> {
    lua.register_userdata_type(|data: &mut mlua::UserDataRegistry<Settings>| {
        ColorSettings::register_userdata(data);
        KeySettings::register_userdata(data);
        AppSettings::register_userdata(data);
        data.add_method("get_custom", |_lua, this, key: String| {
            Ok(this.custom.get(&key).cloned())
        });
        data.add_method_mut(
            "set_custom",
            |lua, this, (key, value): (String, mlua::Value)| {
                if let Some(old_value) = this.custom.get_mut(&key) {
                    let old_value_copy = old_value.clone();
                    if value.is_nil() {
                        this.custom.remove(&key);
                    } else {
                        *old_value = SettingsValue::from_lua(value, lua)?;
                    }
                    old_value_copy.into_lua(lua)
                } else {
                    if !value.is_nil() {
                        this.custom
                            .insert(key, SettingsValue::from_lua(value, lua)?);
                    }
                    Ok(mlua::Value::Nil)
                }
            },
        );
    })?;
    Ok(())
}
