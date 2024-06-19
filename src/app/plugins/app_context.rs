use mlua::UserData;

use crate::app::log::logger::Logger;

#[derive(Debug, Clone)]
pub struct AppContext
{
    pub logger: Logger,
}

impl AppContext
{
    pub fn new() -> Self
    {
        Self::default()
    }
}

impl Default for AppContext
{
    fn default() -> Self
    {
        Self
        {
            logger: Logger::new(),
        }
    }
}

impl UserData for AppContext 
{
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M)
    {
        methods.add_method_mut("log", 
            |_lua, this, (level, message): (u8, String)| 
            {
                this.logger.log(level.into(), &message);
                Ok(())
            }
        );
    }
}