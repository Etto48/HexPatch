use iced_x86::Instruction;
use ratatui::text::{Line, Span, Text};

use crate::asm::assembler::assemble;

use super::{app::App, color_settings::ColorSettings, header::Header};

impl <'a> App<'a>
{
    fn instruction_to_line(color_settings: &ColorSettings, instruction: &Instruction, selected: bool) -> Line<'a>
    {
        let mut line = Line::default();
        line.spans.push(Span::styled(format!("{:16X}",instruction.ip()), 
            if selected
            {
                color_settings.assembly_selected
            }
            else 
            {    
                color_settings.assembly_address
            }
        ));
        line.spans.push(Span::raw(" "));
        let instruction_string = instruction.to_string();
        let mut instruction_pieces = instruction_string.split_whitespace();
        let mnemonic = instruction_pieces.next().unwrap().to_string();
        let args = instruction_pieces.collect::<Vec<&str>>().join(" ");
        let mnemonic_style = 
        match instruction.mnemonic() {
            iced_x86::Mnemonic::Nop => color_settings.assembly_nop,
            iced_x86::Mnemonic::INVALID => color_settings.assembly_bad,
            _ => color_settings.assembly_default,
        };

        line.spans.push(Span::styled(mnemonic, mnemonic_style));
        line.spans.push(Span::raw(" "));
        line.spans.push(Span::raw(args));
        line
    }

