use crossterm::event::KeyEvent;
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
            pub fn register_userdata(data: &mut mlua::UserDataRegistry<crate::app::settings::Settings>)
            {
                $(
                    mlua::UserDataFields::add_field_method_get(data, concat!("key_",stringify!($field_name)), |lua, settings| {
                        crate::app::settings::register_key_settings_macro::
                            get_key(lua, &settings.key.$field_name)
                    });
                    mlua::UserDataFields::add_field_method_set(data, concat!("key_",stringify!($field_name)), |lua, settings, value| {
                        crate::app::settings::register_key_settings_macro::
                            set_key(lua, &mut settings.key.$field_name, value)
                    });
                )*
            }
        }
    };
}

fn key_event_to_lua<'lua>(lua: &'lua Lua, key: &KeyEvent) -> mlua::Result<Table<'lua>>
{
    let ret = lua.create_table()?;
    ret.set("code", KeySettings::key_code_to_string(key.code))?;
    ret.set("modifiers", key.modifiers.bits())?;
    ret.set("kind", format!("{:?}",key.kind))?;
    ret.set("state", key.state.bits())?;
    Ok(ret)
}

fn lua_to_key_event<'lua>(_lua: &'lua Lua, table: &mlua::Table) -> mlua::Result<KeyEvent>
{
    let code = KeySettings::string_to_key_code(&table.get::<_,String>("code")?)
        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
    let modifiers = table.get::<_,u8>("modifiers")?;
    let kind = KeySettings::string_to_key_event_kind(&table.get::<_,String>("kind")?)
        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
    let state = table.get::<_,u8>("state")?;
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