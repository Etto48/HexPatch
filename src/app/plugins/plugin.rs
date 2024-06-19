use std::{error::Error, path::{Path, PathBuf}};

use mlua::{Function, Lua};

use crate::app::{log::logger::Logger, settings::Settings};

use super::{event::{Event, Events}, register_userdata::{register_settings, register_vec_u8}};

pub struct Plugin
{
    lua: Lua,
}

impl Plugin {
    pub fn new_from_source(source: &str, settings: &mut Settings) -> Result<Self, Box<dyn Error>>
    {
        let lua = Lua::new();
        lua.load(source).exec()?;

        register_vec_u8(&lua)?;
        register_settings(&lua)?;

        if let Ok(init) = lua.globals().get::<_, Function>("init")
        {
            lua.scope(|scope|{
                let settings = scope.create_any_userdata_ref_mut(settings)?;
                init.call(settings)
            })?;
        }
        
        Ok(Plugin { lua })
    }

    pub fn new_from_file(path: &str, settings: &mut Settings) -> Result<Self, Box<dyn Error>>
    {
        let source = std::fs::read_to_string(path)?;
        Self::new_from_source(&source, settings)
    }

    fn get_default_plugin_path() -> Option<PathBuf>
    {
        let config = dirs::config_dir()?;
        Some(config.join("HexPatch").join("plugins"))
    }

    pub fn load_plugins(log: &mut Logger, settings: &mut Settings, path: Option<&Path>) -> std::io::Result<Vec<Plugin>>
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
        for entry in std::fs::read_dir(path)?
        {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "lua"
            {
                match Plugin::new_from_file(&path.to_string_lossy(), settings)
                {
                    Ok(plugin) => plugins.push(plugin),
                    Err(e) => log.log(crate::app::log::notification::NotificationLevel::Error, &format!("Could not load plugin \"{}\": {}", path.to_string_lossy(), e)),
                }
            }
        }
        Ok(plugins)
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

    pub fn handle(&self, event: Event)
    {
        match event
        {
            Event::Open { data } =>
            {
                // Call the on_open function
                let on_open = self.lua.globals().get::<_, Function>("on_open").unwrap();
                self.lua.scope(|scope| {
                    let data = scope.create_any_userdata_ref_mut(data)?;
                    on_open.call::<_,()>(data)
                }).unwrap();
            },
            Event::Edit { data, starting_byte, new_bytes } =>
            {
                // Call the on_edit function
                let on_edit = self.lua.globals().get::<_, Function>("on_edit").unwrap();
                self.lua.scope(|scope| {
                    let data = scope.create_any_userdata_ref_mut(data)?;
                    let new_bytes = scope.create_any_userdata_ref_mut(new_bytes)?;
                    on_edit.call::<_,()>((data, starting_byte, new_bytes))
                }).unwrap();
            },
            Event::Save { data } =>
            {
                // Call the on_save function
                let on_save = self.lua.globals().get::<_, Function>("on_save").unwrap();
                self.lua.scope(|scope| {
                    let data = scope.create_any_userdata_ref_mut(data)?;
                    on_save.call::<_,()>(data)
                }).unwrap();
            },
            Event::Key {code, modifiers, kind, state} =>
            {
                // Call the on_key function
                let on_key = self.lua.globals().get::<_, Function>("on_key").unwrap();
                on_key.call::<_,()>((code, modifiers, kind, state)).unwrap();
            },
            Event::Mouse {kind, row, col} =>
            {
                // Call the on_mouse function
                let on_mouse = self.lua.globals().get::<_, Function>("on_mouse").unwrap();
                on_mouse.call::<_,()>((kind, row, col)).unwrap();
            },
        }
    }
}

#[cfg(test)]
mod test
{
    use crossterm::event::{KeyCode, KeyEvent};

    use super::*;

    #[test]
    fn test_init_plugin()
    {
        let test_value = 42;
        let source = format!("
            test_value = 0
            function init()
                test_value = {test_value}
            end
        ");
        let mut settings = Settings::default();
        let plugin = Plugin::new_from_source(&source, &mut settings).unwrap();
        assert_eq!(plugin.lua.globals().get::<_, i32>("test_value").unwrap(), test_value);
    }

    #[test]
    fn test_discover_event_handlers()
    {
        let source = "
            function on_open(data) end
            function on_edit(data, selected_byte, new_bytes) end
            function on_save(data) end
            function on_key(code, modifiers, kind, state) end
            function on_mouse(kind, row, col) end
        ";
        let mut settings = Settings::default();
        let plugin = Plugin::new_from_source(source, &mut settings).unwrap();
        let handlers = plugin.get_event_handlers();
        assert_eq!(handlers, Events::ON_OPEN | Events::ON_EDIT | Events::ON_SAVE | Events::ON_KEY | Events::ON_MOUSE);
        let source = "
            function on_open(data) end
            function on_edit(data, selected_byte, new_bytes) end
            function on_save(data) end
        ";
        let plugin = Plugin::new_from_source(source, &mut settings).unwrap();
        let handlers = plugin.get_event_handlers();
        assert_eq!(handlers, Events::ON_OPEN | Events::ON_EDIT | Events::ON_SAVE);
    }

    #[test]
    fn test_edit_open_data()
    {
        let source = "
            function on_open(data)
                data:set(0,42)
            end
        ";
        let mut settings = Settings::default();
        let plugin = Plugin::new_from_source(source, &mut settings).unwrap();
        let mut data = vec![0; 0x100];
        let event = Event::Open { data: &mut data };
        plugin.handle(event);
        assert_eq!(data[0], 42);
    }

    #[test]
    fn test_init_change_settings()
    {
        let source = "
            function init(settings)
                settings.color_address_selected_fg = \"#FF0000\"
                settings.color_address_selected_bg = \"Black\"
                settings.color_address_default_fg = \"2\"
                settings.color_address_default_bg = nil

                settings.key_up = {code=\"Down\",modifiers=0,kind=\"Press\",state=0}
            end
        ";
        let mut settings = Settings::default();
        let _ = Plugin::new_from_source(source, &mut settings).unwrap();
        assert_eq!(settings.color.address_selected.fg, Some(ratatui::style::Color::Rgb(0xff, 0, 0)));
        assert_eq!(settings.color.address_selected.bg, Some(ratatui::style::Color::Black));
        assert_eq!(settings.color.address_default.fg, Some(ratatui::style::Color::Indexed(2)));
        assert_eq!(settings.color.address_default.bg, None);
        assert_eq!(settings.key.up, KeyEvent::from(KeyCode::Down));
    }
}