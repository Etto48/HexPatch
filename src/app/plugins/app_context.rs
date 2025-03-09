use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use mlua::{Function, Lua, Scope, Table};

use crate::{
    app::{
        data::Data,
        log::{logger::Logger, NotificationLevel},
        pane::Pane,
        popup::popup_state::PopupState,
        settings::Settings,
        App,
    },
    headers::Header,
};

use super::{
    exported_commands::ExportedCommands, exported_header_parsers::ExportedHeaderParsers,
    instruction_info::InstructionInfo, plugin_instant::PluginInstant,
};

#[macro_export]
macro_rules! get_app_context {
    ($app:ident) => {
        $crate::app::plugins::app_context::AppContext::new(
            $app.get_cursor_position().global_byte_index,
            $app.get_current_instruction().map(|i| i.into()),
            $app.screen_size.1,
            $app.screen_size.0,
            $app.blocks_per_row,
            $app.block_size,
            $app.vertical_margin,
            &mut $app.scroll,
            &mut $app.cursor,
            &mut $app.data,
            &$app.header,
            &mut $app.settings,
            &mut $app.logger,
            &mut $app.popup,
            &mut $app.fullscreen,
            &mut $app.selected_pane,
        )
    };
}

pub struct AppContext<'app> {
    pub exported_commands: Arc<Mutex<ExportedCommands>>,
    pub exported_header_parsers: Arc<Mutex<ExportedHeaderParsers>>,
    pub plugin_index: Option<usize>,

    pub screen_height: u16,
    pub screen_width: u16,
    pub blocks_per_row: usize,
    pub block_size: usize,
    pub vertical_margin: u16,
    pub data: Arc<Mutex<&'app mut Data>>,
    pub scroll: &'app mut usize,
    pub cursor: &'app mut (u16, u16),
    pub offset: usize,
    pub current_instruction: Option<InstructionInfo>,
    pub header: &'app Header,
    pub settings: &'app mut Settings,
    pub logger: &'app mut Logger,
    pub popup: Arc<Mutex<&'app mut Option<PopupState>>>,
    pub fullscreen: Arc<Mutex<&'app mut bool>>,
    pub selected_pane: Arc<Mutex<&'app mut Pane>>,
}

