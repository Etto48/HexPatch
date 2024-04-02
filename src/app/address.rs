use ratatui::text::{Line, Span, Text};

use super::{color_settings::ColorSettings, App};

impl <'a> App<'a>
{
    pub(super) fn addresses(color_settings: &ColorSettings, size: usize, block_size: usize, blocks_per_row: usize) -> Text<'a>
    {
        let mut result = Text::default();

        for i in 0..=size/(block_size * blocks_per_row)
        {
            let mut line = Line::default();
            line.spans.push(Span::styled(format!("{:16X}", i * block_size * blocks_per_row), 
            if i == 0 { 
                color_settings.address_selected 
            } else { 
                color_settings.address_default 
            }));
            result.lines.push(line);
        }
        result
    }

    pub(super) fn update_address_cursor(&mut self)
    {
        if self.address_last_row < self.address_view.lines.len()
        {
            self.address_view.lines[self.address_last_row].spans[0].style = self.color_settings.address_default;
        }
        let current_row = self.cursor.1 as usize + self.scroll;
        if current_row < self.address_view.lines.len()
        {
            self.address_last_row = current_row;
            self.address_view.lines[self.address_last_row].spans[0].style = self.color_settings.address_selected;
        }
    }
}