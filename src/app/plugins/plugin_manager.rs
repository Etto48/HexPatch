use std::path::{Path, PathBuf};

use crate::app::{commands::command_info::CommandInfo, log::logger::Logger, settings::Settings};

use super::{app_context::AppContext, event::{Event, Events}, plugin::Plugin};

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

    pub fn load(path: Option<&Path>, log: &mut Logger, settings: &mut Settings) -> std::io::Result<Self> {
        let mut plugin_manager = Self {
            plugins: Self::load_plugins(log, settings, path)?,
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

    fn load_plugins(log: &mut Logger, settings: &mut Settings, path: Option<&Path>) -> std::io::Result<Vec<Plugin>>
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
                    Err(e) => log.log(crate::app::log::notification::NotificationLevel::Error, &format!("Could not load plugin \"{}\": {}", path.to_string_lossy(), e)),
                }
            }
        }
        log.merge(&context.logger);
        Ok(plugins)
    }

    pub fn on_open(&mut self, data: &mut Vec<u8>, logger: &mut Logger) -> mlua::Result<()>
    {
        let mut context = AppContext::new();
        for i in self.on_open.iter()
        {
            let event = Event::Open { data };
            self.plugins[*i].handle(event, &mut context)?;
        }
        logger.merge(&context.logger);
        Ok(())
    }

    pub fn on_save(&mut self, data: &mut Vec<u8>, logger: &mut Logger) -> mlua::Result<()>
    {
        let mut context = AppContext::new();
        for i in self.on_save.iter()
        {
            let event = Event::Save { data };
            self.plugins[*i].handle(event, &mut context)?;
        }
        logger.merge(&context.logger);
        Ok(())
    }

    pub fn on_edit(&mut self, data: &mut Vec<u8>, offset: usize, new_bytes: &mut Vec<u8>, logger: &mut Logger) -> mlua::Result<()>
    {
        let mut context = AppContext::new();
        for i in self.on_edit.iter()
        {
            let event = Event::Edit { data, offset, new_bytes };
            self.plugins[*i].handle(event, &mut context)?;
        }
        logger.merge(&context.logger);
        Ok(())
    }

    pub fn on_key(&mut self, event: crossterm::event::KeyEvent, data: &mut Vec<u8>, current_byte: usize, logger: &mut Logger) -> mlua::Result<()>
    {
        let mut context = AppContext::new();
        for i in self.on_key.iter()
        {
            let event = Event::Key { event, data, current_byte };
            self.plugins[*i].handle(event, &mut context)?;
        }
        logger.merge(&context.logger);
        Ok(())
    }

    pub fn on_mouse(&mut self, kind: String, row: u16, col: u16, logger: &mut Logger) -> mlua::Result<()>
    {
        let mut context = AppContext::new();
        for i in self.on_mouse.iter()
        {
            let event = Event::Mouse { kind: kind.clone(), row, col };
            self.plugins[*i].handle(event, &mut context)?;
        }
        logger.merge(&context.logger);
        Ok(())
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

    pub fn run_command(&mut self, command: &str, logger: &mut Logger) -> mlua::Result<()>
    {
        let mut context = AppContext::new();
        let mut found = false;
        for plugin in self.plugins.iter_mut()
        {
            if let Some(_command_info) = plugin.get_commands()
                .iter()
                .find(|c| c.command == command)
            {
                plugin.run_command(&mut context, command)?;
                found = true;
                break;
            }
        }
        logger.merge(&context.logger);
        if !found { Err(mlua::Error::external(format!("Command \"{}\" not found", command))) } else { Ok(()) }
    }
}

#[cfg(test)]
mod test
{
    use super::*;

    #[test]
    fn test_load_plugins()
    {
        let mut log = Logger::new();
        let mut settings = Settings::default();
        let path = std::path::Path::new("test/plugins");
        let mut plugin_manager = PluginManager::load(Some(path), &mut log, &mut settings).unwrap();
        assert_eq!(plugin_manager.plugins.len(), 2);

        plugin_manager.run_command("p1c1", &mut log).unwrap();
        plugin_manager.run_command("p1c2", &mut log).unwrap();
        plugin_manager.run_command("p2c1", &mut log).unwrap();
        plugin_manager.run_command("p2c2", &mut log).unwrap();

        plugin_manager.on_open(&mut Vec::new(), &mut log).unwrap();

        let messages: Vec<_> = log.iter().collect();
        assert_eq!(messages.len(), 5, "{:?}", messages);
        assert_eq!(messages[0].message, "Plugin 1 Command 1 called");
        assert_eq!(messages[1].message, "Plugin 1 Command 2 called");
        assert_eq!(messages[2].message, "Plugin 2 Command 1 called");
        assert_eq!(messages[3].message, "Plugin 2 Command 2 called");
        assert_eq!(messages[4].message, "Plugin 1 on_open called");
    }
}