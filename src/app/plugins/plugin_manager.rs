use std::path::{Path, PathBuf};

use crossterm::event::{KeyEvent, MouseEvent};

use crate::{
    app::{commands::command_info::CommandInfo, log::NotificationLevel},
    headers::custom_header::CustomHeader,
};

use super::{
    app_context::AppContext,
    event::{Event, Events},
    plugin::Plugin,
    popup_context::PopupContext,
    ui_location::ui_location::UiLocation,
};

#[derive(Default, Debug)]
pub struct PluginManager {
    plugins: Vec<Plugin>,
    on_open: Vec<usize>,
    on_save: Vec<usize>,
    on_edit: Vec<usize>,
    on_key: Vec<usize>,
    on_mouse: Vec<usize>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(path: Option<&Path>, app_context: &mut AppContext) -> std::io::Result<Self> {
        let mut plugin_manager = Self {
            plugins: Self::load_plugins(path, app_context)?,
            ..Default::default()
        };

        for (i, plugin) in plugin_manager.plugins.iter().enumerate() {
            let handlers = plugin.get_event_handlers();
            if handlers.contains(Events::ON_OPEN) {
                plugin_manager.on_open.push(i);
            }
            if handlers.contains(Events::ON_SAVE) {
                plugin_manager.on_save.push(i);
            }
            if handlers.contains(Events::ON_EDIT) {
                plugin_manager.on_edit.push(i);
            }
            if handlers.contains(Events::ON_KEY) {
                plugin_manager.on_key.push(i);
            }
            if handlers.contains(Events::ON_MOUSE) {
                plugin_manager.on_mouse.push(i);
            }
        }
        Ok(plugin_manager)
    }

    fn get_default_plugin_path() -> Option<PathBuf> {
        let config = dirs::config_dir()?;
        Some(config.join("HexPatch").join("plugins"))
    }

