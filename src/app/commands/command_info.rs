use ratatui::text::{Line, Span};

use crate::app::{plugins::plugin_manager::PluginManager, settings::color_settings::ColorSettings};

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub command: String,
    pub description: String,
}

impl CommandInfo {
    pub fn new(command: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            description: description.into(),
        }
    }

    pub fn default_commands() -> Vec<CommandInfo> {
        vec![
            CommandInfo::new("quit", t!("app.commands.quit")),
            CommandInfo::new("dquit", t!("app.commands.dquit")),
            CommandInfo::new("xquit", t!("app.commands.xquit")),
            CommandInfo::new("save", t!("app.commands.save")),
            CommandInfo::new("saveas", t!("app.commands.saveas")),
            CommandInfo::new("csave", t!("app.commands.csave")),
            CommandInfo::new("help", t!("app.commands.help")),
            CommandInfo::new("open", t!("app.commands.open")),
            CommandInfo::new("log", t!("app.commands.log")),
            CommandInfo::new("run", t!("app.commands.run")),
            CommandInfo::new("ftext", t!("app.commands.ftext")),
            CommandInfo::new("fsym", t!("app.commands.fsym")),
            CommandInfo::new("fcom", t!("app.commands.fcom")),
            CommandInfo::new("ecom", t!("app.commands.ecom")),
            CommandInfo::new("text", t!("app.commands.text")),
            CommandInfo::new("patch", t!("app.commands.patch")),
            CommandInfo::new("jump", t!("app.commands.jump")),
            CommandInfo::new("view", t!("app.commands.view")),
            CommandInfo::new("undo", t!("app.commands.undo")),
            CommandInfo::new("redo", t!("app.commands.redo")),
        ]
    }

    pub fn full_list_of_commands(plugin_manager: &PluginManager) -> Vec<CommandInfo> {
        let mut commands = Self::default_commands();
        commands.extend(plugin_manager.get_commands().iter().map(|&c| c.clone()));
        commands
    }

    pub fn to_line(&self, color_settings: &ColorSettings, selected: bool) -> Line<'static> {
        let (s0, s1) = if selected {
            (
                color_settings.command_selected,
                color_settings.command_selected,
            )
        } else {
            (
                color_settings.command_name,
                color_settings.command_description,
            )
        };
        Line::from(vec![
            Span::styled(self.command.clone(), s0),
            Span::styled(" ", s0),
            Span::styled(self.description.clone(), s1),
        ])
        .left_aligned()
    }
}

impl AsRef<str> for CommandInfo {
    fn as_ref(&self) -> &str {
        &self.command
    }
}
