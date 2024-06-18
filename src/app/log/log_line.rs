use ratatui::text::{Line, Span};

use crate::app::settings::color_settings::ColorSettings;
use super::notification::NotificationLevel;

#[derive(Debug, Clone)]
pub struct LogLine
{
    pub level: NotificationLevel,
    pub message: String,
}

impl LogLine
{
    pub fn new(level: NotificationLevel, message: String) -> Self
    {
        LogLine
        {
            level,
            message,
        }
    }

    pub fn to_line(&self, color_settings: &ColorSettings) -> Line<'static>
    {
        let mut line = Line::default();
        let style = match self.level
        {
            NotificationLevel::Debug => color_settings.log_debug,
            NotificationLevel::Info => color_settings.log_info,
            NotificationLevel::Warning => color_settings.log_warning,
            NotificationLevel::Error => color_settings.log_error,
            _ => color_settings.log_info,
        };
        line.spans.push(Span::styled(self.level.to_string(), style));
        line.spans.push(Span::styled(" ", color_settings.log_message));
        line.spans.push(Span::styled(self.message.clone(), color_settings.log_message));
        line.left_aligned()
    }
}