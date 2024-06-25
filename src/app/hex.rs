use std::error::Error;

use ratatui::text::{Line, Span, Text};

use crate::get_app_context;

use super::{asm::assembly_line::AssemblyLine, info_mode::InfoMode, settings::color_settings::ColorSettings, App};

pub(super) struct InstructionInfo
{
    pub offset: isize,
    pub length: usize,
    pub is_section: bool,
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
        for (byte_index, b) in bytes.iter().enumerate()
        {
            let byte_index = byte_index as isize;
            let mut next_line = false;
            let hex_chars = Self::u8_to_hex(*b);
            let hex_high = hex_chars[0].to_string();
            let hex_low = hex_chars[1].to_string();
            let (mut space_style,mut style) = (color_settings.hex_default, Self::get_style_for_byte(color_settings, *b));

            if let Some(instruction_info) = &instruction_info
            {
                let used_style = if instruction_info.is_section
                {
                    color_settings.hex_current_section
                }
                else
                {
                    color_settings.hex_current_instruction
                };
                if byte_index >= instruction_info.offset && byte_index < instruction_info.offset + instruction_info.length as isize
                {
                    let is_last_space = byte_index == instruction_info.offset + instruction_info.length as isize - 1;
                    if !is_last_space
                    {
                        space_style = used_style;
                    }
                    style = used_style;
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

    pub(super) fn resize_to_size(&mut self, width: u16, height: u16)
    {
        let blocks_per_row = Self::calc_blocks_per_row(self.block_size, width);
        if (width, height) != self.screen_size
        {
            self.screen_size = (width, height);
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
        (blocks_per_row as usize).max(1)
    }

    pub(super) fn u8_to_hex(input: u8) -> [char; 2]
    {
        let symbols = ['0','1','2','3','4','5','6','7','8','9','A','B','C','D','E','F'];
        let low = input & 0x0f;
        let high = (input & 0xf0) >> 4;
        [symbols[high as usize], symbols[low as usize]]
    }

    pub(super) fn edit_data(&mut self, mut value: char) -> Result<(), Box<dyn Error>>
    {
        value = value.to_uppercase().next().unwrap(); 

        if value.is_ascii_hexdigit()
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

            let mut new_bytes = vec![new_byte];
            let mut app_context = get_app_context!(self);

            self.plugin_manager.on_edit(
                &mut new_bytes,
                &mut app_context
            );
            new_bytes.truncate(self.data.len().checked_sub(cursor_position.global_byte_index).unwrap());

            self.data[cursor_position.global_byte_index..cursor_position.global_byte_index + new_bytes.len()].copy_from_slice(&new_bytes);

            if old_byte != new_byte
            {
                self.dirty = true;
            }
            self.edit_assembly(new_bytes.len());
        }
        Ok(())
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
                if let Some(assembly_line) = current_instruction
                {
                    let offset = assembly_line.file_address() as isize - start_byte as isize;
                    let length = assembly_line.len();
                    let is_section = matches!(assembly_line, AssemblyLine::SectionTag(_));
                    Some(InstructionInfo { offset, length, is_section})
                }
                else
                {
                    None
                }
            }
            else
            {
                None
            }
        };
        Self::bytes_to_styled_hex(&self.settings.color, bytes, self.block_size, self.blocks_per_row, selected_byte_index, high_byte, instruction_info)
    }
}

#[cfg(test)]
mod test
{
    use super::*;

    #[test]
    fn resize()
    {
        let data = vec![0; 0x100];
        let mut app = App::mockup(data);
        app.resize_to_size(80, 24);
        app.resize_to_size(80, 50);
        app.resize_to_size(40, 24);
        app.resize_to_size(250, 250);

        app.resize_to_size(80, 1);
        app.resize_to_size(250, 1);
        app.resize_to_size(40, 1);
        app.resize_to_size(1, 1);

        app.resize_to_size(1, 50);
        app.resize_to_size(1, 250);
        app.resize_to_size(1, 24);
        app.resize_to_size(1, 1);
        app.resize_to_size(80, 24);
    }
}