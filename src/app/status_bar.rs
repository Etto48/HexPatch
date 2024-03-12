use ratatui::text::{Line, Span, Text};

use super::App;

impl <'a> App<'a>
{
    pub(super) fn build_status_bar(&self) -> Text<'a>
    {
        let mut status_bar = Text::default();
        status_bar.style = self.color_settings.status_bar;
        let mut line = Line::default();
        line.style = self.color_settings.status_bar;
        let max_len = self.screen_size.0 as usize;
        let current_position = self.get_cursor_position();

        line.spans.push(Span::styled(" ", self.color_settings.status_bar));

        let (notification_string, notification_style) = match self.notificaiton
        {
            super::notification::NotificationLevel::None => (" ", self.color_settings.status_bar),
            super::notification::NotificationLevel::Debug => ("●", self.color_settings.status_debug),
            super::notification::NotificationLevel::Info => ("●", self.color_settings.status_info),
            super::notification::NotificationLevel::Warning => ("●", self.color_settings.status_warning),
            super::notification::NotificationLevel::Error => ("●", self.color_settings.status_error),
        };        
        line.spans.push(Span::styled(notification_string, notification_style));
        line.spans.push(Span::styled(" ", self.color_settings.status_bar));
        if self.notificaiton != super::notification::NotificationLevel::None
        {
            line.spans.push(Span::styled(self.log[self.log.len() - 1].message.clone(), self.color_settings.status_bar));
        }

        let current_location_span = Span::styled(format!("{:16X} {} ",current_position.global_byte_index, 
        if current_position.high_byte
        {
            "H"
        }
        else
        {
            "L"
        }), self.color_settings.status_bar);
        let padding_spaces_string = " ".repeat(max_len - line.width() - current_location_span.width() - 2);

        line.spans.push(Span::raw(padding_spaces_string));
        line.spans.push(current_location_span);
        status_bar.lines.push(line);
        status_bar
    }
}