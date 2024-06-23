use std::error::Error;

use mlua::{Function, Lua};

use crate::app::{commands::command_info::CommandInfo, log::NotificationLevel, settings::{register_key_settings_macro::key_event_to_lua, Settings}};

use super::{ app_context::AppContext, event::{Event, Events}, exported_commands::ExportedCommands, register_userdata::{register_logger, register_settings, register_vec_u8}};

#[derive(Debug)]
pub struct Plugin
{
    lua: Lua,
    commands: ExportedCommands,
}

impl Plugin {
    pub fn new_from_source(source: &str, settings: &mut Settings, context: &mut AppContext) -> Result<Self, Box<dyn Error>>
    {
        let lua = Lua::new();
        lua.load(source).exec()?;

        register_vec_u8(&lua)?;
        register_settings(&lua)?;
        register_logger(&lua)?;
        context.exported_commands = ExportedCommands::default();
        if let Ok(init) = lua.globals().get::<_, Function>("init")
        {
            lua.scope(|scope|{
                let settings = scope.create_any_userdata_ref_mut(settings)?;
                let context = scope.create_userdata_ref_mut(context)?;
                init.call((settings, context))
            })?;
        }
        
        Ok(Plugin { lua , commands: context.exported_commands.take() })
    }

    pub fn new_from_file(path: &str, settings: &mut Settings, context: &mut AppContext) -> Result<Self, Box<dyn Error>>
    {
        let source = std::fs::read_to_string(path)?;
        Self::new_from_source(&source, settings, context)
    }

    pub fn get_event_handlers(&self) -> Events
    {
        let mut handlers = Events::NONE;
        if self.lua.globals().get::<_, Function>("on_open").is_ok()
        {
            handlers |= Events::ON_OPEN;
        }
        if self.lua.globals().get::<_, Function>("on_edit").is_ok()
        {
            handlers |= Events::ON_EDIT;
        }
        if self.lua.globals().get::<_, Function>("on_save").is_ok()
        {
            handlers |= Events::ON_SAVE;
        }
        if self.lua.globals().get::<_, Function>("on_key").is_ok()
        {
            handlers |= Events::ON_KEY;
        }
        if self.lua.globals().get::<_, Function>("on_mouse").is_ok()
        {
            handlers |= Events::ON_MOUSE;
        }
        handlers
    }

    /// Handle an event, if an error occurs, return the error
    /// see [Plugin::handle] for a version that logs the error
    pub fn handle_with_error(&mut self, event: Event, context: &mut AppContext) -> mlua::Result<()>
    {
        context.exported_commands = self.commands.take();
        let ret = match event
        {
            Event::Open { data} =>
            {
                // Call the on_open function
                let on_open = self.lua.globals().get::<_, Function>("on_open").unwrap();
                self.lua.scope(|scope| {
                    let data = scope.create_any_userdata_ref_mut(data)?;
                    let context = scope.create_userdata_ref_mut(context)?;
                    on_open.call::<_,()>((data, context))
                })
            },
            Event::Edit { data, offset: starting_byte, new_bytes} =>
            {
                // Call the on_edit function
                let on_edit = self.lua.globals().get::<_, Function>("on_edit").unwrap();
                self.lua.scope(|scope| {
                    let data = scope.create_any_userdata_ref_mut(data)?;
                    let new_bytes = scope.create_any_userdata_ref_mut(new_bytes)?;
                    let context = scope.create_userdata_ref_mut(context)?;
                    on_edit.call::<_,()>((data, starting_byte, new_bytes, context))
                })
            },
            Event::Save { data} =>
            {
                // Call the on_save function
                let on_save = self.lua.globals().get::<_, Function>("on_save").unwrap();
                self.lua.scope(|scope| {
                    let data = scope.create_any_userdata_ref_mut(data)?;
                    let context = scope.create_userdata_ref_mut(context)?;
                    on_save.call::<_,()>((data, context))
                })
            },
            Event::Key {event, data, current_byte} =>
            {
                // Call the on_key function
                let on_key = self.lua.globals().get::<_, Function>("on_key").unwrap();
                let event = key_event_to_lua(&self.lua, &event).unwrap();
                self.lua.scope(|scope| {
                    let data = scope.create_any_userdata_ref_mut(data)?;
                    let context = scope.create_userdata_ref_mut(context)?;
                    on_key.call::<_,()>((event, data, current_byte, context))
                })
            },
            Event::Mouse {kind, row, col} =>
            {
                // Call the on_mouse function
                let on_mouse = self.lua.globals().get::<_, Function>("on_mouse").unwrap();
                self.lua.scope(|scope| {
                    let context = scope.create_userdata_ref_mut(context)?;
                    on_mouse.call::<_,()>((kind, row, col, context))
                })
            },
        };
        self.commands = context.exported_commands.take();
        ret
    }

    pub fn handle(&mut self, event: Event, context: &mut AppContext)
    {
        if let Err(e) = self.handle_with_error(event, context)
        {
            context.logger.log(NotificationLevel::Error, &format!("In plugin: {}", e));
        }
    }

