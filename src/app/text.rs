use ratatui::text::{Line, Span, Text};

use super::{color_settings::ColorSettings, notification::NotificationLevel, App};

impl <'a> App<'a>
{
    pub(super) fn bytes_to_styled_text(color_settings: &ColorSettings, bytes: &'_[u8], block_size: usize, blocks_per_row: usize) -> Text<'a>
    {
        let mut ret = Text::default();
        let mut current_line = Line::default();
        let mut local_block = 0;
        let mut local_byte = 0;
        let mut byte_index = 0;
        for b in bytes
        {
            let style = if byte_index == 0
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

    pub(super) fn insert_text(&mut self, text: &str)
    {
        self.patch_bytes(text.as_bytes(), false);
    }

    pub(super) fn update_text_cursor(&mut self)
    {
        let cursor_position = self.get_cursor_position();
        let current_line = cursor_position.line_index;
        let current_byte = cursor_position.line_byte_index;
        let current_text_span = current_byte * 2;

        if self.text_last_byte_index < self.data.len()
        {
            let old_byte = self.data[self.text_last_byte_index];
            let style = Self::get_style_for_byte(&self.color_settings, old_byte);
            self.text_view.lines[self.text_cursor.0].spans[self.text_cursor.1].style = style;
        }

        self.text_last_byte_index = cursor_position.global_byte_index;
        self.text_cursor = (current_line, current_text_span);
        if self.text_cursor.0 < self.text_view.lines.len() && self.text_cursor.1 < self.text_view.lines[self.text_cursor.0].spans.len()
        {
            self.text_view.lines[self.text_cursor.0].spans[self.text_cursor.1].style = self.color_settings.text_selected;
        }
    }

    fn found_text_here(&self, starting_from: usize, text: &str) -> bool
    {
        for (i,byte) in text.bytes().enumerate()
        {
            if self.data.len() <= starting_from + i
            {
                return false;
            }
            else if self.data[starting_from + i] != byte
            {
                return false;
            }
        }
        true
    }

    pub(super) fn find_text(&mut self, text: &str)
    {
        if text.len() == 0 || self.data.len() == 0
        {
            return;
        }
        let already_searched = self.text_last_searched_string == text;
        if !already_searched
        {
            self.text_last_searched_string = text.to_string();
        }
        let mut search_here = self.get_cursor_position().global_byte_index;
        // find the next occurence of the text
        if already_searched && Self::found_text_here(&self, search_here, text)
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
            if Self::found_text_here(&self, actual_search_here, text)
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