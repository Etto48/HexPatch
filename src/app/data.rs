use ratatui::{style::{Color, Modifier, Style}, text::{Line, Span, Text}};

use super::App;

impl <'a> App<'a>
{
    pub(super) fn get_style_for_byte(byte: u8) -> Style
    {
        match byte
        {
            // null
            0x00 => Style::default().fg(Color::DarkGray),
            // newline
            0x0A | 0x0C | 0x0D => Style::default().fg(Color::LightRed),
            // whitespace
            0x20 | 0x09 | 0x0B => Style::default().fg(Color::Rgb(244, 202, 183)),
            // numbers
            0x30..=0x39 => Style::default().fg(Color::Rgb(204, 152, 113)),
            // uppercase
            0x41..=0x5A => Style::default().fg(Color::Rgb(204, 152, 113)),
            // lowercase
            0x61..=0x7A => Style::default().fg(Color::Rgb(204, 152, 113)),
            // special characters
            0x20..=0x7E => Style::default().fg(Color::Rgb(204, 152, 113)).add_modifier(Modifier::DIM),
            _ => Style::default()
        }
    }

    pub(super) fn bytes_to_styled_hex(bytes: &[u8], block_size: usize, blocks_per_row: usize) -> Text<'a>
    {
        let mut ret = Text::default();
        let mut current_line = Line::default();
        let mut local_block = 0;
        let mut local_byte = 0;
        for b in bytes
        {
            let mut next_line = false;
            let hex_chars = Self::u8_to_hex(*b);
            let mut hex_string = hex_chars.iter().collect::<String>();
            hex_string.push(' ');
            local_byte += 1;
            if local_byte % block_size == 0
            {
                local_byte = 0;
                hex_string.push(' ');

                local_block += 1;
                if local_block % blocks_per_row == 0
                {
                    local_block = 0;
                    next_line = true;
                }
            }

            let style = Self::get_style_for_byte(*b);
            let span = Span::styled(hex_string, style);
            current_line.spans.push(span);

            if next_line
            {
                let new_line = std::mem::replace(&mut current_line, Line::default());
                ret.lines.push(new_line);
            }
        }
        if current_line.spans.len() > 0
        {
            ret.lines.push(current_line);
        }

        ret
    }

    pub(super) fn bytes_to_styled_text(bytes: &'_[u8], block_size: usize, blocks_per_row: usize) -> Text<'a>
    {
        let mut ret = Text::default();
        let mut current_line = Line::default();
        let mut local_block = 0;
        let mut local_byte = 0;
        for b in bytes
        {
            let mut next_line = false;
            let char = Self::u8_to_char(*b);
            let mut char_string = char.to_string();
            char_string.push(' ');
            local_byte += 1;
            if local_byte % block_size == 0
            {
                local_byte = 0;
                char_string.push(' ');

                local_block += 1;
                if local_block % blocks_per_row == 0
                {
                    local_block = 0;
                    next_line = true;
                }
            }

            let style = Self::get_style_for_byte(*b);
            let span = Span::styled(char_string, style);
            current_line.spans.push(span);

            if next_line
            {
                let new_line = std::mem::replace(&mut current_line, Line::default());
                ret.lines.push(new_line);
            }
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
        self.hex_view = Self::bytes_to_styled_hex(&self.data, self.block_size, self.blocks_per_row);
        self.text_view = Self::bytes_to_styled_text(&self.data, self.block_size, self.blocks_per_row);
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

    pub(super) fn u8_to_char(input: u8) -> char
    {
        match input
        {
            0x20..=0x7E => input as char,
            0x0A => '⏎',
            0x0C => '↡',
            0x0D => '↵',
            0x08 => '⇤',
            0x09 => '⇥',
            0x1B => '␛',
            0x7F => '␡',
            _ => '.'
        }
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

            let mut old_str = self.hex_view.lines[cursor_position.line_index as usize]
                .spans[cursor_position.line_byte_index as usize].content.to_string();

            if old_str.as_bytes()[(cursor_position.local_x % 3) as usize] != value as u8
            {
                self.dirty = true;
            }

            unsafe {
                old_str.as_bytes_mut()[(cursor_position.local_x % 3) as usize] = value as u8;
            }
            
            let hex = old_str.chars().filter(|c| c.is_whitespace() == false).collect::<String>();

            let byte = u8::from_str_radix(&hex, 16).unwrap();

            self.data[cursor_position.global_byte_index as usize] = byte;

            let style = Self::get_style_for_byte(byte);
            self.hex_view.lines[cursor_position.line_index as usize]
                .spans[cursor_position.line_byte_index as usize] = Span::styled(old_str, style);
            
            let text = App::u8_to_char(byte);
            let old_str = self.text_view.lines[cursor_position.line_index as usize]
                .spans[cursor_position.line_byte_index as usize].content.to_string();
            let text_iterator = old_str.chars().filter(|c| c.is_whitespace());
            let mut new_str = text.to_string();
            new_str.extend(text_iterator);

            self.text_view.lines[cursor_position.line_index as usize]
                .spans[cursor_position.line_byte_index as usize] = Span::styled(new_str, style);
        }
        self.edit_assembly();
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