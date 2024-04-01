use std::error::Error;

use super::{notification::NotificationLevel, App};

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
    pub fn from_string(command: &str) -> Command
    {
        let args = command.split_whitespace().collect::<Vec<&str>>();
        match args.get(0).cloned()
        {
            Some("quit") | Some("q") => {Command::Quit}
            Some("quit!") | Some("q!") => {Command::QuitWithoutSave}
            Some("quit+") | Some("x") => {Command::QuitWithSave}
            Some("save") | Some("w") => {Command::Save}
            Some(_) => {Command::Unknown}
            None => {Command::Empty}
        }
    }
}

impl <'a> App<'a>
{
    pub(super) fn run_command(&mut self, command: &str) -> Result<(), Box<dyn Error>>
    {
        self.log(NotificationLevel::Debug, &format!("Parsing command: \"{}\"", command));
        match Command::from_string(command)
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
                self.save_data()?;
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