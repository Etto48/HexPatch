use mlua::{Function, UserData};

use crate::app::log::logger::Logger;

use super::exported_commands::ExportedCommands;

#[derive(Debug, Clone, Default)]
pub struct AppContext
{
    pub logger: Logger,
    pub exported_commands: ExportedCommands,
}

impl AppContext
{
    /// Same as `AppContext::default()`.
    pub fn new() -> Self
    {
        Self::default()
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

        methods.add_method_mut("add_command", 
            |lua, this, (command, description): (String, String)| 
            {
                if let Ok(_command_fn) = lua.globals().get::<_,Function>(command.clone())
                {
                    this.exported_commands.add_command(command, description);
                    Ok(())
                }
                else
                {
                    Err(mlua::Error::external(format!("Function '{}' not found but needed to export the command", command)))
                }
            }
        );

        methods.add_method_mut("remove_command", 
            |_lua, this, command: String|
            {
                if this.exported_commands.remove_command(&command)
                {
                    Ok(())
                }
                else
                {
                    Err(mlua::Error::external(format!("Command '{}' not found", command)))
                }
            }
        );
    }
}