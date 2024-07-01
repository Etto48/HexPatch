use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseEvent};
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

fn key_modifiers_to_table(lua: &Lua, modifiers: KeyModifiers) -> mlua::Result<Table> {
    let ret = lua.create_table()?;
    ret.set("alt", modifiers.contains(KeyModifiers::ALT))?;
    ret.set("control", modifiers.contains(KeyModifiers::CONTROL))?;
    ret.set("hyper", modifiers.contains(KeyModifiers::HYPER))?;
    ret.set("meta", modifiers.contains(KeyModifiers::META))?;
    ret.set("shift", modifiers.contains(KeyModifiers::SHIFT))?;
    ret.set("super", modifiers.contains(KeyModifiers::SUPER))?;
    Ok(ret)
}

fn key_state_to_table(lua: &Lua, state: KeyEventState) -> mlua::Result<Table> {
    let ret = lua.create_table()?;
    ret.set("caps_lock", state.contains(KeyEventState::CAPS_LOCK))?;
    ret.set("keypad", state.contains(KeyEventState::KEYPAD))?;
    ret.set("num_lock", state.contains(KeyEventState::NUM_LOCK))?;
    Ok(ret)
}

pub fn mouse_event_to_lua<'lua>(lua: &'lua Lua, mouse: &MouseEvent) -> mlua::Result<Table<'lua>> {
    let ret = lua.create_table()?;
    ret.set("kind", format!("{:?}", mouse.kind))?;
    ret.set("column", mouse.column)?;
    ret.set("row", mouse.row)?;

    let modifiers = key_modifiers_to_table(lua, mouse.modifiers)?;
    ret.set("modifiers", modifiers)?;
    Ok(ret)
}

pub fn key_event_to_lua<'lua>(lua: &'lua Lua, key: &KeyEvent) -> mlua::Result<Table<'lua>> {
    let ret = lua.create_table()?;
    ret.set("code", KeySettings::key_code_to_string(key.code))?;

    let modifiers = key_modifiers_to_table(lua, key.modifiers)?;
    ret.set("modifiers", modifiers)?;

    ret.set("kind", format!("{:?}", key.kind))?;

    let state = key_state_to_table(lua, key.state)?;
    ret.set("state", state)?;

    Ok(ret)
}

pub fn lua_to_key_event(_lua: &Lua, table: &mlua::Table) -> mlua::Result<KeyEvent> {
    let code = match table.get::<_, String>("code") {
        Ok(value) => KeySettings::string_to_key_code(&value).map_err(mlua::Error::RuntimeError)?,
        Err(e) => match e {
            mlua::Error::FromLuaConversionError {
                from: "nil",
                to: "String",
                message: _,
            } => KeyCode::Null,
            _ => return Err(e),
        },
    };

    let mut modifiers = KeyModifiers::NONE;
    if let Ok(modifiers_table) = table.get::<_, Table>("modifiers") {
        if modifiers_table.get::<_, bool>("alt").unwrap_or(false) {
            modifiers |= KeyModifiers::ALT;
        }
        if modifiers_table.get::<_, bool>("control").unwrap_or(false) {
            modifiers |= KeyModifiers::CONTROL;
        }
        if modifiers_table.get::<_, bool>("hyper").unwrap_or(false) {
            modifiers |= KeyModifiers::HYPER;
        }
        if modifiers_table.get::<_, bool>("meta").unwrap_or(false) {
            modifiers |= KeyModifiers::META;
        }
        if modifiers_table.get::<_, bool>("shift").unwrap_or(false) {
            modifiers |= KeyModifiers::SHIFT;
        }
        if modifiers_table.get::<_, bool>("super").unwrap_or(false) {
            modifiers |= KeyModifiers::SUPER;
        }
    }

    let kind = match table.get::<_, String>("kind") {
        Ok(value) => {
            KeySettings::string_to_key_event_kind(&value).map_err(mlua::Error::RuntimeError)?
        }
        Err(e) => match e {
            mlua::Error::FromLuaConversionError {
                from: "nil",
                to: "String",
                message: _,
            } => KeyEventKind::Press,
            _ => return Err(e),
        },
    };

    let mut state = KeyEventState::NONE;
    if let Ok(state_table) = table.get::<_, Table>("state") {
        if state_table.get::<_, bool>("caps_lock").unwrap_or(false) {
            state |= KeyEventState::CAPS_LOCK;
        }
        if state_table.get::<_, bool>("keypad").unwrap_or(false) {
            state |= KeyEventState::KEYPAD;
        }
        if state_table.get::<_, bool>("num_lock").unwrap_or(false) {
            state |= KeyEventState::NUM_LOCK;
        }
    }

    Ok(KeyEvent {
        code,
        modifiers,
        kind,
        state,
    })
}

pub(super) fn get_key<'lua>(lua: &'lua Lua, key: &KeyEvent) -> mlua::Result<mlua::Value<'lua>> {
    key_event_to_lua(lua, key).map(mlua::Value::Table)
}

pub(super) fn set_key<'lua>(
    lua: &'lua Lua,
    color: &mut KeyEvent,
    value: mlua::Value<'lua>,
) -> mlua::Result<()> {
    if let Some(table) = value.as_table() {
        *color = lua_to_key_event(lua, table)?;
        Ok(())
    } else {
        Err(mlua::Error::RuntimeError("Expected table".to_string()))
    }
}
