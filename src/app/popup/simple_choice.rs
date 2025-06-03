use ratatui::text::{Line, Span};

use crate::app::settings::color_settings::ColorSettings;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SimpleChoice {
    Yes,
    No,
    Cancel,
}

impl SimpleChoice {
    pub fn to_line(&self, color_settings: &ColorSettings) -> Line<'static> {
        let mut ret = Line::from(vec![
            Span::styled(t!("app.yes"), color_settings.yes),
            Span::raw("  "),
            Span::styled(t!("app.no"), color_settings.no),
            Span::raw("  "),
            Span::styled(t!("app.cancel"), color_settings.menu_text),
        ]);

        match self {
            SimpleChoice::Yes => ret.spans[0].style = color_settings.yes_selected,
            SimpleChoice::No => ret.spans[2].style = color_settings.no_selected,
            SimpleChoice::Cancel => ret.spans[4].style = color_settings.menu_text_selected,
        }

        ret
    }

    pub fn next(&self) -> Self {
        match self {
            SimpleChoice::Yes => SimpleChoice::No,
            SimpleChoice::No => SimpleChoice::Cancel,
            SimpleChoice::Cancel => SimpleChoice::Yes,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            SimpleChoice::Yes => SimpleChoice::Cancel,
            SimpleChoice::No => SimpleChoice::Yes,
            SimpleChoice::Cancel => SimpleChoice::No,
        }
    }
}
