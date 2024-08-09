use std::time::Instant;

use mlua::UserData;

#[derive(Clone, Copy, Debug)]
pub struct PluginInstant {
    pub inner: Instant
}

impl PluginInstant {
    pub fn now() -> Self {
        Self { inner: Instant::now() }
    }
}

impl UserData for PluginInstant {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("elapsed", |_, this, ()| {
            Ok(this.inner.elapsed().as_secs_f64())
        });
    }
}
