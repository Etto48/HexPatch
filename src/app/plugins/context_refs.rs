use std::sync::{Arc, Mutex};

use mlua::{Function, Lua, Scope};

use crate::{app::{log::{logger::Logger, NotificationLevel}, popup_state::PopupState, settings::Settings}, headers::Header};

use super::{exported_commands::ExportedCommands, instruction_info::InstructionInfo};

#[macro_export]
macro_rules! get_context_refs {
    ($app:ident) => {
        $crate::app::plugins::context_refs::ContextRefs::new(
            $app.get_cursor_position().global_byte_index, 
            $app.get_current_instruction()
                .map(|i| i.into()), 
            &mut $app.data, 
            &$app.header, 
            &mut $app.settings, 
            &mut $app.logger, 
            &mut $app.popup
        )
    };
}

pub struct ContextRefs<'app> {
    pub exported_commands: Arc<Mutex<ExportedCommands>>,
    pub plugin_index: Option<usize>,

    pub data: &'app mut Vec<u8>,
    pub offset: usize,
    pub current_instruction: Option<InstructionInfo>,
    pub header: &'app Header,
    pub settings: &'app mut Settings,
    pub logger: &'app mut Logger,
    pub popup: &'app mut Option<PopupState>,
}

impl<'app> ContextRefs<'app> {
    pub fn new(
        offset: usize,
        current_instruction: Option<InstructionInfo>,
        data: &'app mut Vec<u8>,
        header: &'app Header, 
        settings: &'app mut Settings, 
        logger: &'app mut Logger, 
        popup: &'app mut Option<PopupState>) -> Self {
        Self {
            exported_commands: Arc::new(Mutex::new(ExportedCommands::default())),
            plugin_index: None,
            data,
            offset,
            current_instruction,
            header,
            settings,
            logger,
            popup,
        }
    }

    pub fn reset_exported_commands(&mut self) {
        self.exported_commands = Arc::new(Mutex::new(ExportedCommands::default()));
    }

    pub fn set_exported_commands(&mut self, exported_commands: ExportedCommands) {
        self.exported_commands = Arc::new(Mutex::new(exported_commands));
    }

    pub fn take_exported_commands(&mut self) -> ExportedCommands {
        self.exported_commands.lock().unwrap().take()
    }
}

pub fn create_lua_context<'lua>(
    lua: &'lua Lua, 
    scope: &Scope<'lua, '_>, 
    context_refs: &'lua mut ContextRefs) -> mlua::Table<'lua> 
{
    let context = lua.create_table().unwrap();
    context.set("log", scope.create_function_mut(|_, (level, message): (u8, String)| {
        context_refs.logger.log(NotificationLevel::from(level), &message);
        Ok(())
    }).unwrap()).unwrap();

    let exported_commands = context_refs.exported_commands.clone();
    context.set("add_command", 
        scope.create_function_mut(move |lua, (command, description): (String, String)| 
        {
            if let Ok(_command_fn) = lua.globals().get::<_,Function>(command.clone())
            {
                exported_commands.lock().unwrap().add_command(command, description);
                Ok(())
            }
            else
            {
                Err(mlua::Error::external(format!("Function '{}' not found but needed to export the command", command)))
            }
        }).unwrap()
    ).unwrap();

    let exported_commands = context_refs.exported_commands.clone();
    context.set("remove_command", 
        scope.create_function_mut(move |_, command: String|
        {
            if exported_commands.lock().unwrap().remove_command(&command)
            {
                Ok(())
            }
            else
            {
                Err(mlua::Error::external(format!("Command '{}' not found", command)))
            }
        }).unwrap()
    ).unwrap();

    context.set("open_popup", scope.create_function_mut(
        |_, callback: String| 
        {
            if context_refs.popup.is_some()
            {
                Err(mlua::Error::external("Popup already open"))
            }
            else if lua.globals().get::<_,Function>(callback.clone()).is_err()
            {
                Err(mlua::Error::external(format!("Function '{}' not found but needed to open the popup", callback)))
            }
            else
            {
                *context_refs.popup = Some(PopupState::Custom { 
                    plugin_index: context_refs.plugin_index.unwrap(), 
                    callback 
                });
                Ok(())
            }
        }).unwrap()
    ).unwrap();

    context.set("data", scope.create_any_userdata_ref_mut(context_refs.data).unwrap()).unwrap();
    context.set("offset", context_refs.offset).unwrap();
    context.set("current_instruction", context_refs.current_instruction.clone()).unwrap();
    context.set("header", scope.create_userdata_ref(context_refs.header).unwrap()).unwrap();
    context.set("settings", scope.create_any_userdata_ref_mut(context_refs.settings).unwrap()).unwrap();

    context
}