use mlua::{FromLua, IntoLua};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pane {
    Hex,
    View,
}

impl IntoLua for Pane {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        Ok(mlua::Value::String(match self {
            Pane::Hex => lua.create_string("hex").unwrap(),
            Pane::View => lua.create_string("view").unwrap(),
        }))
    }
}

impl FromLua for Pane {
    fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        let value = value.to_string()?;
        match value.as_str() {
            "hex" => Ok(Pane::Hex),
            "view" => Ok(Pane::View),
            _ => Err(mlua::Error::external("Invalid Pane")),
        }
    }
}