    pub(super) fn assembly_from_bytes(color_settings: &ColorSettings, bytes: &[u8], header: &Option<Header>) -> (Text<'a>, Vec<usize>, Vec<Instruction>)
    {
        let mut output = Text::default();
        let mut line_offsets = vec![0; bytes.len()];
        let mut instructions = Vec::new();

        let bitness = match header
        {
            Some(header) => header.bitness(),
            None => 64,
        };

        let decoder = iced_x86::Decoder::new(bitness, bytes, iced_x86::DecoderOptions::NONE);
        let mut byte_index = 0;
        let mut line_index = 0;
        for instruction in decoder {
            instructions.push(instruction);
            let line = Self::instruction_to_line(color_settings, &instruction, line_index == 0);
            
            for _ in 0..instruction.len() {
                line_offsets[byte_index] = line_index;
                byte_index += 1;
            }
            line_index += 1;
            output.lines.push(line);
        }
        (output, line_offsets, instructions)
    }

    pub(super) fn bytes_from_assembly(&self, assembly: &str) -> Result<Vec<u8>, String>
    {        
        let bytes = assemble(assembly, 64);
        match bytes
        {
            Ok(bytes) => Ok(bytes),
            Err(e) => 
            {
                Err(format!("{}", e.to_string()))
            },
        }
    }

    pub(super) fn patch_bytes(&mut self, bytes: &[u8])
    {
        let current_instruction = self.get_current_instruction();
        let current_ip = current_instruction.ip();
        for (i, byte) in bytes.iter().enumerate()
        {
            self.data[current_ip as usize + i] = *byte;
        }
        self.color_instruction_bytes(&current_instruction, true);
        for (i, byte) in bytes.iter().enumerate()
        {
            let style = Self::get_style_for_byte(&self.color_settings, *byte);
            let cursor_position = self.get_expected_cursor_position(current_ip as usize + i, true);
            let [high_byte, low_byte] = Self::u8_to_hex(*byte);

            self.hex_view.lines[cursor_position.line_index].spans[cursor_position.line_byte_index * 3].content = high_byte.to_string().into();
            self.hex_view.lines[cursor_position.line_index].spans[cursor_position.line_byte_index * 3].style = style;
            self.hex_view.lines[cursor_position.line_index].spans[cursor_position.line_byte_index * 3 + 1].content = low_byte.to_string().into();
            self.hex_view.lines[cursor_position.line_index].spans[cursor_position.line_byte_index * 3 + 1].style = style;
            
            self.text_view.lines[cursor_position.line_index].spans[cursor_position.line_byte_index * 2].content = Self::u8_to_char(*byte).to_string().into();
            self.text_view.lines[cursor_position.line_index].spans[cursor_position.line_byte_index * 2].style = style;
        }
        self.dirty = true;
        self.edit_assembly();
        self.update_cursors();
    }

    pub(super) fn patch(&mut self, assembly: &str)
    {
        let bytes = self.bytes_from_assembly(assembly);
        match bytes
        {
            Ok(bytes) => self.patch_bytes(&bytes),
            Err(e) => {
                self.log("Error", &e);
            }
        }
    }

    pub(super) fn update_assembly_scroll(&mut self)
    {
        let cursor_position = self.get_cursor_position();
        let current_ip = cursor_position.global_byte_index.min(self.assembly_offsets.len() - 1);
        let current_scroll = self.assembly_offsets[current_ip];
        
        self.assembly_view.lines[self.assembly_scroll].spans[0].style = self.color_settings.assembly_address;
        self.assembly_view.lines[current_scroll].spans[0].style = self.color_settings.assembly_selected;
        self.assembly_scroll = current_scroll;
    }

    pub(super) fn get_assembly_view_scroll(&self) -> usize
    {
        let visible_lines = self.screen_size.1 - 3;
        let center_of_view = visible_lines / 2;
        let view_scroll = (self.assembly_scroll as isize - center_of_view as isize).clamp(0, (self.assembly_view.lines.len() as isize - visible_lines as isize).max(0));
        
        return view_scroll as usize;
    }

    pub(super) fn get_current_instruction(&self) -> Instruction
    {
        let current_istruction_index =  self.assembly_offsets[self.get_cursor_position().global_byte_index];
        self.assembly_instructions[current_istruction_index as usize]
    }

    pub(super) fn get_instruction_at(&self, index: usize) -> Instruction
    {
        let current_istruction_index =  self.assembly_offsets[index];
        self.assembly_instructions[current_istruction_index as usize]
    }

    pub(super) fn edit_assembly(&mut self)
    {
        let from_byte = self.get_current_instruction().ip() as usize;
        let mut decoder = iced_x86::Decoder::new(64, &self.data[from_byte..], iced_x86::DecoderOptions::NONE);
        decoder.set_ip(from_byte as u64);
        let mut offsets = Vec::new();
        let mut instructions = Vec::new();
        let mut instruction_lines = Vec::new();
        let mut to_byte = self.data.len();

        let from_instruction = self.assembly_offsets[from_byte];

        for instruction in decoder
        {   
            let old_instruction = self.get_instruction_at(instruction.ip() as usize);
            if old_instruction == instruction && old_instruction.ip() == instruction.ip()
            {
                to_byte = old_instruction.ip() as usize;
                break;
            }
            instructions.push(instruction);
            instruction_lines.push(Self::instruction_to_line(&self.color_settings, &instruction, false));
            for _ in 0..instruction.len()
            {
                offsets.push(from_instruction + instructions.len() - 1);
            }
        }
        if from_byte == to_byte
        {
            return;
        }

        let to_instruction = self.assembly_offsets[to_byte];

        let mut original_instruction_count = 1;
        let mut original_instruction_ip = self.assembly_offsets[from_byte];
        for i in from_byte..to_byte
        {
            if self.assembly_offsets[i] != original_instruction_ip
            {
                original_instruction_count += 1;
                original_instruction_ip = self.assembly_offsets[i];
            }
        }

        let new_instruction_count = instructions.len();

        let delta = new_instruction_count as isize - original_instruction_count as isize;

        self.assembly_offsets.splice(from_byte..to_byte, offsets);
        for offset in self.assembly_offsets.iter_mut().skip(to_byte)
        {
            *offset = (*offset as isize + delta) as usize;
        }

        for i in from_instruction..to_instruction
        {
            self.log("Debug", &format!("Removing instruction \"{}\" at {}", self.assembly_instructions[i], self.assembly_instructions[i].ip()));
        }
        for i in 0..instructions.len()
        {
            self.log("Debug", &format!("Adding instruction \"{}\" at {}", instructions[i], instructions[i].ip()));
        }

        self.assembly_instructions.splice(from_instruction..to_instruction, instructions);
        self.assembly_view.lines.splice(from_instruction..to_instruction, instruction_lines);

        self.update_assembly_scroll();
    }
}