    pub fn run_command(&mut self, context: &mut AppContext, command: &str) -> mlua::Result<()>
    {
        let command_fn = self.lua.globals().get::<_, Function>(command)?;
        context.exported_commands = self.commands.take();
        let ret = self.lua.scope(|scope| {
            let context = scope.create_userdata_ref_mut(context)?;
            command_fn.call::<_,()>(context)
        });
        self.commands = context.exported_commands.take();
        ret
    }

    pub fn get_commands(&self) -> &[CommandInfo]
    {
        self.commands.get_commands()
    }
}

#[cfg(test)]
mod test
{
    use crossterm::event::{KeyCode, KeyEvent};
    use ratatui::style::Style;

    use crate::app::{log::NotificationLevel, settings::settings_value::SettingsValue};

    use super::*;

    #[test]
    fn test_init_plugin()
    {
        let test_value = 42;
        let source = format!("
            test_value = 0
            function init(settings, context)
                test_value = {test_value}
            end
        ");
        let mut settings = Settings::default();
        let mut context = AppContext::default();
        let plugin = Plugin::new_from_source(&source, &mut settings, &mut context).unwrap();
        assert_eq!(plugin.lua.globals().get::<_, i32>("test_value").unwrap(), test_value);
    }

    #[test]
    fn test_discover_event_handlers()
    {
        let source = "
            function on_open(data, context) end
            function on_edit(data, selected_byte, new_bytes, context) end
            function on_save(data, context) end
            function on_key(key_event, data, current_byte, context) end
            function on_mouse(kind, row, col, context) end
        ";
        let mut settings = Settings::default();
        let mut context = AppContext::default();
        let plugin = Plugin::new_from_source(source, &mut settings, &mut context).unwrap();
        let handlers = plugin.get_event_handlers();
        assert_eq!(handlers, Events::ON_OPEN | Events::ON_EDIT | Events::ON_SAVE | Events::ON_KEY | Events::ON_MOUSE);
        let source = "
            function on_open(data, context) end
            function on_edit(data, selected_byte, new_bytes, context) end
            function on_save(data, context) end
        ";
        let plugin = Plugin::new_from_source(source, &mut settings, &mut context).unwrap();
        let handlers = plugin.get_event_handlers();
        assert_eq!(handlers, Events::ON_OPEN | Events::ON_EDIT | Events::ON_SAVE);
    }

    #[test]
    fn test_edit_open_data()
    {
        let source = "
            function on_open(data, context)
                data:set(0,42)
            end
        ";
        let mut settings = Settings::default();
        let mut context = AppContext::default();
        let mut data = vec![0; 0x100];
        let mut plugin = Plugin::new_from_source(source, &mut settings, &mut context).unwrap();
        let mut context = AppContext::default();
        let event = Event::Open { data: &mut data };
        plugin.handle_with_error(event, &mut context).unwrap();
        assert_eq!(data[0], 42);
    }

    #[test]
    fn test_init_change_settings()
    {
        let source = "
            function init(settings, context)
                settings.color_address_selected = {fg=\"#ff0000\",bg=\"Black\"}
                settings.color_address_default = {fg=2}

                settings.key_up = {code=\"Down\",modifiers=0,kind=\"Press\",state=0}

                if settings:get_custom(\"test\") ~= \"Hello\" then
                    error(\"Custom setting not set\")
                end
                settings:set_custom(\"string\", \"World\")
                settings:set_custom(\"integer\", 42)
                settings:set_custom(\"float\", 3.14)
                settings:set_custom(\"boolean\", true)
                settings:set_custom(\"nil\", nil)
                settings:set_custom(\"style\", {fg=\"#ff0000\",bg=\"#000000\"})
                settings:set_custom(\"key\", {code=\"Up\"})
            end
        ";
        let mut settings = Settings::default();
        let mut context = AppContext::default();
        settings.custom.insert("test".to_string(), SettingsValue::from("Hello"));
        let _ = Plugin::new_from_source(source, &mut settings, &mut context).unwrap();
        assert_eq!(settings.color.address_selected.fg, Some(ratatui::style::Color::Rgb(0xff, 0, 0)));
        assert_eq!(settings.color.address_selected.bg, Some(ratatui::style::Color::Black));
        assert_eq!(settings.color.address_default.fg, Some(ratatui::style::Color::Indexed(2)));
        assert_eq!(settings.color.address_default.bg, None);
        assert_eq!(settings.key.up, KeyEvent::from(KeyCode::Down));
        assert_eq!(settings.custom.get("string").unwrap(), &SettingsValue::from("World"));
        assert_eq!(settings.custom.get("integer").unwrap(), &SettingsValue::from(42));
        assert_eq!(settings.custom.get("float").unwrap(), &SettingsValue::from(3.14));
        assert_eq!(settings.custom.get("boolean").unwrap(), &SettingsValue::from(true));
        assert!(settings.custom.get("nil").is_none());
        assert_eq!(settings.custom.get("style").unwrap(), &SettingsValue::from(
            Style::new()
                .fg(ratatui::style::Color::Rgb(0xff, 0, 0))
                .bg(ratatui::style::Color::Rgb(0, 0, 0))
        ));
        assert_eq!(settings.custom.get("key").unwrap(), &SettingsValue::from(KeyEvent::from(KeyCode::Up)));
    }

    #[test]
    fn test_on_key_with_init()
    {
        let source = "
            command = nil
            function init(settings, context)
                command = settings.key_confirm
            end
            function on_key(key_event, data, current_byte, context)
                if key_event.code == command.code then
                    data:set(current_byte, 42)
                end
            end
        ";
        let mut settings = Settings::default();
        let mut context = AppContext::default();
        let mut plugin = Plugin::new_from_source(source, &mut settings, &mut context).unwrap();
        let mut data = vec![0; 0x100];
        let event = Event::Key { event: KeyEvent::from(KeyCode::Down), data: &mut data, current_byte: 0 };
        plugin.handle_with_error(event, &mut context).unwrap();
        assert_eq!(data[0], 0);
        let event = Event::Key { event: settings.key.confirm, data: &mut data, current_byte: 0 };
        plugin.handle_with_error(event, &mut context).unwrap();
        assert_eq!(data[0], 42);
    }

    #[test]
    fn test_log_from_lua()
    {
        let source = "
            function init(settings, context)
                context:log(1, \"Hello from init\")
            end

            function on_open(data, context)
                context:log(2, \"Hello from on_open\")
            end
        ";
        let mut settings = Settings::default();
        let mut context = AppContext::default();
        let mut plugin = Plugin::new_from_source(source, &mut settings, &mut context).unwrap();

        {
            let mut message_iter = context.logger.iter();
            let message = message_iter.next().unwrap();
            assert_eq!(message.level, NotificationLevel::Debug);
            assert_eq!(message.message, "Hello from init");
            assert_eq!(context.logger.get_notification_level(), NotificationLevel::Debug);
        }

        context.logger.clear();

        let mut data = vec![0; 0x100];
        let event = Event::Open { data: &mut data };
        plugin.handle_with_error(event, &mut context).unwrap();

        {
            let mut message_iter = context.logger.iter();
            let message = message_iter.next().unwrap();
            assert_eq!(message.level, NotificationLevel::Info);
            assert_eq!(message.message, "Hello from on_open");
            assert_eq!(context.logger.get_notification_level(), NotificationLevel::Info);
            assert!(message_iter.next().is_none());
        }
    }

    #[test]
    fn test_export_command()
    {
        let source = "
            function init(settings, context)
                context:add_command(\"test\", \"Test command\")
            end
        ";
        let mut settings = Settings::default();
        let mut context = AppContext::default();
        assert!(Plugin::new_from_source(source, &mut settings, &mut context).is_err(), 
            "Should not be able to export a command without defining it first");

        let source = "
            function init(settings, context)
                context:add_command(\"test\", \"Test command\")
                context:add_command(\"test3\", \"Test command 3\")
            end

            -- Add and remove commands
            function test(context)
                context:add_command(\"test2\", \"Test command 2\")
                context:remove_command(\"test\")
            end

            -- Intentional error
            function test2(context)
                context:add_command(\"does_not_exist\", \"This command does not exist\")
            end

            -- No duplicate command should be added
            function test3(context)
                context:add_command(\"test\", \"Test command\")
                context:add_command(\"test\", \"Test command 1\")
            end
        ";

        let mut plugin = Plugin::new_from_source(source, &mut settings, &mut context).unwrap();

        let commands = plugin.commands.get_commands();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].command, "test");
        assert_eq!(commands[0].description, "Test command");
        assert_eq!(commands[1].command, "test3");
        assert_eq!(commands[1].description, "Test command 3");

        plugin.run_command(&mut context, "test").unwrap();
        
        let commands = plugin.commands.get_commands();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].command, "test3");
        assert_eq!(commands[0].description, "Test command 3");
        assert_eq!(commands[1].command, "test2");
        assert_eq!(commands[1].description, "Test command 2");

        assert!(plugin.run_command(&mut context, "test2").is_err(), 
            "Should not be able to add a command that is not defined");
        
        let commands = plugin.commands.get_commands();
        assert_eq!(commands.len(), 2, 
            "No commands should be lost when an error occurs");
        assert_eq!(commands[0].command, "test3");
        assert_eq!(commands[0].description, "Test command 3");
        assert_eq!(commands[1].command, "test2");
        assert_eq!(commands[1].description, "Test command 2");

        plugin.run_command(&mut context, "test3").unwrap();

        let commands = plugin.commands.get_commands();
        assert_eq!(commands.len(), 3, 
            "No duplicate commands should be added");
        assert_eq!(commands[0].command, "test3");
        assert_eq!(commands[0].description, "Test command 3");
        assert_eq!(commands[1].command, "test2");
        assert_eq!(commands[1].description, "Test command 2");
        assert_eq!(commands[2].command, "test");
        assert_eq!(commands[2].description, "Test command 1", 
            "Should overwrite the description of the command");
    }
}