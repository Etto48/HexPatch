use ratatui::text::{Line, Span, Text};

use super::{log::NotificationLevel, settings::color_settings::ColorSettings, App};

impl App
{
    pub(super) fn bytes_to_styled_text(color_settings: &ColorSettings, bytes: &'_[u8], block_size: usize, blocks_per_row: usize, selected_byte_offset: usize) -> Text<'static>
    {
        let mut ret = Text::default();
        ret.lines.reserve(bytes.len() / (block_size * blocks_per_row) + 1);
        let mut current_line = Line::default();
        let mut local_block = 0;
        let mut local_byte = 0;
        for (byte_index, b) in bytes.iter().enumerate()
        {
            let style = if byte_index == selected_byte_offset
            {
                color_settings.text_selected
            }
            else
            {
                Self::get_style_for_byte(color_settings, *b)
            };
            let mut next_line = false;
            let char = Self::u8_to_char(*b);
            let char_string = char.to_string();
            let span = Span::styled(char_string, style);
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

            let span = Span::raw(spacing_string);
            current_line.spans.push(span);

            if next_line
            {
                let new_line = std::mem::take(&mut current_line);
                ret.lines.push(new_line);
            }
        }
        if !current_line.spans.is_empty()
        {
            ret.lines.push(current_line);
        }

        ret
    }

    pub(super) fn insert_text(&mut self, text: &str)
    {
        self.patch_bytes(text.as_bytes(), false);
    }

    fn found_text_here(&self, starting_from: usize, text: &str) -> bool
    {
        for (i,byte) in text.bytes().enumerate()
        {
            if self.data.len() <= starting_from + i || 
                self.data[starting_from + i] != byte
            {
                return false;
            }
        }
        true
    }

    pub(super) fn get_text_view(&self, start_row: usize, end_row: usize) -> Text<'static>
    {
        let start_byte = start_row * self.blocks_per_row * self.block_size;
        let end_byte = end_row * self.blocks_per_row * self.block_size;
        let end_byte = std::cmp::min(end_byte, self.data.len());
        let bytes = &self.data[start_byte..end_byte];
        let selected_byte_offset = self.get_cursor_position().global_byte_index.saturating_sub(start_byte);
        Self::bytes_to_styled_text(&self.settings.color, bytes, self.block_size, self.blocks_per_row, selected_byte_offset)
    }

    pub(super) fn find_text(&mut self, text: &str)
    {
        if text.is_empty() || self.data.is_empty()
        {
            return;
        }
        let already_searched = self.text_last_searched_string == text;
        if !already_searched
        {
            self.text_last_searched_string = text.to_string();
        }
        let mut search_here = self.get_cursor_position().global_byte_index;
        // find the next occurrence of the text
        if already_searched && Self::found_text_here(self, search_here, text)
        {
            search_here += text.len();
        }
        else
        {
            search_here = 0;
        }
        let max_search_index = self.data.len() + search_here;
        while search_here < max_search_index
        {
            let actual_search_here = search_here % self.data.len();
            if Self::found_text_here(self, actual_search_here, text)
            {
                self.jump_to(actual_search_here, false);
                return;
            }
            search_here += 1;
        }
        self.log(NotificationLevel::Warning, "Text not found");
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
}