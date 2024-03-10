use ratatui::text::{Line, Span};

use super::{color_settings::ColorSettings, App};

pub struct LogLine
{
    pub(super) level: String,
    pub(super) message: String,
}

impl LogLine
{
    pub(super) fn new(level: String, message: String) -> Self
    {
        LogLine
        {
            level,
            message,
        }
    }

    pub(super) fn to_line(&self, color_settings: &ColorSettings) -> Line
    {
        let mut line = Line::default();
        let style = match self.level.as_str()
        {
            "Info" => color_settings.log_info,
            "Debug" => color_settings.log_debug,
            "Warning" => color_settings.log_warning,
            "Error" => color_settings.log_error,
            _ => color_settings.log_info,
        };
        line.spans.push(Span::styled(format!("{:7}", self.level), style));
        line.spans.push(Span::styled(" ", color_settings.log_message));
        line.spans.push(Span::styled(format!("{}", self.message), color_settings.log_message));
        line.left_aligned()
    }
}

impl <'a> App<'a>
{
    pub(super) fn log(&mut self, level: &str, message: &str)
    {
        self.log.push(LogLine::new(level.to_string(), message.to_string()));
    }
}