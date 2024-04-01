use ratatui::text::{Line, Span};

use super::{color_settings::ColorSettings, App};

#[derive(Debug, Clone)]
pub struct HelpLine {
    pub command: String,
    pub description: String
}

impl HelpLine
{
    pub fn new(command: &str, description: &str) -> Self 
    {
        Self {
            command: command.to_string(),
            description: description.to_string()
        }
    }

    pub fn to_line(&self, color_settings: &ColorSettings) -> Line<'_> 
    {
        let mut line = Line::default();
        line.spans.push(Span::styled(format!("{:>2}",&self.command), color_settings.help_command));
        line.spans.push(Span::styled(": ", color_settings.menu_text));
        line.spans.push(Span::styled(&self.description, color_settings.menu_text));
        line.left_aligned()
    }
}

impl <'a> App<'a>
{
    pub(super) fn help_list() -> Vec<HelpLine>
    {
        vec![
            HelpLine::new("←→↑↓", "Move and scroll"),
            HelpLine::new("PgUp/PgDn", "Scroll page up/down"),
            HelpLine::new("Home/End", "Scroll to start/end"),
            HelpLine::new("^S", "Save"),
            HelpLine::new("^X", "Save and quit"),
            HelpLine::new("^C", "Quit"),
            HelpLine::new("V", "Change view"),
            HelpLine::new("J", "Jumpt to location"),
            HelpLine::new("S", "Search symbol"),
            HelpLine::new("P", "Patch assembly"),
            HelpLine::new("L", "Open log"),
            HelpLine::new("H", "Help"),
        ]
    }
}