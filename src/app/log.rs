use ratatui::text::{Line, Span};

use super::{color_settings::ColorSettings, notification::NotificationLevel, App};

#[derive(Debug, Clone)]
pub struct LogLine
{
    pub(super) level: NotificationLevel,
    pub(super) message: String,
}

impl LogLine
{
    pub(super) fn new(level: NotificationLevel, message: String) -> Self
    {
        LogLine
        {
            level,
            message,
        }
    }

    pub(super) fn to_line(&self, color_settings: &ColorSettings) -> Line<'static>
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
        line.spans.push(Span::styled(format!("{}", self.level), style));
        line.spans.push(Span::styled(" ", color_settings.log_message));
        line.spans.push(Span::styled(format!("{}", self.message), color_settings.log_message));
        line.left_aligned()
    }
}

impl App
{
    pub(super) fn log(&mut self, level: NotificationLevel, message: &str)
    {
        self.notificaiton.bump_notification_level(level);
        self.log.push(LogLine::new(level, message.to_string()));
    }
}