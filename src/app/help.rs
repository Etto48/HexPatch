use ratatui::text::{Line, Span};

use super::{settings::{color_settings::ColorSettings, key_settings::KeySettings}, App};

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

    pub fn to_line(&self, color_settings: &ColorSettings) -> Line<'static> 
    {
        let mut line = Line::default();
        line.spans.push(Span::styled(self.command.to_string(), color_settings.help_command));
        line.spans.push(Span::styled(": ", color_settings.menu_text));
        line.spans.push(Span::styled(self.description.clone(), color_settings.menu_text));
        line.left_aligned()
    }
}

impl App
{
    pub fn key_event_to_string(event: crossterm::event::KeyEvent) -> String
    {
        let mut result = String::new();
        if event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
        {
            result.push_str("Ctrl+");
        }
        if event.modifiers.contains(crossterm::event::KeyModifiers::ALT)
        {
            result.push_str("Alt+");
        }
        if event.modifiers.contains(crossterm::event::KeyModifiers::SHIFT)
        {
            result.push_str("Shift+");
        }
        if event.modifiers.contains(crossterm::event::KeyModifiers::SUPER)
        {
            result.push_str("Super+");
        }
        if event.modifiers.contains(crossterm::event::KeyModifiers::HYPER)
        {
            result.push_str("Hyper+");
        }
        if event.modifiers.contains(crossterm::event::KeyModifiers::META)
        {
            result.push_str("Meta+");
        }
        
        result.push_str(
            &match event.code
            {
                crossterm::event::KeyCode::Left => "←".into(),
                crossterm::event::KeyCode::Right => "→".into(),
                crossterm::event::KeyCode::Up => "↑".into(),
                crossterm::event::KeyCode::Down => "↓".into(),
                crossterm::event::KeyCode::F(n) => 
                {
                    format!("F{}", n)
                },
                crossterm::event::KeyCode::Char(c) => 
                {
                    match c
                    {
                        ' ' => "Space".into(),
                        '\t' => "Tab".into(),
                        '\n' => "LF".into(),
                        '\r' => "CR".into(),
                        _ => c.to_ascii_uppercase().to_string()
                    }
                },
                c => format!("{:?}", c)
            }
        );
        result
    }

    pub(super) fn help_list(key_settings: &KeySettings) -> Vec<HelpLine>
    {
        vec![
            HelpLine::new(&Self::key_event_to_string(key_settings.up), "Move up"),
            HelpLine::new(&Self::key_event_to_string(key_settings.down), "Move down"),
            HelpLine::new(&Self::key_event_to_string(key_settings.left), "Move left"),
            HelpLine::new(&Self::key_event_to_string(key_settings.right), "Move right"),
            HelpLine::new(&Self::key_event_to_string(key_settings.next), "Next instruction or block"),
            HelpLine::new(&Self::key_event_to_string(key_settings.previous), "Previous instruction or block"),
            HelpLine::new(&Self::key_event_to_string(key_settings.page_up), "Scroll up"),
            HelpLine::new(&Self::key_event_to_string(key_settings.page_down), "Scroll down"),
            HelpLine::new(&Self::key_event_to_string(key_settings.goto_start), "Scroll to start"),
            HelpLine::new(&Self::key_event_to_string(key_settings.goto_end), "Scroll to end"),
            HelpLine::new(&Self::key_event_to_string(key_settings.run), "Run command"),
            HelpLine::new(&Self::key_event_to_string(key_settings.save), "Save"),
            HelpLine::new(&Self::key_event_to_string(key_settings.save_as), "Save as"),
            HelpLine::new(&Self::key_event_to_string(key_settings.save_and_quit), "Save and quit"),
            HelpLine::new(&Self::key_event_to_string(key_settings.quit), "Quit"),
            HelpLine::new(&Self::key_event_to_string(key_settings.open), "Open file"),
            HelpLine::new(&Self::key_event_to_string(key_settings.change_view), "Change view"),
            HelpLine::new(&Self::key_event_to_string(key_settings.jump), "Jump to location"),
            HelpLine::new(&Self::key_event_to_string(key_settings.find_symbol), "Search symbol"),
            HelpLine::new(&Self::key_event_to_string(key_settings.find_text), "Search text"),
            HelpLine::new(&Self::key_event_to_string(key_settings.patch_text), "Patch text"),
            HelpLine::new(&Self::key_event_to_string(key_settings.patch_assembly), "Patch assembly"),
            HelpLine::new(&Self::key_event_to_string(key_settings.log), "Open log"),
            HelpLine::new(&Self::key_event_to_string(key_settings.confirm), "Confirm"),
            HelpLine::new(&Self::key_event_to_string(key_settings.close_popup), "Close popup"),
            HelpLine::new(&Self::key_event_to_string(key_settings.new_line), "Insert new line (with multiline text)"),
            HelpLine::new(&Self::key_event_to_string(key_settings.clear_log), "Clear log (with log open)"),
            HelpLine::new(&Self::key_event_to_string(key_settings.help), "Help"),
        ]
    }
}