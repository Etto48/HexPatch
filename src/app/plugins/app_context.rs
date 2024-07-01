use std::sync::{Arc, Mutex};

use mlua::{Function, Lua, Scope};

use crate::{
    app::{
        data::Data,
        log::{logger::Logger, NotificationLevel},
        popup::popup_state::PopupState,
        settings::Settings,
    },
    headers::Header,
};

use super::{exported_commands::ExportedCommands, instruction_info::InstructionInfo};

#[macro_export]
macro_rules! get_app_context {
    ($app:ident) => {
        $crate::app::plugins::app_context::AppContext::new(
            $app.get_cursor_position().global_byte_index,
            $app.get_current_instruction().map(|i| i.into()),
            $app.screen_size.1,
            $app.screen_size.0,
            &mut $app.data,
            &$app.header,
            &mut $app.settings,
            &mut $app.logger,
            &mut $app.popup,
        )
    };
}

pub struct AppContext<'app> {
    pub exported_commands: Arc<Mutex<ExportedCommands>>,
    pub plugin_index: Option<usize>,

    pub screen_height: u16,
    pub screen_width: u16,
    pub data: &'app mut Data,
    pub offset: usize,
    pub current_instruction: Option<InstructionInfo>,
    pub header: &'app Header,
    pub settings: &'app mut Settings,
    pub logger: &'app mut Logger,
    pub popup: Arc<Mutex<&'app mut Option<PopupState>>>,
}

impl<'app> AppContext<'app> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        offset: usize,
        current_instruction: Option<InstructionInfo>,
        screen_height: u16,
        screen_width: u16,
        data: &'app mut Data,
        header: &'app Header,
        settings: &'app mut Settings,
        logger: &'app mut Logger,
        popup: &'app mut Option<PopupState>,
    ) -> Self {
        Self {
            exported_commands: Arc::new(Mutex::new(ExportedCommands::default())),
            plugin_index: None,
            screen_height,
            screen_width,
            data,
            offset,
            current_instruction,
            header,
            settings,
            logger,
            popup: Arc::new(Mutex::new(popup)),
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

    pub fn to_lua<'lua>(
        &'lua mut self,
        lua: &'lua Lua,
        scope: &Scope<'lua, '_>,
    ) -> mlua::Table<'lua> {
        let context = lua.create_table().unwrap();
        context
            .set(
                "log",
                scope
                    .create_function_mut(|_, (level, message): (u8, String)| {
                        self.logger.log(NotificationLevel::from(level), &message);
                        Ok(())
                    })
                    .unwrap(),
            )
            .unwrap();

        let exported_commands = self.exported_commands.clone();
        context
            .set(
                "add_command",
                scope
                    .create_function_mut(move |lua, (command, description): (String, String)| {
                        if let Ok(_command_fn) = lua.globals().get::<_, Function>(command.clone()) {
                            exported_commands
                                .lock()
                                .unwrap()
                                .add_command(command, description);
                            Ok(())
                        } else {
                            Err(mlua::Error::external(format!(
                                "Function '{}' not found but needed to export the command",
                                command
                            )))
                        }
                    })
                    .unwrap(),
            )
            .unwrap();

        let exported_commands = self.exported_commands.clone();
        context
            .set(
                "remove_command",
                scope
                    .create_function_mut(move |_, command: String| {
                        if exported_commands.lock().unwrap().remove_command(&command) {
                            Ok(())
                        } else {
                            Err(mlua::Error::external(format!(
                                "Command '{}' not found",
                                command
                            )))
                        }
                    })
                    .unwrap(),
            )
            .unwrap();

        context
            .set(
                "open_popup",
                scope
                    .create_function_mut(|_, callback: String| {
                        let mut popup = self.popup.lock().unwrap();
                        if popup.is_some() {
                            Err(mlua::Error::external("Popup already open"))
                        } else if lua.globals().get::<_, Function>(callback.clone()).is_err() {
                            Err(mlua::Error::external(format!(
                                "Function '{}' not found but needed to open the popup",
                                callback
                            )))
                        } else {
                            **popup = Some(PopupState::Custom {
                                plugin_index: self.plugin_index.unwrap(),
                                callback,
                            });
                            Ok(())
                        }
                    })
                    .unwrap(),
            )
            .unwrap();

        context
            .set(
                "get_popup",
                scope
                    .create_function(|_, ()| {
                        let popup = self.popup.lock().unwrap();
                        if let Some(PopupState::Custom {
                            plugin_index,
                            callback,
                        }) = *popup as &Option<PopupState>
                        {
                            if self.plugin_index.unwrap() != *plugin_index {
                                Ok(mlua::Value::Nil)
                            } else {
                                Ok(mlua::Value::String(
                                    lua.create_string(callback.as_str()).unwrap(),
                                ))
                            }
                        } else {
                            Ok(mlua::Value::Nil)
                        }
                    })
                    .unwrap(),
            )
            .unwrap();

        context
            .set(
                "close_popup",
                scope
                    .create_function_mut(|_, expected_callback: Option<String>| {
                        let mut popup = self.popup.lock().unwrap();
                        if let Some(PopupState::Custom {
                            plugin_index,
                            callback,
                        }) = *popup as &mut Option<PopupState>
                        {
                            if expected_callback.is_some()
                                && expected_callback.as_ref() != Some(callback)
                            {
                                Err(mlua::Error::external(
                                    "A popup is open but not the one expected.",
                                ))
                            } else if self.plugin_index.unwrap() != *plugin_index {
                                Err(mlua::Error::external(
                                    "A popup is open but not from this plugin.",
                                ))
                            } else {
                                **popup = None;
                                Ok(())
                            }
                        } else {
                            Err(mlua::Error::external("No plugin related popup is open."))
                        }
                    })
                    .unwrap(),
            )
            .unwrap();

        context.set("screen_height", self.screen_height).unwrap();
        context.set("screen_width", self.screen_width).unwrap();
        context
            .set("data", scope.create_userdata_ref_mut(self.data).unwrap())
            .unwrap();
        context.set("offset", self.offset).unwrap();
        context
            .set("current_instruction", self.current_instruction.clone())
            .unwrap();
        context
            .set("header", scope.create_userdata_ref(self.header).unwrap())
            .unwrap();
        context
            .set(
                "settings",
                scope.create_any_userdata_ref_mut(self.settings).unwrap(),
            )
            .unwrap();

        context
    }
}
