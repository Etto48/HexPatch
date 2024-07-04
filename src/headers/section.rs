use std::fmt::Display;

use mlua::UserData;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Section {
    pub name: String,
    pub virtual_address: u64,
    pub file_offset: u64,
    pub size: u64,
}

impl Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}: [{:X} - {:X})",
            self.name,
            self.file_offset,
            self.file_offset + self.size
        )
    }
}

impl UserData for Section {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_, this| Ok(this.name.clone()));
        fields.add_field_method_get("virtual_address", |_, this| Ok(this.virtual_address));
        fields.add_field_method_get("file_offset", |_, this| Ok(this.file_offset));
        fields.add_field_method_get("size", |_, this| Ok(this.size));
    }
}
