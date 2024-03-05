use ratatui::{style::{Color, Style}, text::{Line, Span, Text}};

use super::{color_settings::ColorSettings, App};

impl <'a> App<'a>
{
    pub(super) fn bytes_to_styled_hex(color_settings: &ColorSettings, bytes: &[u8], block_size: usize, blocks_per_row: usize) -> Text<'a>
    {
        let mut ret = Text::default();
        let mut current_line = Line::default();
        let mut local_block = 0;
        let mut local_byte = 0;
        let mut byte_index = 0;
        for b in bytes
        {
            let mut next_line = false;
            let hex_chars = Self::u8_to_hex(*b);
            let hex_high = hex_chars[0].to_string();
            let hex_low = hex_chars[1].to_string();
            let style = Self::get_style_for_byte(color_settings, *b);
            let span = Span::styled(hex_high, 
            if byte_index == 0
            {
                color_settings.hex_selected
            }
            else
            {
                style
            });
            current_line.spans.push(span);
            let span = Span::styled(hex_low, style);
            current_line.spans.push(span);
            let mut spacing_string = " ".to_string();
            local_byte += 1;
            if local_byte % block_size == 0
            {
                local_byte = 0;
                spacing_string.push(' ');

                local_block += 1;
                if local_block % blocks_per_row == 0
                {
                    local_block = 0;
                    next_line = true;
                }
            }

            let style = Self::get_style_for_byte(color_settings, *b);
            let span = Span::styled(spacing_string, style);
            current_line.spans.push(span);

            if next_line
            {
                let new_line = std::mem::replace(&mut current_line, Line::default());
                ret.lines.push(new_line);
            }
            byte_index += 1;
        }
        if current_line.spans.len() > 0
        {
            ret.lines.push(current_line);
        }

        ret
    }

    pub(super) fn resize_if_needed(&mut self, width: u16)
    {
        let blocks_per_row = self.calc_blocks_per_row(width);
        if self.blocks_per_row != blocks_per_row
        {
            self.resize(blocks_per_row);
        }
    }

    pub(super) fn resize(&mut self, blocks_per_row: usize)
    {
        self.blocks_per_row = blocks_per_row;
        self.address_view = Self::addresses(self.data.len(), self.block_size, self.blocks_per_row);
        self.hex_view = Self::bytes_to_styled_hex(&self.color_settings, &self.data, self.block_size, self.blocks_per_row);
        self.text_view = Self::bytes_to_styled_text(&self.color_settings, &self.data, self.block_size, self.blocks_per_row);
        // TODO: check if cursor is updated correctly
    }

    pub(super) fn calc_blocks_per_row(&self, width: u16) -> usize
    {
        let block_characters_hex = self.block_size * 3 + 1;
        let block_characters_text = self.block_size * 2 + 1;
        let available_width = width - 18 - 2 - 2;
        let complessive_chars_per_block = block_characters_hex + block_characters_text;
        let blocks_per_row = (available_width + 2) / complessive_chars_per_block as u16;
        blocks_per_row as usize
    }

    pub(super) fn u8_to_hex(input: u8) -> [char; 2]
    {
        let symbols = ['0','1','2','3','4','5','6','7','8','9','A','B','C','D','E','F'];
        let low = input & 0x0f;
        let high = (input & 0xf0) >> 4;
        [symbols[high as usize], symbols[low as usize]]
    }

    pub(super) fn addresses(size: usize, block_size: usize, blocks_per_row: usize) -> Text<'a>
    {
        let mut result = Text::default();

        for i in 0..=size/(block_size * blocks_per_row)
        {
            let mut line = Line::default();
            line.spans.push(Span::styled(format!("{:16X}", i * block_size * blocks_per_row), if i % 2 == 0 {Style::default().fg(Color::DarkGray)} else {Style::default()}));
            result.lines.push(line);
        }
        result
    }

    pub(super) fn edit_data(&mut self, mut value: char)
    {
        value = value.to_uppercase().next().unwrap(); 

        if value >= '0' && value <= '9' || value >= 'A' && value <= 'F'
        {   
            let cursor_position = self.get_cursor_position();
            
            let hex = if cursor_position.local_x % 3 == 0
            {
                format!("{}{}", value, self.hex_view.lines[cursor_position.line_index]
                    .spans[cursor_position.line_byte_index * 3 + 1].content)
            }
            else
            {
                format!("{}{}", self.hex_view.lines[cursor_position.line_index]
                    .spans[cursor_position.line_byte_index * 3].content, value)
            };

            let old_byte = self.data[cursor_position.global_byte_index];

            let byte = u8::from_str_radix(&hex, 16).unwrap();

            self.data[cursor_position.global_byte_index] = byte;

            if old_byte != byte
            {
                self.dirty = true;
            }

            let style = Self::get_style_for_byte(&self.color_settings, byte);
            self.hex_view.lines[cursor_position.line_index]
                .spans[cursor_position.line_byte_index * 3 + cursor_position.local_x % 3] = Span::styled(value.to_string(), style);
            
            let text = App::u8_to_char(byte);
            let new_str = text.to_string();

            self.text_view.lines[cursor_position.line_index]
                .spans[cursor_position.line_byte_index * 2] = Span::styled(new_str, style);
        }
        self.update_hex_cursor();
        self.update_text_cursor();
        self.edit_assembly();
    }

    pub(super) fn update_hex_cursor(&mut self)
    {
        let cursor_position = self.get_cursor_position();
        if self.hex_last_byte_index < self.data.len()
        {
            let old_byte = self.data[self.hex_last_byte_index];
            let style = Self::get_style_for_byte(&self.color_settings, old_byte);    
            self.hex_view.lines[self.hex_cursor.0].spans[self.hex_cursor.1].style = style;
        }

        self.hex_last_byte_index = cursor_position.global_byte_index;
        self.hex_cursor = (cursor_position.line_index, cursor_position.line_byte_index * 3 + cursor_position.local_x % 3);
        if self.hex_cursor.0 < self.hex_view.lines.len() && self.hex_cursor.1 < self.hex_view.lines[self.hex_cursor.0].spans.len()
        {
            self.hex_view.lines[self.hex_cursor.0].spans[self.hex_cursor.1].style = self.color_settings.hex_selected;
        }
    }

    pub(super) fn save_data(&mut self)
    {
        self.output = "Converting data...".to_string();
        self.output = "Saving...".to_string();
        std::fs::write(&self.path, &self.data).unwrap();
        self.dirty = false;
        self.output = format!("Saved to {}", self.path.to_str().unwrap());
    }
}