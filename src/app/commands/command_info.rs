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
            CommandInfo::new("quit", "Quit the program."),
            CommandInfo::new("dquit", "Quit the program without saving."),
            CommandInfo::new("xquit", "Save and quit the program."),
            CommandInfo::new("save", "Save the current file."),
            CommandInfo::new("saveas", "Save the current file as a new file."),
            CommandInfo::new("help", "Display the help page."),
            CommandInfo::new("open", "Open a file."),
            CommandInfo::new("log", "Open the log."),
            CommandInfo::new("run", "Run a command."),
            CommandInfo::new("ftext", "Find text."),
            CommandInfo::new("fsym", "Find a symbol."),
            CommandInfo::new("fcom", "Find a comment."),
            CommandInfo::new("ecom", "Edit a comment."),
            CommandInfo::new("text", "Insert text."),
            CommandInfo::new("patch", "Patch assembly."),
            CommandInfo::new("jump", "Jump to address."),
            CommandInfo::new("view", "Switch between text and assembly."),
            CommandInfo::new("undo", "Undo the last change."),
            CommandInfo::new("redo", "Redo the last change."),
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