    fn load_plugins(
        path: Option<&Path>,
        app_context: &mut AppContext,
    ) -> std::io::Result<Vec<Plugin>> {
        let mut plugins = Vec::new();
        let path = match path {
            Some(path) => path.to_path_buf(),
            None => Self::get_default_plugin_path()
                .ok_or(std::io::Error::other(t!("errors.get_default_plugin_path")))?,
        };
        std::fs::create_dir_all(&path)?;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "lua" {
                match Plugin::new_from_file(&path.to_string_lossy(), app_context) {
                    Ok(plugin) => {
                        plugins.push(plugin);
                    }
                    Err(e) => app_context.logger.log(
                        NotificationLevel::Error,
                        t!(
                            "app.messages.plugin_load_error",
                            path = path.to_string_lossy(),
                            error = e
                        ),
                    ),
                }
            }
        }
        Ok(plugins)
    }

    pub fn on_open(&mut self, app_context: &mut AppContext) {
        for i in self.on_open.iter() {
            app_context.plugin_index = Some(*i);
            let event = Event::Open;
            self.plugins[*i].handle(event, app_context);
        }
    }

    pub fn on_save(&mut self, app_context: &mut AppContext) {
        for i in self.on_save.iter() {
            app_context.plugin_index = Some(*i);
            let event = Event::Save;
            self.plugins[*i].handle(event, app_context);
        }
    }

    pub fn on_edit(&mut self, new_bytes: &mut Vec<u8>, app_context: &mut AppContext) {
        for i in self.on_edit.iter() {
            app_context.plugin_index = Some(*i);
            let event = Event::Edit { new_bytes };
            self.plugins[*i].handle(event, app_context);
        }
    }

    pub fn on_key(&mut self, event: KeyEvent, app_context: &mut AppContext) {
        for i in self.on_key.iter() {
            app_context.plugin_index = Some(*i);
            let event = Event::Key { event };
            self.plugins[*i].handle(event, app_context);
        }
    }

    pub fn on_mouse(
        &mut self,
        event: MouseEvent,
        location: Option<UiLocation>,
        app_context: &mut AppContext,
    ) {
        for i in self.on_mouse.iter() {
            app_context.plugin_index = Some(*i);
            let event = Event::Mouse {
                event,
                location: location.clone(),
            };
            self.plugins[*i].handle(event, app_context);
        }
    }

    pub fn on_focus(&mut self, app_context: &mut AppContext) {
        for i in self.on_open.iter() {
            app_context.plugin_index = Some(*i);
            let event = Event::Focus;
            self.plugins[*i].handle(event, app_context);
        }
    }

    pub fn on_blur(&mut self, app_context: &mut AppContext) {
        for i in self.on_open.iter() {
            app_context.plugin_index = Some(*i);
            let event = Event::Blur;
            self.plugins[*i].handle(event, app_context);
        }
    }

    pub fn on_paste(&mut self, text: impl AsRef<str>, app_context: &mut AppContext) {
        for i in self.on_open.iter() {
            app_context.plugin_index = Some(*i);
            let event = Event::Paste {
                text: text.as_ref().to_string(),
            };
            self.plugins[*i].handle(event, app_context);
        }
    }

    pub fn on_resize(&mut self, width: u16, height: u16, app_context: &mut AppContext) {
        for i in self.on_open.iter() {
            app_context.plugin_index = Some(*i);
            let event = Event::Resize { width, height };
            self.plugins[*i].handle(event, app_context);
        }
    }

    pub fn get_commands(&self) -> Vec<&CommandInfo> {
        let mut commands = Vec::new();
        let command_count = self.plugins.iter().map(|p| p.get_commands().len()).sum();
        commands.reserve(command_count);
        for plugin in self.plugins.iter() {
            commands.extend(plugin.get_commands());
        }
        commands
    }

    pub fn run_command(&mut self, command: &str, app_context: &mut AppContext) -> mlua::Result<()> {
        let mut found = false;
        for (i, plugin) in self.plugins.iter_mut().enumerate() {
            if let Some(_command_info) = plugin.get_commands().iter().find(|c| c.command == command)
            {
                app_context.plugin_index = Some(i);
                plugin.run_command(command, app_context)?;
                found = true;
                break;
            }
        }
        if !found {
            Err(mlua::Error::external(format!(
                "Command \"{command}\" not found"
            )))
        } else {
            Ok(())
        }
    }

    pub fn fill_popup(
        &mut self,
        plugin_index: usize,
        callback: impl AsRef<str>,
        popup_context: PopupContext,
        app_context: AppContext,
    ) -> mlua::Result<()> {
        self.plugins[plugin_index].fill_popup(callback, popup_context, app_context)
    }

    pub fn try_parse_header(&mut self, app_context: &mut AppContext) -> Option<CustomHeader> {
        for (i, plugin) in self.plugins.iter_mut().enumerate() {
            app_context.plugin_index = Some(i);
            if let Some(header) = plugin.try_parse_header(app_context) {
                return Some(header);
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use crate::{
        app::{log::NotificationLevel, App},
        get_app_context,
    };

    use super::*;

    #[test]
    fn test_load_plugins() {
        let mut app = App::mockup(vec![0; 0x100]);
        app.logger.clear();
        let mut app_context = get_app_context!(app);
        let test_plugins_path = Path::new("test").join("plugins");
        app.plugin_manager =
            PluginManager::load(Some(&test_plugins_path), &mut app_context).unwrap();
        assert_eq!(app.plugin_manager.plugins.len(), 2);

        app.plugin_manager
            .run_command("p1c1", &mut app_context)
            .unwrap();
        app.plugin_manager
            .run_command("p1c2", &mut app_context)
            .unwrap();
        app.plugin_manager
            .run_command("p2c1", &mut app_context)
            .unwrap();
        app.plugin_manager
            .run_command("p2c2", &mut app_context)
            .unwrap();

        app.plugin_manager.on_open(&mut app_context);
        // If there was an error, the logger will have a message
        assert_ne!(
            app_context.logger.get_notification_level(),
            NotificationLevel::Error
        );

        let messages: Vec<_> = app_context.logger.iter().collect();
        assert_eq!(messages.len(), 5, "{messages:?}");
        assert_eq!(messages[0].message, "Plugin 1 Command 1 called");
        assert_eq!(messages[1].message, "Plugin 1 Command 2 called");
        assert_eq!(messages[2].message, "Plugin 2 Command 1 called");
        assert_eq!(messages[3].message, "Plugin 2 Command 2 called");
        assert_eq!(messages[4].message, "Plugin 1 on_open called");
    }
}