impl<'app> AppContext<'app> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        offset: usize,
        current_instruction: Option<InstructionInfo>,
        screen_height: u16,
        screen_width: u16,
        blocks_per_row: usize,
        block_size: usize,
        vertical_margin: u16,
        scroll: &'app mut usize,
        cursor: &'app mut (u16, u16),
        data: &'app mut Data,
        header: &'app Header,
        settings: &'app mut Settings,
        logger: &'app mut Logger,
        popup: &'app mut Option<PopupState>,
        fullscreen: &'app mut bool,
        selected_pane: &'app mut Pane,
    ) -> Self {
        Self {
            exported_commands: Arc::new(Mutex::new(ExportedCommands::default())),
            exported_header_parsers: Arc::new(Mutex::new(ExportedHeaderParsers::default())),
            plugin_index: None,
            screen_height,
            screen_width,
            blocks_per_row,
            block_size,
            vertical_margin,
            data: Arc::new(Mutex::new(data)),
            scroll,
            cursor,
            offset,
            current_instruction,
            header,
            settings,
            logger,
            popup: Arc::new(Mutex::new(popup)),
            fullscreen: Arc::new(Mutex::new(fullscreen)),
            selected_pane: Arc::new(Mutex::new(selected_pane)),
        }
    }

    pub fn reset_exported_commands(&mut self) {
        self.exported_commands = Arc::new(Mutex::new(ExportedCommands::default()));
    }

    pub fn reset_exported_header_parsers(&mut self) {
        self.exported_header_parsers = Arc::new(Mutex::new(ExportedHeaderParsers::default()));
    }

    pub fn set_exported_commands(&mut self, exported_commands: ExportedCommands) {
        self.exported_commands = Arc::new(Mutex::new(exported_commands));
    }

    pub fn set_exported_header_parsers(&mut self, exported_header_parsers: ExportedHeaderParsers) {
        self.exported_header_parsers = Arc::new(Mutex::new(exported_header_parsers));
    }

    pub fn take_exported_commands(&mut self) -> ExportedCommands {
        self.exported_commands.lock().unwrap().take()
    }

    pub fn take_exported_header_parsers(&mut self) -> ExportedHeaderParsers {
        self.exported_header_parsers.lock().unwrap().take()
    }

    pub fn to_lua<'scope, 'env>(
        &'env mut self,
        lua: &'scope Lua,
        scope: &'scope Scope<'scope, 'env>,
    ) -> mlua::Table {
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
                        if let Ok(_command_fn) = lua.globals().get::<Function>(command.clone()) {
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

        let exported_header_parsers = self.exported_header_parsers.clone();
        context
            .set(
                "add_header_parser",
                scope
                    .create_function_mut(move |lua, callback: String| {
                        if let Ok(_header_parser_fn) =
                            lua.globals().get::<Function>(callback.clone())
                        {
                            exported_header_parsers
                                .lock()
                                .unwrap()
                                .add_header_parser(callback);
                            Ok(())
                        } else {
                            Err(mlua::Error::external(format!(
                                "Function '{}' not found but needed to export the header parser",
                                callback
                            )))
                        }
                    })
                    .unwrap(),
            )
            .unwrap();

        let exported_header_parsers = self.exported_header_parsers.clone();
        context
            .set(
                "remove_header_parser",
                scope
                    .create_function_mut(move |_, callback: String| {
                        if exported_header_parsers
                            .lock()
                            .unwrap()
                            .remove_header_parser(&callback)
                        {
                            Ok(())
                        } else {
                            Err(mlua::Error::external(format!(
                                "Header parser '{}' not found",
                                callback
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
                        } else if lua.globals().get::<Function>(callback.clone()).is_err() {
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
        let data = lua.create_table().unwrap();
        data.set("len", self.data.lock().unwrap().len()).unwrap();
        data.set(
            "get",
            scope
                .create_function_mut(|_, (_this, index): (Table, usize)| {
                    let data = self.data.lock().unwrap();
                    match data.get(index) {
                        Some(byte) => Ok(byte),
                        None => Err(mlua::Error::external("Index out of bounds")),
                    }
                })
                .unwrap(),
        )
        .unwrap();
        data.set(
            "set",
            scope
                .create_function_mut(|_, (_this, index, byte): (Table, usize, u8)| {
                    let mut data = self.data.lock().unwrap();
                    data.set(index, byte)
                })
                .unwrap(),
        )
        .unwrap();
        context.set("data", data).unwrap();
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
            .set(
                "get_instant_now",
                scope
                    .create_function(|_, ()| Ok(PluginInstant::now()))
                    .unwrap(),
            )
            .unwrap();
        context
            .set(
                "jump_to",
                scope
                    .create_function_mut(|_, file_address: usize| {
                        App::jump_to_no_self(
                            file_address,
                            self.data.lock().unwrap().deref(),
                            (self.screen_width, self.screen_height),
                            self.vertical_margin,
                            self.scroll,
                            self.cursor,
                            self.block_size,
                            self.blocks_per_row,
                        );
                        Ok(())
                    })
                    .unwrap(),
            )
            .unwrap();
        context
            .set(
                "get_fullscreen",
                scope
                    .create_function(|_, ()| {
                        let fullscreen = self.fullscreen.lock().unwrap();
                        Ok(fullscreen.clone())
                    })
                    .unwrap(),
            )
            .unwrap();
        context
            .set(
                "set_fullscreen",
                scope
                    .create_function_mut(|_, fullscreen: bool| {
                        **self.fullscreen.lock().unwrap() = fullscreen;
                        Ok(())
                    })
                    .unwrap(),
            )
            .unwrap();
        context
            .set(
                "get_selected_pane",
                scope
                    .create_function(|_, ()| {
                        let selected_pane = self.selected_pane.lock().unwrap();
                        Ok(selected_pane.clone())
                    })
                    .unwrap(),
            )
            .unwrap();
        context
            .set(
                "set_selected_pane",
                scope
                    .create_function_mut(|_, selected_pane: Pane| {
                        **self.selected_pane.lock().unwrap() = selected_pane;
                        Ok(())
                    })
                    .unwrap(),
            )
            .unwrap();

        context
    }
}
