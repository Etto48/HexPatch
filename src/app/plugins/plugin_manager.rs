use std::path::{Path, PathBuf};

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::text::Text;

use crate::app::{commands::command_info::CommandInfo, log::{logger::Logger, NotificationLevel}, popup_state::PopupState, settings::Settings};

use super::{app_context::AppContext, context_refs::ContextRefs, event::{Event, Events}, plugin::Plugin};

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

    pub fn load(
        path: Option<&Path>, 
        log: &mut Logger, 
        popup: &mut Option<PopupState>, 
        settings: &mut Settings) -> std::io::Result<Self> {
        let mut plugin_manager = Self {
            plugins: Self::load_plugins(log, popup, settings, path)?,
            ..Default::default()
        };

        for (i, plugin) in plugin_manager.plugins.iter().enumerate() {
            let handlers = plugin.get_event_handlers();
            if handlers.contains(Events::ON_OPEN)
            {
                plugin_manager.on_open.push(i);
            }
            if handlers.contains(Events::ON_SAVE)
            {
                plugin_manager.on_save.push(i);
            }
            if handlers.contains(Events::ON_EDIT)
            {
                plugin_manager.on_edit.push(i);
            }
            if handlers.contains(Events::ON_KEY)
            {
                plugin_manager.on_key.push(i);
            }
            if handlers.contains(Events::ON_MOUSE)
            {
                plugin_manager.on_mouse.push(i);
            }
        }
        Ok(plugin_manager)
    }

    fn get_default_plugin_path() -> Option<PathBuf>
    {
        let config = dirs::config_dir()?;
        Some(config.join("HexPatch").join("plugins"))
    }

    fn load_plugins(
        log: &mut Logger, 
        popup: &mut Option<PopupState>, 
        settings: &mut Settings, 
        path: Option<&Path>) -> std::io::Result<Vec<Plugin>>
    {
        let mut plugins = Vec::new();
        let path = match path
        {
            Some(path) => path.to_path_buf(),
            None => Self::get_default_plugin_path().ok_or(
                std::io::Error::new(std::io::ErrorKind::Other, "Could not get default plugin path")
            )?
        };
        std::fs::create_dir_all(&path)?;
        let mut context = AppContext::default();
        for entry in std::fs::read_dir(path)?
        {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "lua"
            {
                match Plugin::new_from_file(&path.to_string_lossy(), settings, &mut context)
                {
                    Ok(plugin) => 
                    {
                        plugins.push(plugin);
                    },
                    Err(e) => log.log(NotificationLevel::Error, &format!("Could not load plugin \"{}\": {}", path.to_string_lossy(), e)),
                }
            }
        }
        log.merge(&context.logger);
        Self::popup_if_needed(popup, &context);
        Ok(plugins)
    }

    pub fn on_open(
        &mut self, 
        context_refs: &mut ContextRefs)
    {
        let mut context = AppContext::new();
        for i in self.on_open.iter()
        {
            context.plugin_index = Some(*i);
            let event = Event::Open;
            self.plugins[*i].handle(event, &mut context, context_refs);
        }
        context_refs.logger.merge(&context.logger);
        Self::popup_if_needed(context_refs.popup, &context);
    }

    pub fn on_save(
        &mut self, 
        context_refs: &mut ContextRefs)
    {
        let mut context = AppContext::new();
        for i in self.on_save.iter()
        {
            context.plugin_index = Some(*i);
            let event = Event::Save;
            self.plugins[*i].handle(event, &mut context, context_refs);
        }
        context_refs.logger.merge(&context.logger);
        Self::popup_if_needed(context_refs.popup, &context)
    }

    pub fn on_edit(
        &mut self, 
        new_bytes: &mut Vec<u8>,
        context_refs: &mut ContextRefs)
    {
        let mut context = AppContext::new();
        for i in self.on_edit.iter()
        {
            context.plugin_index = Some(*i);
            let event = Event::Edit { 
                new_bytes
            };
            self.plugins[*i].handle(event, &mut context, context_refs);
        }
        context_refs.logger.merge(&context.logger);
        Self::popup_if_needed(context_refs.popup, &context);
    }

    pub fn on_key(
        &mut self, 
        event: KeyEvent,
        context_refs: &mut ContextRefs)
    {
        let mut context = AppContext::new();
        for i in self.on_key.iter()
        {
            context.plugin_index = Some(*i);
            let event = Event::Key { 
                event, 
            };
            self.plugins[*i].handle(event, &mut context, context_refs);
        }
        context_refs.logger.merge(&context.logger);
        Self::popup_if_needed(context_refs.popup, &context);
    }

    pub fn on_mouse(
        &mut self, 
        event: MouseEvent, 
        context_refs: &mut ContextRefs)
    {
        let mut context = AppContext::new();
        for i in self.on_mouse.iter()
        {
            context.plugin_index = Some(*i);
            let event = Event::Mouse { event };
            self.plugins[*i].handle(event, &mut context, context_refs);
        }
        context_refs.logger.merge(&context.logger);
        Self::popup_if_needed(context_refs.popup, &context);
    }

    pub fn get_commands(&self) -> Vec<&CommandInfo>
    {
        let mut commands = Vec::new();
        let command_count = self.plugins.iter().map(|p| p.get_commands().len()).sum();
        commands.reserve(command_count);
        for plugin in self.plugins.iter()
        {
            commands.extend(plugin.get_commands());
        }
        commands
    }

    pub fn run_command(
        &mut self, 
        command: &str, 
        context_refs: &mut ContextRefs) -> mlua::Result<()>
    {
        let mut context = AppContext::new();
        let mut found = false;
        for (i, plugin) in self.plugins.iter_mut().enumerate()
        {
            if let Some(_command_info) = plugin.get_commands()
                .iter()
                .find(|c| c.command == command)
            {
                context.plugin_index = Some(i);
                plugin.run_command(
                    command,
                    &mut context, 
                    context_refs
                )?;
                found = true;
                break;
            }
        }
        context_refs.logger.merge(&context.logger);
        Self::popup_if_needed(context_refs.popup, &context);
        if !found { Err(mlua::Error::external(format!("Command \"{}\" not found", command))) } else { Ok(()) }
    }

    pub fn fill_popup(
        &mut self, 
        plugin_index: usize,
        callback: impl AsRef<str>,
        popup_text: &mut Text<'static>,
        popup_title: &mut String,
        context_refs: ContextRefs) -> mlua::Result<()> 
    {
        self.plugins[plugin_index].fill_popup(
            callback,
            popup_text, 
            popup_title,
            context_refs)
    }

    fn popup_if_needed(popup: &mut Option<PopupState>, context: &AppContext)
    {
        if let Some((plugin_index, callback)) = &context.popup
        {
            popup.replace(PopupState::Custom { 
                plugin_index: *plugin_index, 
                callback: callback.clone() 
            });
        }
    }
}

