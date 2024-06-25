use crate::app::commands::command_info::CommandInfo;

#[derive(Debug, Default, Clone)]
pub struct ExportedCommands
{
    pub commands: Vec<CommandInfo>,
}

impl ExportedCommands
{
    /// If the command already exists, it will be overwritten.
    pub fn add_command(&mut self, command: String, description: String)
    { // TODO: maybe use a HashMap instead of a Vec if this gets slow
        if let Some(index) = self.commands.iter().position(|c| c.command == command)
        {
            self.commands[index].description = description;
        }
        else
        {
            self.commands.push(CommandInfo::new(command, description));
        }
    }

    pub fn remove_command(&mut self, command: &str) -> bool
    {
        if let Some(index) = self.commands.iter().position(|c| c.command == command)
        {
            self.commands.remove(index);
            true
        }
        else
        {
            false
        }
    }

    pub fn take(&mut self) -> Self
    {
        std::mem::take(self)
    }

    pub fn get_commands(&self) -> &[CommandInfo]
    {
        &self.commands
    }
}