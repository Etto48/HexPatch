use ratatui::text::{Line, Span, Text};

use super::{assembly::AssemblyLine, color_settings::ColorSettings, info_mode::InfoMode, notification::NotificationLevel, App};

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
        let blocks_per_row = Self::calc_blocks_per_row(self.block_size, width);
        if self.blocks_per_row != blocks_per_row
        {
            self.resize(blocks_per_row);
        }
    }

    pub(super) fn resize(&mut self, blocks_per_row: usize)
    {
        let old_cursor = self.get_cursor_position();
        self.blocks_per_row = blocks_per_row;
        self.address_view = Self::addresses(&&self.color_settings, self.data.len(), self.block_size, self.blocks_per_row);
        self.hex_view = Self::bytes_to_styled_hex(&self.color_settings, &self.data, self.block_size, self.blocks_per_row);
        self.text_view = Self::bytes_to_styled_text(&self.color_settings, &self.data, self.block_size, self.blocks_per_row);
        self.hex_cursor = (0, 0);
        self.hex_last_byte_index = 0;
        self.text_cursor = (0, 0);
        self.text_last_byte_index = 0;
        self.address_last_row = 0;

        self.jump_to(old_cursor.global_byte_index, false);
    }

    pub(super) fn calc_blocks_per_row(block_size: usize, width: u16) -> usize
    {
        let block_characters_hex = block_size * 3 + 1;
        let block_characters_text = block_size * 2 + 1;
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

    pub(super) fn edit_data(&mut self, mut value: char)
    {
        value = value.to_uppercase().next().unwrap(); 

        if value >= '0' && value <= '9' || value >= 'A' && value <= 'F'
        {   
            let cursor_position = self.get_cursor_position();
            let old_instruction = self.get_current_instruction().clone();
            self.color_instruction_bytes(&old_instruction, true);
            
            let hex = if cursor_position.high_byte
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
            let [high_char, low_char] = Self::u8_to_hex(byte);

            self.hex_view.lines[cursor_position.line_index]
                .spans[cursor_position.line_byte_index * 3] = Span::styled(high_char.to_string(), style);
            self.hex_view.lines[cursor_position.line_index]
                .spans[cursor_position.line_byte_index * 3 + 1] = Span::styled(low_char.to_string(), style);
            
            
            let text = App::u8_to_char(byte);
            let new_str = text.to_string();

            self.text_view.lines[cursor_position.line_index]
                .spans[cursor_position.line_byte_index * 2] = Span::styled(new_str, style);
        }
        self.edit_assembly();
        self.update_hex_cursor();
        self.update_text_cursor();
    }

    pub(super) fn update_hex_cursor(&mut self)
    {
        let cursor_position = self.get_cursor_position();
        let instruction = self.get_current_instruction().clone();
        if self.hex_last_byte_index < self.data.len()
        {
            let old_byte = self.data[self.hex_last_byte_index];
            let old_instruction = self.get_instruction_at(self.hex_last_byte_index).clone();
            let style = Self::get_style_for_byte(&self.color_settings, old_byte);
            self.hex_view.lines[self.hex_cursor.0].spans[self.hex_cursor.1].style = style;
            self.color_instruction_bytes(&old_instruction, true);
        }

        if self.info_mode == InfoMode::Assembly
        {
            self.color_instruction_bytes(&instruction, false);
        }

        self.hex_last_byte_index = cursor_position.global_byte_index;
        self.hex_cursor = (cursor_position.line_index, cursor_position.line_byte_index * 3 + cursor_position.get_high_byte_offset());
        if self.hex_cursor.0 < self.hex_view.lines.len() && self.hex_cursor.1 < self.hex_view.lines[self.hex_cursor.0].spans.len()
        {
            self.hex_view.lines[self.hex_cursor.0].spans[self.hex_cursor.1].style = self.color_settings.hex_selected;
        }
    }

    pub(super) fn color_instruction_bytes(&mut self, instruction: &AssemblyLine, original_color: bool)
    {
        if let AssemblyLine::Instruction(instruction) = instruction
        {
            for i in instruction.file_address as usize..instruction.file_address as usize + instruction.instruction.len() {
                let gui_pos = self.get_expected_cursor_position(i, true);
                let style = if original_color
                {
                    Self::get_style_for_byte(&self.color_settings, self.data[i])
                }
                else 
                {
                    self.color_settings.hex_current_instruction   
                };
                self.hex_view.lines[gui_pos.line_index].spans[gui_pos.line_byte_index*3].style = style;
                self.hex_view.lines[gui_pos.line_index].spans[gui_pos.line_byte_index*3+1].style = style;
                if i != instruction.file_address as usize + instruction.instruction.len()-1 {
                    self.hex_view.lines[gui_pos.line_index].spans[gui_pos.line_byte_index*3+2].style = style;
                }
            }
        }
    }

    pub(super) fn save_data(&mut self) -> Result<(), std::io::Error>
    {
        std::fs::write(&self.path, &self.data)?;
        self.dirty = false;
        self.log(NotificationLevel::Info, &format!("Saved to {}", self.path.to_string_lossy()));
        Ok(())
    }
}