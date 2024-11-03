use std::fmt::{Display, Formatter};

use crossterm::event::{KeyCode, KeyEvent, KeyEventState, KeyModifiers};
use mlua::{FromLua, IntoLua};
use ratatui::style::{Modifier, Style};
use serde::{ser::SerializeMap, Deserialize, Serialize};

use super::{
    key_settings::KeySettings,
    register_color_settings_macro::{get_style, set_style},
    register_key_settings_macro::{key_event_to_lua, lua_to_key_event},
};

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Style(Style),
    Key(KeyEvent),
}

impl From<bool> for SettingsValue {
    fn from(value: bool) -> Self {
        SettingsValue::Bool(value)
    }
}

impl From<i64> for SettingsValue {
    fn from(value: i64) -> Self {
        SettingsValue::Int(value)
    }
}

impl From<f64> for SettingsValue {
    fn from(value: f64) -> Self {
        SettingsValue::Float(value)
    }
}

impl From<String> for SettingsValue {
    fn from(value: String) -> Self {
        SettingsValue::String(value)
    }
}

impl From<&str> for SettingsValue {
    fn from(value: &str) -> Self {
        SettingsValue::String(value.to_string())
    }
}

impl From<Style> for SettingsValue {
    fn from(value: Style) -> Self {
        SettingsValue::Style(value)
    }
}

impl From<KeyEvent> for SettingsValue {
    fn from(value: KeyEvent) -> Self {
        SettingsValue::Key(value)
    }
}

impl Display for SettingsValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingsValue::Bool(value) => write!(f, "{}", value),
            SettingsValue::Int(value) => write!(f, "{}", value),
            SettingsValue::Float(value) => write!(f, "{}", value),
            SettingsValue::String(value) => write!(f, "{}", value),
            SettingsValue::Style(value) => {
                write!(
                    f,
                    "{{fg:{},",
                    value.fg.map_or("None".to_string(), |c| c.to_string())
                )?;
                write!(
                    f,
                    "bg:{},",
                    value.bg.map_or("None".to_string(), |c| c.to_string())
                )?;
                write!(
                    f,
                    "underline:{},",
                    value
                        .underline_color
                        .map_or("None".to_string(), |c| c.to_string())
                )?;
                write!(f, "add_modifier:{},", value.add_modifier.bits())?;
                write!(f, "sub_modifier:{}}}", value.sub_modifier.bits())?;
                Ok(())
            }
            SettingsValue::Key(value) => {
                write!(f, "{{code:{},", KeySettings::key_code_to_string(value.code))?;
                write!(f, "modifiers:{},", value.modifiers.bits())?;
                write!(
                    f,
                    "kind:{},",
                    KeySettings::key_event_kind_to_string(value.kind)
                )?;
                write!(f, "state:{}}}", value.state.bits())?;
                Ok(())
            }
        }
    }
}

impl Serialize for SettingsValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            SettingsValue::Bool(value) => serializer.serialize_bool(*value),
            SettingsValue::Int(value) => serializer.serialize_i64(*value),
            SettingsValue::Float(value) => serializer.serialize_f64(*value),
            SettingsValue::String(value) => serializer.serialize_str(value),
            SettingsValue::Style(value) => {
                let len = 2
                    + value.fg.is_some() as usize
                    + value.bg.is_some() as usize
                    + value.underline_color.is_some() as usize;
                let mut map = serializer.serialize_map(Some(len))?;
                if let Some(fg) = value.fg {
                    map.serialize_key("fg")?;
                    map.serialize_value(&fg.to_string())?;
                }
                if let Some(bg) = value.bg {
                    map.serialize_key("bg")?;
                    map.serialize_value(&bg.to_string())?;
                }
                if let Some(underline) = value.underline_color {
                    map.serialize_key("underline")?;
                    map.serialize_value(&underline.to_string())?;
                }
                map.serialize_key("add_modifier")?;
                map.serialize_value(&value.add_modifier.bits())?;
                map.serialize_key("sub_modifier")?;
                map.serialize_value(&value.sub_modifier.bits())?;
                map.end()
            }
            SettingsValue::Key(value) => {
                let mut map = serializer.serialize_map(Some(4))?;
                map.serialize_key("code")?;
                map.serialize_value(&KeySettings::key_code_to_string(value.code))?;
                map.serialize_key("modifiers")?;
                map.serialize_value(&value.modifiers.bits())?;
                map.serialize_key("kind")?;
                map.serialize_value(&KeySettings::key_event_kind_to_string(value.kind))?;
                map.serialize_key("state")?;
                map.serialize_value(&value.state.bits())?;
                map.end()
            }
        }
    }
}

struct SettingsValueVisitor;

