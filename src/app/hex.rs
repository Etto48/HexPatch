use ratatui::text::{Line, Span, Text};

use super::{assembly::AssemblyLine, color_settings::ColorSettings, info_mode::InfoMode, notification::NotificationLevel, App};

pub(super) struct InstructionInfo
{
    pub offset: isize,
    pub length: usize,
}

impl App
{
    pub(super) fn bytes_to_styled_hex(
        color_settings: &ColorSettings, 
        bytes: &[u8], 
        block_size: usize, 
        blocks_per_row: usize, 
        selected_byte_index: usize, 
        high_byte: bool, 
        instruction_info: Option<InstructionInfo>
    ) -> Text<'static>
    {
        let mut ret = Text::default();
        ret.lines.reserve(bytes.len() / (block_size * blocks_per_row) + 1);
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
            let (mut space_style,mut style) = (color_settings.hex_default, Self::get_style_for_byte(color_settings, *b));

            if let Some(instruction_info) = &instruction_info
            {
                if byte_index >= instruction_info.offset && byte_index < instruction_info.offset + instruction_info.length as isize
                {
                    let is_last_space = byte_index == instruction_info.offset + instruction_info.length as isize - 1;
                    if !is_last_space
                    {
                        space_style = color_settings.hex_current_instruction;
                    }
                    style = color_settings.hex_current_instruction;
                }
            }

            let span = Span::styled(hex_high, 
            if byte_index == selected_byte_index as isize && high_byte
            {
                color_settings.hex_selected
            }
            else
            {
                style
            });
            current_line.spans.push(span);
            let span = Span::styled(hex_low, if byte_index == selected_byte_index as isize && !high_byte
            {
                color_settings.hex_selected
            }
            else
            {
                style
            });
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

            let span = Span::styled(spacing_string, space_style);
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

        self.jump_to(old_cursor.global_byte_index, false);
    }

    pub(super) fn calc_blocks_per_row(block_size: usize, width: u16) -> usize
    {
        let block_characters_hex = block_size * 3 + 1;
        let block_characters_text = block_size * 2 + 1;
        let available_width = width.saturating_sub(18 + 2 + 2);
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

            let old_byte = self.data[cursor_position.global_byte_index];
            let old_byte_str = format!("{:02X}", old_byte);
            let new_byte_str = if cursor_position.high_byte
            {
                format!("{}{}", value, old_byte_str.chars().nth(1).unwrap())
            }
            else
            {
                format!("{}{}", old_byte_str.chars().nth(0).unwrap(), value)
            };
            let new_byte = u8::from_str_radix(&new_byte_str, 16).unwrap();

            self.data[cursor_position.global_byte_index] = new_byte;

            if old_byte != new_byte
            {
                self.dirty = true;
            }
        }
        self.edit_assembly(1);
    }

    /// start_row is included, end_row is excluded
    pub(super) fn get_hex_view(&self, start_row: usize, end_row: usize) -> Text<'static>
    {
        let start_byte = start_row * self.blocks_per_row * self.block_size;
        let end_byte = end_row * self.blocks_per_row * self.block_size;
        let end_byte = std::cmp::min(end_byte, self.data.len());
        let bytes = &self.data[start_byte..end_byte];
        let selected_byte_index = self.get_cursor_position().global_byte_index.saturating_sub(start_byte);
        let high_byte = self.get_cursor_position().high_byte;
        let instruction_info = 
        {
            if self.info_mode == InfoMode::Assembly
            {
                let current_instruction = self.get_current_instruction();
                let instruction_info = if let Some(instruction) = current_instruction
                {
                    if let AssemblyLine::Instruction(instruction) = instruction
                    {
                        let offset = instruction.file_address as isize - start_byte as isize;
                        let length = instruction.instruction.len();
                        Some(InstructionInfo { offset: offset, length })
                    }
                    else
                    {
                        None
                    }
                }
                else
                {
                    None
                };
                instruction_info
            }
            else
            {
                None
            }
        };
        Self::bytes_to_styled_hex(&self.color_settings, bytes, self.block_size, self.blocks_per_row, selected_byte_index, high_byte, instruction_info)
    }

    pub(super) fn save_data(&mut self) -> Result<(), std::io::Error>
    {
        std::fs::write(&self.path, &self.data)?;
        self.dirty = false;
        self.log(NotificationLevel::Info, &format!("Saved to {}", self.path.to_string_lossy()));
        Ok(())
    }
}