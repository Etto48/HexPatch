use ratatui::text::{Line, Span};

use crate::app::settings::color_settings::ColorSettings;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryChoice {
    Yes,
    No,
}

impl BinaryChoice {
    pub fn to_line(&self, color_settings: &ColorSettings) -> Line<'static> {
        let mut ret = Line::from(vec![
            Span::styled("Yes", color_settings.yes),
            Span::raw("  "),
            Span::styled("No", color_settings.no),
        ]);

        match self {
            BinaryChoice::Yes => ret.spans[0].style = color_settings.yes_selected,
            BinaryChoice::No => ret.spans[2].style = color_settings.no_selected,
        }

        ret
    }

    pub fn next(&self) -> Self {
        match self {
            BinaryChoice::Yes => BinaryChoice::No,
            BinaryChoice::No => BinaryChoice::Yes,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            BinaryChoice::Yes => BinaryChoice::No,
            BinaryChoice::No => BinaryChoice::Yes,
        }
    }
}
