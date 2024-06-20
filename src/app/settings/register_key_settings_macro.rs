use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use mlua::{Lua, Table};

use super::key_settings::KeySettings;

#[macro_export]
macro_rules! RegisterKeySettings {(
    $(#[$attr:meta])*
    $pub:vis struct $key_settings:ident {
        $(
            $(#[$field_attr:meta])*
            $field_pub:vis $field_name:ident: $field_type:ty,
        )*
    }) => {
        impl $key_settings
        {
            pub fn register_userdata(data: &mut mlua::UserDataRegistry<$crate::app::settings::Settings>)
            {
                $(
                    mlua::UserDataFields::add_field_method_get(data, concat!("key_",stringify!($field_name)), |lua, settings| {
                        $crate::app::settings::register_key_settings_macro::
                            get_key(lua, &settings.key.$field_name)
                    });
                    mlua::UserDataFields::add_field_method_set(data, concat!("key_",stringify!($field_name)), |lua, settings, value| {
                        $crate::app::settings::register_key_settings_macro::
                            set_key(lua, &mut settings.key.$field_name, value)
                    });
                )*
            }
        }
    };
}

pub fn key_event_to_lua<'lua>(lua: &'lua Lua, key: &KeyEvent) -> mlua::Result<Table<'lua>>
{
    let ret = lua.create_table()?;
    ret.set("code", KeySettings::key_code_to_string(key.code))?;
    ret.set("modifiers", key.modifiers.bits())?;
    ret.set("kind", format!("{:?}",key.kind))?;
    ret.set("state", key.state.bits())?;
    Ok(ret)
}

pub fn lua_to_key_event(_lua: &Lua, table: &mlua::Table) -> mlua::Result<KeyEvent>
{
    let code = match table.get::<_,String>("code") {
        Ok(value) => 
            KeySettings::string_to_key_code(&value).map_err(mlua::Error::RuntimeError)?,
        Err(e) => match e
        {
            mlua::Error::FromLuaConversionError { from: "nil", to: "String", message: _ } => KeyCode::Null,
            _ => return Err(e)
        }
    };
    let modifiers = match table.get::<_,u8>("modifiers")
    {
        Ok(value) => value,
        Err(e) => match e
        {
            mlua::Error::FromLuaConversionError { from: "nil", to: "u8", message: _ } => 0,
            _ => return Err(e)
        }
    };
    let kind = match table.get::<_,String>("kind") {
        Ok(value) => 
            KeySettings::string_to_key_event_kind(&value).map_err(mlua::Error::RuntimeError)?,
        Err(e) => match e
        {
            mlua::Error::FromLuaConversionError { from: "nil", to: "String", message: _ } => KeyEventKind::Press,
            _ => return Err(e)
        }
    };
    let state = match table.get::<_,u8>("state")
    {
        Ok(value) => value,
        Err(e) => match e
        {
            mlua::Error::FromLuaConversionError { from: "nil", to: "u8", message: _ } => 0,
            _ => return Err(e)
        }
    };
    Ok(KeyEvent {
        code,
        modifiers: crossterm::event::KeyModifiers::from_bits(modifiers).unwrap(),
        kind,
        state: crossterm::event::KeyEventState::from_bits(state).unwrap(),
    })
}

pub(super) fn get_key<'lua>(lua: &'lua Lua, key: &KeyEvent) -> mlua::Result<mlua::Value<'lua>>
{
    key_event_to_lua(lua, key).map(mlua::Value::Table)
}

pub(super) fn set_key<'lua>(lua: &'lua Lua, color: &mut KeyEvent, value: mlua::Value<'lua>) -> mlua::Result<()>
{
    if let Some(table) = value.as_table()
    {
        *color = lua_to_key_event(lua, table)?;
        Ok(())
    } else {
        Err(mlua::Error::RuntimeError("Expected table".to_string()))
    }
}