use mlua::{FromLua, IntoLua, Lua, UserDataFields, UserDataMethods};

use crate::app::settings::{color_settings::ColorSettings, key_settings::KeySettings, settings_value::SettingsValue, Settings};

pub fn register_vec_u8(lua: &Lua) -> mlua::Result<()> 
{
    lua.register_userdata_type(|data: &mut mlua::UserDataRegistry<Vec<u8>>|
    {
        data.add_field_method_get("len", 
            |_lua, this| 
            Ok(this.len())
        );
        data.add_method("get", |_lua, this, index: usize| {
            if let Some(byte) = this.get(index)
            {
                Ok(*byte)
            } else {
                Err(mlua::Error::RuntimeError("Index out of bounds".to_string()))
            }
        });
        data.add_method_mut("set", |_lua, this, (index, value): (usize, u8)| {
            if let Some(byte) = this.get_mut(index)
            {
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

pub fn register_settings(lua: &Lua) -> mlua::Result<()> 
{
    lua.register_userdata_type(|data: &mut mlua::UserDataRegistry<Settings>| 
    {
        ColorSettings::register_userdata(data);
        KeySettings::register_userdata(data);
        data.add_method("get_custom", 
            |_lua, this, key: String| 
            Ok(this.custom.get(&key).cloned())
        );
        data.add_method_mut("set_custom", 
            |lua, this, (key, value): (String, mlua::Value)| 
            {
                if let Some(old_value) = this.custom.get_mut(&key)
                {
                    let old_value_copy = old_value.clone();
                    if value.is_nil()
                    {
                        this.custom.remove(&key);
                    }
                    else
                    {
                        *old_value = SettingsValue::from_lua(value, lua)?;
                    }
                    old_value_copy.into_lua(lua)
                }
                else
                {
                    if !value.is_nil()
                    {
                        this.custom.insert(key, SettingsValue::from_lua(value, lua)?);
                    }
                    Ok(mlua::Value::Nil)
                }
            }
        );
    })?;
    Ok(())
}

pub fn register_logger(lua: &Lua) -> mlua::Result<()> 
{
    lua.register_userdata_type(|data: &mut mlua::UserDataRegistry<crate::app::log::logger::Logger>| 
    {
        data.add_method_mut("log", 
            |_lua, this, (level, message): (u8, String)| 
            {
                this.log(level.into(), &message);
                Ok(())
            }
        );
    })?;
    Ok(())
}