impl SettingsValueVisitor {
    fn visit_bool<E>(self, value: bool) -> Result<SettingsValue, E> {
        Ok(SettingsValue::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<SettingsValue, E> {
        Ok(SettingsValue::Int(value))
    }

    fn visit_f64<E>(self, value: f64) -> Result<SettingsValue, E> {
        Ok(SettingsValue::Float(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<SettingsValue, E> {
        Ok(SettingsValue::String(value.to_string()))
    }

    fn visit_style<E>(self, style: Style) -> Result<SettingsValue, E> {
        Ok(SettingsValue::Style(style))
    }

    fn visit_key<E>(self, value: KeyEvent) -> Result<SettingsValue, E> {
        Ok(SettingsValue::Key(value))
    }
}

impl<'de> serde::de::Visitor<'de> for SettingsValueVisitor {
    type Value = SettingsValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a boolean, integer, number, string, style, or key event")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        self.visit_bool(value)
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v.into())
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v.into())
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v.into())
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        self.visit_i64(value)
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v.into())
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v.into())
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v.into())
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Ok(value) = i64::try_from(v) {
            self.visit_i64(value)
        } else {
            Err(E::custom("u64 value is too large to fit in an i64"))
        }
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_f64(v.into())
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
        self.visit_f64(value)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        self.visit_str(value)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut style = Style::default();
        let mut key_event = KeyEvent::new(KeyCode::Null, KeyModifiers::NONE);
        #[derive(Debug, PartialEq)]
        enum MapKind {
            Unknown,
            Style,
            Key,
        }
        let mut map_kind = MapKind::Unknown;
        while let Some(key) = map.next_key()? {
            if map_kind == MapKind::Unknown {
                match key {
                    "fg" | "bg" | "underline" | "add_modifier" | "sub_modifier" => {
                        map_kind = MapKind::Style;
                    }
                    "code" | "modifiers" | "kind" | "state" => {
                        map_kind = MapKind::Key;
                    }
                    _ => break,
                }
            }

            match map_kind {
                MapKind::Unknown => break,
                MapKind::Style => match key {
                    "fg" => {
                        let value = map.next_value()?;
                        style.fg = Some(value);
                    }
                    "bg" => {
                        let value = map.next_value()?;
                        style.bg = Some(value);
                    }
                    "underline" => {
                        let value = map.next_value()?;
                        style.underline_color = Some(value);
                    }
                    "add_modifier" => {
                        let value = map.next_value()?;
                        style.add_modifier = Modifier::from_bits(value).ok_or_else(|| {
                            serde::de::Error::custom("Invalid style add modifier")
                        })?;
                    }
                    "sub_modifier" => {
                        let value = map.next_value()?;
                        style.sub_modifier = Modifier::from_bits(value).ok_or_else(|| {
                            serde::de::Error::custom("Invalid style sub modifier")
                        })?;
                    }
                    _ => {
                        map_kind = MapKind::Unknown;
                        break;
                    }
                },
                MapKind::Key => match key {
                    "code" => {
                        let value = map.next_value::<String>()?;
                        key_event.code = KeySettings::string_to_key_code(&value)
                            .map_err(serde::de::Error::custom)?;
                    }
                    "modifiers" => {
                        let value = map.next_value()?;
                        key_event.modifiers = KeyModifiers::from_bits(value)
                            .ok_or_else(|| serde::de::Error::custom("Invalid key modifiers"))?;
                    }
                    "kind" => {
                        let value = map.next_value::<String>()?;
                        key_event.kind = KeySettings::string_to_key_event_kind(&value)
                            .map_err(serde::de::Error::custom)?;
                    }
                    "state" => {
                        let value = map.next_value()?;
                        key_event.state = KeyEventState::from_bits(value)
                            .ok_or_else(|| serde::de::Error::custom("Invalid key event state"))?;
                    }
                    _ => {
                        map_kind = MapKind::Unknown;
                        break;
                    }
                },
            }
        }
        match map_kind {
            MapKind::Unknown => Err(serde::de::Error::custom(
                "Invalid table, expected style or key event",
            )),
            MapKind::Style => self.visit_style(style),
            MapKind::Key => self.visit_key(key_event),
        }
    }
}

impl<'de> Deserialize<'de> for SettingsValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(SettingsValueVisitor)
    }
}

impl IntoLua for SettingsValue {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        match self {
            SettingsValue::Bool(value) => value.into_lua(lua),
            SettingsValue::Int(value) => value.into_lua(lua),
            SettingsValue::Float(value) => value.into_lua(lua),
            SettingsValue::String(value) => value.into_lua(lua),
            SettingsValue::Style(value) => Ok(mlua::Value::Table(get_style(lua, &value)?)),
            SettingsValue::Key(value) => Ok(mlua::Value::Table(key_event_to_lua(lua, &value)?)),
        }
    }
}

impl FromLua for SettingsValue {
    fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::Boolean(value) => Ok(SettingsValue::Bool(value)),
            mlua::Value::Integer(value) => Ok(SettingsValue::Int(value)),
            mlua::Value::Number(value) => Ok(SettingsValue::Float(value)),
            mlua::Value::String(value) => Ok(SettingsValue::String(value.to_str()?.to_string())),
            mlua::Value::Table(value) => {
                if value.contains_key("fg")?
                    || value.contains_key("bg")?
                    || value.contains_key("underline")?
                    || value.contains_key("add_modifier")?
                    || value.contains_key("sub_modifier")?
                {
                    let mut style = Style::default();
                    set_style(_lua, &mut style, value)?;
                    Ok(SettingsValue::Style(style))
                } else if value.contains_key("code")?
                    || value.contains_key("modifiers")?
                    || value.contains_key("kind")?
                    || value.contains_key("state")?
                {
                    let key_event = lua_to_key_event(_lua, &value)?;
                    Ok(SettingsValue::Key(key_event))
                } else {
                    Err(mlua::Error::RuntimeError(
                        "Invalid table, expected style or key event".to_string(),
                    ))
                }
            }
            _ => Err(mlua::Error::RuntimeError(
                "Expected boolean, integer, number, string, style or key event".to_string(),
            )),
        }
    }
}
