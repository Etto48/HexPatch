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
            pub fn register_userdata(data: &mut mlua::UserDataRegistry<crate::app::settings::Settings>)
            {
                $(
                    mlua::UserDataFields::add_field_method_get(data, concat!("color_",stringify!($field_name),"_fg"), |lua, settings| {
                        crate::app::settings::register_color_settings_macro::
                            get_color(lua, &settings.color.$field_name.fg)
                    });
                    mlua::UserDataFields::add_field_method_set(data, concat!("color_",stringify!($field_name),"_fg"), |lua, settings, value| {
                        crate::app::settings::register_color_settings_macro::
                            set_color(lua, &mut settings.color.$field_name.fg, value)
                    });
                    mlua::UserDataFields::add_field_method_get(data, concat!("color_",stringify!($field_name),"_bg"), |lua, settings| {
                        crate::app::settings::register_color_settings_macro::
                            get_color(lua, &settings.color.$field_name.bg)
                    });
                    mlua::UserDataFields::add_field_method_set(data, concat!("color_",stringify!($field_name),"_bg"), |lua, settings, value| {
                        crate::app::settings::register_color_settings_macro::
                            set_color(lua, &mut settings.color.$field_name.bg, value)
                    });
                )*
            }
        }
    };
}

pub(super) fn get_color<'lua>(lua: &'lua mlua::Lua, color: &Option<ratatui::style::Color>) -> mlua::Result<mlua::Value<'lua>>
{
    Ok(match color
    {
        Some(color) => mlua::Value::String(lua.create_string(color.to_string())?),
        None => mlua::Value::Nil,
    })
}

pub(super) fn set_color<'lua>(_lua: &'lua mlua::Lua, color: &mut Option<ratatui::style::Color>, value: mlua::Value<'lua>) -> mlua::Result<()>
{
    match value
    {
        mlua::Value::String(value) =>
        {
            let new_color = <ratatui::style::Color as std::str::FromStr>::from_str(value.to_str()?);
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
        _ => Err(mlua::Error::RuntimeError("Invalid color type".to_string()))
    }
}