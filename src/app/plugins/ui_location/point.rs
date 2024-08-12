use mlua::IntoLua;
use ratatui::layout::Rect;

use super::rect_borders::RectBorders;

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    /// Returns Some if this point is inside the Rect. Returns None if it's outside
    pub fn get_relative_location(&self, rect: &Rect) -> Option<(Point, RectBorders)> {
        if self.x >= rect.x
            && self.x < rect.x + rect.width
            && self.y >= rect.y
            && self.y < rect.y + rect.height
        {
            Some((
                Point::new(self.x - rect.x, self.y - rect.y),
                RectBorders {
                    top: self.y == rect.y,
                    bottom: self.y == rect.y + rect.height.saturating_sub(1),
                    left: self.x == rect.x,
                    right: self.x == rect.x + rect.width.saturating_sub(1),
                },
            ))
        } else {
            None
        }
    }
}

impl<'lua> IntoLua<'lua> for Point {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let ret = lua.create_table()?;
        ret.set("x", self.x)?;
        ret.set("y", self.y)?;
        Ok(mlua::Value::Table(ret))
    }
}