#[cfg(test)]
mod test
{
    use crate::{app::log::NotificationLevel, headers::Header};

    use super::*;

    #[test]
    fn test_load_plugins()
    {
        let mut log = Logger::new();
        let mut settings = Settings::default();
        let path = std::path::Path::new("test/plugins");
        let mut popup_state = None;
        let mut plugin_manager = PluginManager::load(Some(path), &mut log, &mut popup_state, &mut settings).unwrap();
        assert_eq!(plugin_manager.plugins.len(), 2);
        let mut data = vec![0; 0x100];
        let header = Header::default();
        let mut popup_state = None;
        let mut context_refs = ContextRefs::new(0, None, &mut data, &header, &settings, &mut log, &mut popup_state);

        plugin_manager.run_command("p1c1", &mut context_refs).unwrap();
        plugin_manager.run_command("p1c2", &mut context_refs).unwrap();
        plugin_manager.run_command("p2c1", &mut context_refs).unwrap();
        plugin_manager.run_command("p2c2", &mut context_refs).unwrap();

        plugin_manager.on_open(&mut context_refs);
        // If there was an error, the logger will have a message
        assert_ne!(log.get_notification_level(), NotificationLevel::Error);

        let messages: Vec<_> = log.iter().collect();
        assert_eq!(messages.len(), 5, "{:?}", messages);
        assert_eq!(messages[0].message, "Plugin 1 Command 1 called");
        assert_eq!(messages[1].message, "Plugin 1 Command 2 called");
        assert_eq!(messages[2].message, "Plugin 2 Command 1 called");
        assert_eq!(messages[3].message, "Plugin 2 Command 2 called");
        assert_eq!(messages[4].message, "Plugin 1 on_open called");
    }
}