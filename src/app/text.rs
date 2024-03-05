use ratatui::text::{Line, Span, Text};

use super::{color_settings::ColorSettings, App};

impl <'a> App<'a>
{
    pub(super) fn bytes_to_styled_text(color_settings: &ColorSettings, bytes: &'_[u8], block_size: usize, blocks_per_row: usize) -> Text<'a>
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

            let style = Self::get_style_for_byte(color_settings, *b);
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