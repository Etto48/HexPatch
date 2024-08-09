use mlua::Table;
use ratatui::style::Style;

#[macro_export]
macro_rules! RegisterColorSettings {(
    $(#[$attr:meta])*
    $pub:vis struct $color_settings:ident {
        $(
            $(#[$field_attr:meta])*
            $field_pub:vis $field_name:ident: $field_type:ty,
        )*
    }) => {
        impl $color_settings
        {
            pub fn register_userdata(data: &mut mlua::UserDataRegistry<$crate::app::settings::Settings>)
            {
                $(
                    mlua::UserDataFields::add_field_method_get(data, concat!("color_",stringify!($field_name)), |lua, settings| {
                        $crate::app::settings::register_color_settings_macro::
                            get_style(lua, &settings.color.$field_name)
                    });
                    mlua::UserDataFields::add_field_method_set(data, concat!("color_",stringify!($field_name)), |lua, settings, value| {
                        $crate::app::settings::register_color_settings_macro::
                            set_style(lua, &mut settings.color.$field_name, value)
                    });
                )*
            }
        }
    };
}

pub(super) fn get_style<'lua>(lua: &'lua mlua::Lua, style: &Style) -> mlua::Result<Table<'lua>> {
    let table = lua.create_table()?;
    table.set("fg", get_color(lua, &style.fg)?)?;
    table.set("bg", get_color(lua, &style.bg)?)?;
    table.set("underline", get_color(lua, &style.underline_color)?)?;
    table.set("add_modifier", style.add_modifier.bits())?;
    table.set("sub_modifier", style.sub_modifier.bits())?;
    Ok(table)
}

pub(in crate::app) fn set_style<'lua>(
    _lua: &'lua mlua::Lua,
    style: &mut Style,
    value: Table<'lua>,
) -> mlua::Result<()> {
    match value.get("fg") {
        Ok(value) => set_color(_lua, &mut style.fg, value)?,
        Err(e) => match e {
            mlua::Error::FromLuaConversionError {
                from: "nil",
                to: "Table",
                message: _,
            } => style.fg = None,
            _ => return Err(e),
        },
    }
    match value.get("bg") {
        Ok(value) => set_color(_lua, &mut style.bg, value)?,
        Err(e) => match e {
            mlua::Error::FromLuaConversionError {
                from: "nil",
                to: "Table",
                message: _,
            } => style.bg = None,
            _ => return Err(e),
        },
    }
    match value.get("underline") {
        Ok(value) => set_color(_lua, &mut style.underline_color, value)?,
        Err(e) => match e {
            mlua::Error::FromLuaConversionError {
                from: "nil",
                to: "Table",
                message: _,
            } => style.underline_color = None,
            _ => return Err(e),
        },
    }
    style.add_modifier =
        ratatui::style::Modifier::from_bits_truncate(match value.get::<_, u16>("add_modifier") {
            Ok(value) => value,
            Err(e) => match e {
                mlua::Error::FromLuaConversionError {
                    from: "nil",
                    to: "u16",
                    message: _,
                } => 0,
                _ => return Err(e),
            },
        });
    style.sub_modifier =
        ratatui::style::Modifier::from_bits_truncate(match value.get::<_, u16>("sub_modifier") {
            Ok(value) => value,
            Err(e) => match e {
                mlua::Error::FromLuaConversionError {
                    from: "nil",
                    to: "u16",
                    message: _,
                } => 0,
                _ => return Err(e),
            },
        });
    Ok(())
}

pub(super) fn get_color<'lua>(
    lua: &'lua mlua::Lua,
    color: &Option<ratatui::style::Color>,
) -> mlua::Result<mlua::Value<'lua>> {
    Ok(match color {
        Some(color) => mlua::Value::String(lua.create_string(color.to_string())?),
        None => mlua::Value::Nil,
    })
}

pub(super) fn set_color<'lua>(
    _lua: &'lua mlua::Lua,
    color: &mut Option<ratatui::style::Color>,
    value: mlua::Value<'lua>,
) -> mlua::Result<()> {
    match value {
        mlua::Value::String(value) => {
            let new_color = <ratatui::style::Color as std::str::FromStr>::from_str(value.to_str()?);
            if let Ok(new_color) = new_color {
                *color = Some(new_color);
                Ok(())
            } else {
                Err(mlua::Error::RuntimeError("Invalid color code".to_string()))
            }
        }
        mlua::Value::Integer(value) => {
            let new_color =
                <ratatui::style::Color as std::str::FromStr>::from_str(&value.to_string());
            if let Ok(new_color) = new_color {
                *color = Some(new_color);
                Ok(())
            } else {
                Err(mlua::Error::RuntimeError("Invalid color code".to_string()))
            }
        }
        mlua::Value::Nil => {
            *color = None;
            Ok(())
        }
        ty => Err(mlua::Error::RuntimeError(format!(
            "Invalid color type: {ty:?}"
        ))),
    }
}
