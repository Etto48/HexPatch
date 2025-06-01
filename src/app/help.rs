use ratatui::text::{Line, Span};

use super::{
    settings::{color_settings::ColorSettings, key_settings::KeySettings},
    App,
};

#[derive(Debug, Clone)]
pub struct HelpLine {
    pub command: String,
    pub description: String,
}

impl HelpLine {
    pub fn new(command: &str, description: &str) -> Self {
        Self {
            command: command.to_string(),
            description: description.to_string(),
        }
    }

    pub fn to_line(&self, color_settings: &ColorSettings) -> Line<'static> {
        let mut line = Line::default();
        line.spans.push(Span::styled(
            self.command.to_string(),
            color_settings.help_command,
        ));
        line.spans
            .push(Span::styled(": ", color_settings.menu_text));
        line.spans.push(Span::styled(
            self.description.clone(),
            color_settings.help_description,
        ));
        line.left_aligned()
    }
}

impl App {
    pub fn key_event_to_string(event: crossterm::event::KeyEvent) -> String {
        let mut result = String::new();
        if event
            .modifiers
            .contains(crossterm::event::KeyModifiers::CONTROL)
        {
            result.push_str(&t!("keys.mods.control"));
        }
        if event
            .modifiers
            .contains(crossterm::event::KeyModifiers::ALT)
        {
            result.push_str(&t!("keys.mods.alt"));
        }
        if event
            .modifiers
            .contains(crossterm::event::KeyModifiers::SHIFT)
        {
            result.push_str(&t!("keys.mods.shift"));
        }
        if event
            .modifiers
            .contains(crossterm::event::KeyModifiers::SUPER)
        {
            result.push_str(&t!("keys.mods.super"));
        }
        if event
            .modifiers
            .contains(crossterm::event::KeyModifiers::HYPER)
        {
            result.push_str(&t!("keys.mods.hyper"));
        }
        if event
            .modifiers
            .contains(crossterm::event::KeyModifiers::META)
        {
            result.push_str(&t!("keys.mods.meta"));
        }

        result.push_str(&match event.code {
            crossterm::event::KeyCode::Left => "←".into(),
            crossterm::event::KeyCode::Right => "→".into(),
            crossterm::event::KeyCode::Up => "↑".into(),
            crossterm::event::KeyCode::Down => "↓".into(),
            crossterm::event::KeyCode::F(n) => {
                format!("F{}", n)
            }
            crossterm::event::KeyCode::Char(c) => match c {
                ' ' => t!("keys.space").into(),
                '\t' => t!("keys.tab").into(),
                '\n' => "LF".into(),
                '\r' => "CR".into(),
                _ => c.to_ascii_uppercase().to_string(),
            },
            c => format!("{:?}", c),
        });
        result
    }

    pub(super) fn help_list(key_settings: &KeySettings) -> Vec<HelpLine> {
        vec![
            HelpLine::new(
                &Self::key_event_to_string(key_settings.up),
                &t!("app.help.up"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.down),
                &t!("app.help.down"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.left),
                &t!("app.help.left"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.right),
                &t!("app.help.right"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.next),
                &t!("app.help.next"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.previous),
                &t!("app.help.previous"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.page_up),
                &t!("app.help.page_up"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.page_down),
                &t!("app.help.page_down"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.goto_start),
                &t!("app.help.goto_start"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.goto_end),
                &t!("app.help.goto_end"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.run),
                &t!("app.help.run"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.save),
                &t!("app.help.save"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.save_as),
                &t!("app.help.save_as"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.save_and_quit),
                &t!("app.help.save_and_quit"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.quit),
                &t!("app.help.quit"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.open),
                &t!("app.help.open"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.change_view),
                &t!("app.help.change_view"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.change_selected_pane),
                &t!("app.help.change_selected_pane"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.fullscreen),
                &t!("app.help.fullscreen"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.jump),
                &t!("app.help.jump"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.find_symbol),
                &t!("app.help.find_symbol"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.find_text),
                &t!("app.help.find_text"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.patch_text),
                &t!("app.help.patch_text"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.patch_assembly),
                &t!("app.help.patch_assembly"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.log),
                &t!("app.help.log"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.confirm),
                &t!("app.help.confirm"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.close_popup),
                &t!("app.help.close_popup"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.new_line),
                &t!("app.help.new_line"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.clear_log),
                &t!("app.help.clear_log"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.undo),
                &t!("app.help.undo"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.redo),
                &t!("app.help.redo"),
            ),
            HelpLine::new(
                &Self::key_event_to_string(key_settings.help),
                &t!("app.help.help"),
            ),
        ]
    }
}
