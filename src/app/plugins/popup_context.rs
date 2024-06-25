use mlua::{Lua, Scope, Table};
use ratatui::text::Text;

pub struct PopupContext<'app> 
{
    pub text: &'app mut Text<'static>,
    pub title: &'app mut String,
    pub height: &'app mut usize,
    pub width: &'app mut usize
}

impl<'app> PopupContext<'app> 
{
    pub fn new(
        text: &'app mut Text<'static>, 
        title: &'app mut String,
        height: &'app mut usize,
        width: &'app mut usize) -> Self
    {
        Self { 
            text, 
            title,
            height,
            width
        }
    }

    pub fn to_lua<'lua>(
        &'lua mut self, 
        lua: &'lua Lua, 
        scope: &Scope<'lua, '_>) -> Table<'lua>
    {
        let table = lua.create_table().unwrap();
        table.set("text", scope.create_any_userdata_ref_mut(self.text).unwrap()).unwrap();
        table.set("title", scope.create_any_userdata_ref_mut(self.title).unwrap()).unwrap();
        table.set("height", scope.create_any_userdata_ref_mut(self.height).unwrap()).unwrap();
        table.set("width", scope.create_any_userdata_ref_mut(self.width).unwrap()).unwrap();
        table
    }
}