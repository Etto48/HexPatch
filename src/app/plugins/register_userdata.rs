use mlua::{Lua, UserDataMethods};

use crate::app::settings::{color_settings::ColorSettings, key_settings::KeySettings, Settings};

pub fn register_vec_u8(lua: &Lua) -> mlua::Result<()> 
{
    lua.register_userdata_type(|data: &mut mlua::UserDataRegistry<Vec<u8>>|
    {
        data.add_method("len", 
            |_lua, this, ()| 
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
    })?;
    Ok(())
}