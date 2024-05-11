use ratatui::text::{Line, Span, Text};

use super::App;

impl App
{
    pub(super) fn build_status_bar(&self) -> Text<'static>
    {
        let mut status_bar = Text::default();
        status_bar.style = self.settings.color.status_bar;
        let mut line = Line {
            style: self.settings.color.status_bar,
            ..Default::default()
        };
        let max_len = self.screen_size.0 as usize;
        let current_position = self.get_cursor_position();

        line.spans.push(Span::styled(" ", self.settings.color.status_bar));

        let (notification_str, notification_style) = match self.notification
        {
            super::notification::NotificationLevel::None => (" ", self.settings.color.status_bar),
            super::notification::NotificationLevel::Debug => ("●", self.settings.color.status_debug),
            super::notification::NotificationLevel::Info => ("●", self.settings.color.status_info),
            super::notification::NotificationLevel::Warning => ("●", self.settings.color.status_warning),
            super::notification::NotificationLevel::Error => ("●", self.settings.color.status_error),
        };
        line.spans.push(Span::styled(notification_str, notification_style));
        line.spans.push(Span::styled(" ", self.settings.color.status_bar));
        if self.notification != super::notification::NotificationLevel::None
        {
            line.spans.push(Span::styled(self.log[self.log.len() - 1].message.chars().take(max_len - 25).collect::<String>(), self.settings.color.status_bar));
        }

        let current_location_span = Span::styled(format!("{:16X} {} ",current_position.global_byte_index, 
        if current_position.high_byte
        {
            "H"
        }
        else
        {
            "L"
        }), self.settings.color.status_bar);
        let space_number = max_len as isize - line.width() as isize - current_location_span.width() as isize - 2;
        if space_number < 0
        {
            return Text::default();
        }
        let space_number = space_number as usize;
        let padding_spaces_string = " ".repeat(space_number);

        line.spans.push(Span::raw(padding_spaces_string));
        line.spans.push(current_location_span);
        status_bar.lines.push(line);
        status_bar
    }
}