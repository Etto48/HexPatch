use std::fmt::{Display, Formatter};

use mlua::{FromLua, IntoLua};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsValue
{
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

impl From<bool> for SettingsValue
{
    fn from(value: bool) -> Self
    {
        SettingsValue::Bool(value)
    }
}

impl From<i64> for SettingsValue
{
    fn from(value: i64) -> Self
    {
        SettingsValue::Int(value)
    }
}

impl From<f64> for SettingsValue
{
    fn from(value: f64) -> Self
    {
        SettingsValue::Float(value)
    }
}

impl From<String> for SettingsValue
{
    fn from(value: String) -> Self
    {
        SettingsValue::String(value)
    }
}

impl From<&str> for SettingsValue
{
    fn from(value: &str) -> Self
    {
        SettingsValue::String(value.to_string())
    }
}

impl Display for SettingsValue
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            SettingsValue::Bool(value) => write!(f, "{}", value),
            SettingsValue::Int(value) => write!(f, "{}", value),
            SettingsValue::Float(value) => write!(f, "{}", value),
            SettingsValue::String(value) => write!(f, "{}", value),
        }
    }
}

impl Serialize for SettingsValue
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer
    {
        match self
        {
            SettingsValue::Bool(value) => serializer.serialize_bool(*value),
            SettingsValue::Int(value) => serializer.serialize_i64(*value),
            SettingsValue::Float(value) => serializer.serialize_f64(*value),
            SettingsValue::String(value) => serializer.serialize_str(value),
        }
    }
}

struct SettingsValueVisitor;

impl SettingsValueVisitor
{
    fn visit_bool<E>(self, value: bool) -> Result<SettingsValue, E>
    {
        Ok(SettingsValue::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<SettingsValue, E>
    {
        Ok(SettingsValue::Int(value))
    }

    fn visit_f64<E>(self, value: f64) -> Result<SettingsValue, E>
    {
        Ok(SettingsValue::Float(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<SettingsValue, E>
    {
        Ok(SettingsValue::String(value.to_string()))
    }
}

impl <'de> serde::de::Visitor<'de> for SettingsValueVisitor
{
    type Value = SettingsValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a boolean, integer, number, or string")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    {
        self.visit_bool(value)
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        self.visit_i64(v.into())
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        self.visit_i64(v.into())
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        self.visit_i64(v.into())
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    {
        self.visit_i64(value)
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        self.visit_i64(v.into())
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        self.visit_i64(v.into())
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        self.visit_i64(v.into())
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        if let Some(value) = i64::try_from(v).ok()
        {
            self.visit_i64(value)
        } else {
            Err(E::custom("u64 value is too large to fit in an i64"))
        }
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        self.visit_f64(v.into())
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    {
        self.visit_f64(value)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    {
        self.visit_str(value)
    }
}

impl<'de> Deserialize<'de> for SettingsValue
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        deserializer.deserialize_any(SettingsValueVisitor)
    }
}

impl<'lua> IntoLua<'lua> for SettingsValue
{
    fn into_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
        match self
        {
            SettingsValue::Bool(value) => value.into_lua(lua),
            SettingsValue::Int(value) => value.into_lua(lua),
            SettingsValue::Float(value) => value.into_lua(lua),
            SettingsValue::String(value) => value.into_lua(lua),
        }
    }
}

impl<'lua> FromLua<'lua> for SettingsValue
{
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value
        {
            mlua::Value::Boolean(value) => Ok(SettingsValue::Bool(value)),
            mlua::Value::Integer(value) => Ok(SettingsValue::Int(value)),
            mlua::Value::Number(value) => Ok(SettingsValue::Float(value)),
            mlua::Value::String(value) => Ok(SettingsValue::String(value.to_str()?.to_string())),
            _ => Err(mlua::Error::RuntimeError("Expected boolean, integer, number, or string".to_string())),
        }
    }
}