use mlua::UserData;

#[derive(Debug, Clone, Default)]
pub struct Data {
    pub bytes: Vec<u8>,
    pub dirty: bool,
}

impl Data {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            dirty: false,
        }
    }
}

impl UserData for Data {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("len", |_, this| Ok(this.bytes.len() as i64));
        fields.add_field_method_get("is_dirty", |_, this| Ok(this.dirty));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get", |_, this, i: usize| match this.bytes.get(i) {
            Some(&byte) => Ok(byte),
            None => Err(mlua::Error::external("index out of bounds")),
        });

        methods.add_method_mut("set", |_, this, (i, byte): (usize, u8)| {
            match this.bytes.get_mut(i) {
                Some(b) => {
                    *b = byte;
                    this.dirty = true;
                    Ok(())
                }
                None => Err(mlua::Error::external("index out of bounds")),
            }
        });
    }
}
