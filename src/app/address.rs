use ratatui::text::{Line, Span, Text};

use super::App;

impl App
{
    pub(super) fn get_address_view(&self, start_row: usize, end_row: usize) -> Text<'static>
    {
        let mut ret = Text::default();
        ret.lines.reserve(end_row - start_row);
        let selected_row = self.get_cursor_position().line_index;
        for i in start_row..end_row
        {
            let mut line = Line::default();
            line.spans.push(Span::styled(format!("{:16X}", i * self.block_size * self.blocks_per_row), 
            if i == selected_row
            {
                self.color_settings.address_selected
            }
            else
            {
                self.color_settings.address_default
            }));
            ret.lines.push(line);
        }
        ret
    }
}