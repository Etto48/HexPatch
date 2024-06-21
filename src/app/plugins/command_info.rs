#[derive(Debug, Clone)]
pub struct CommandInfo
{
    pub command: String,
    pub description: String,
}

impl CommandInfo
{
    pub fn new(command: String, description: String) -> Self
    {
        Self { command, description }
    }
}
