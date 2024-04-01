use std::error::Error;

use ratatui::text::{Line, Span};

use super::{color_settings::ColorSettings, notification::NotificationLevel, App};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command
{
    Quit,
    QuitWithoutSave,
    QuitWithSave,
    Save,
    Empty,
    Unknown,
}

impl Command
{
    pub fn get_commands() -> Vec<&'static str>
    {
        vec![
            "quit",
            "dquit",
            "xquit",
            "write",
        ]
    }
    pub fn from_string(command: &str) -> Command
    {
        match command
        {
            "quit" => Command::Quit,
            "dquit" => Command::QuitWithoutSave,
            "xquit" => Command::QuitWithSave,
            "write" => Command::Save,
            "" => Command::Empty,
            _ => Command::Unknown,
        }
    }

    pub fn to_line(&self, color_settings: &ColorSettings, selected: bool) -> Line<'static>
    {
        let (s0, s1) = if selected {
            (color_settings.command_selected, color_settings.command_selected)
        } else {
            (color_settings.command_name, color_settings.command_description)
        };
        match self
        {
            Command::Quit => Line::from(vec![Span::styled("quit", s0), Span::styled(" Quit the program.", s1)]),
            Command::QuitWithoutSave => Line::from(vec![Span::styled("dquit", s0), Span::styled(" Quit the program without saving.", s1)]),
            Command::QuitWithSave => Line::from(vec![Span::styled("xquit", s0), Span::styled(" Save and quit the program.", s1)]),
            Command::Save => Line::from(vec![Span::styled("write", s0), Span::styled(" Save the current file.", s1)]),
            Command::Empty => Line::from(vec![Span::styled("", s0), Span::styled("", s1)]),
            Command::Unknown => Line::from(vec![Span::styled("Unknown command", s0), Span::styled(" Unknown command", s1)]),
        }.left_aligned()
    }
}

impl <'a> App<'a>
{
    pub(super) fn find_commands(&mut self, command: &str) -> Vec<Command>
    {
        let ret = self.commands.fuzzy_search_sorted(command);
        ret.into_iter().map(|cmd| Command::from_string(&cmd)).collect()
    }

    pub(super) fn run_command(&mut self, command: &str, scroll: usize) -> Result<(), Box<dyn Error>>
    {
        let command_opt = self.find_commands(command).into_iter().skip(scroll).next();
        let command_enum = command_opt.expect("Scroll out of bounds for run_command.");
        match command_enum
        {
            Command::Quit => {
                self.quit(None)?;
            }
            Command::QuitWithoutSave => {
                self.quit(Some(false))?;
            }
            Command::QuitWithSave => {
                self.quit(Some(true))?;
            }
            Command::Save => {
                if self.dirty
                {
                    self.save_data()?;
                }
            }
            Command::Empty => {}
            Command::Unknown => {
                self.log(NotificationLevel::Error, &format!("Unknown command: \"{}\"", command));
            }
        }
        Ok(())
    }

    pub(super) fn quit(&mut self, save: Option<bool>) -> Result<(), Box<dyn Error>>
    {
        match save
        {
            Some(true) => {
                self.log(NotificationLevel::Debug, "Saving and quitting...");
                if self.dirty
                {
                    self.save_data()?;
                }
                self.needs_to_exit = true;       
            }
            Some(false) => {
                self.log(NotificationLevel::Debug, "Quitting without saving...");
                self.needs_to_exit = true;
            }
            None => {
                self.log(NotificationLevel::Debug, "Quitting...");
                if self.dirty
                {
                    self.log(NotificationLevel::Warning, "You have unsaved changes.")
                }
                else
                {
                    self.needs_to_exit = true;
                }
            }
        }
        Ok(())
    }
}