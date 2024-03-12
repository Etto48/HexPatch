use iced_x86::Instruction;
use ratatui::text::{Line, Span};

use crate::asm::assembler::assemble;

use super::{app::App, color_settings::ColorSettings, header::{Header, Section}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotExecutableSection
{
    pub name: String,
    pub ip: u64,
    pub size: usize
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssemblyLine
{
    Instruction(Instruction),
    NotExecutableSection(NotExecutableSection)
}

impl AssemblyLine
{
    pub fn ip(&self) -> u64
    {
        match self
        {
            AssemblyLine::Instruction(instruction) => instruction.ip(),
            AssemblyLine::NotExecutableSection(section) => section.ip,
        }
    }

    pub fn to_line(&self, color_settings: &ColorSettings, current_byte_index: usize) -> Line
    {
        match self
        {
            AssemblyLine::Instruction(instruction) => {
                let selected = current_byte_index >= instruction.ip() as usize && current_byte_index < instruction.ip() as usize + instruction.len();
                App::instruction_to_line(color_settings, instruction, selected)
            },
            AssemblyLine::NotExecutableSection(section) => 
            {
                let selected = current_byte_index >= section.ip as usize && current_byte_index < section.ip as usize + section.size;
                let mut line = Line::default();
                let address_style = if selected
                {
                    color_settings.assembly_selected
                }
                else
                {
                    color_settings.assembly_address
                };
                line.spans.push(Span::styled(format!("{:16X}", section.ip), address_style));
                line.spans.push(Span::raw(" "));
                line.spans.push(Span::styled(format!("[{} ({} bytes)]", section.name, section.size), color_settings.assembly_section));
                line
            }
        }
    }
}

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

    pub(super) fn sections_from_bytes(bytes: &[u8], header: &Header) -> (Vec<usize>, Vec<AssemblyLine>)
    {
        let mut line_offsets = vec![0; bytes.len()];
        let mut lines = Vec::new();
        let mut sections = header.get_sections();
        if sections.len() == 0
        {
            sections.push(Section {
                name: ".text".to_string(),
                virtual_address: 0,
                address: 0,
                size: bytes.len() as u64,
            });
        }

        let mut current_byte = 0;
        for section in sections
        {
            while section.address > current_byte as u64
            {
                lines.push(AssemblyLine::NotExecutableSection(
                    NotExecutableSection {
                        name: "Unknown".to_string(),
                        ip: current_byte as u64,
                        size: section.address as usize - current_byte
                    }
                ));
                for _ in 0..section.address as usize - current_byte
                {
                    line_offsets[current_byte] = lines.len() - 1;
                    current_byte += 1;
                }
            }
            if section.size == 0
            {
                continue;
            }
            match section.name.as_str()
            {
                ".text" => {
                    let (offsets, instructions) = Self::assembly_from_section(bytes, header, section.address as usize, section.size as usize, lines.len());
                    line_offsets.splice(section.address as usize..section.address as usize + section.size as usize, offsets);
                    lines.extend(instructions);
                    current_byte += section.size as usize;
                },
                name => {
                    lines.push(AssemblyLine::NotExecutableSection(
                        NotExecutableSection {
                            name: name.to_string(),
                            ip: section.address,
                            size: section.size as usize,
                        }
                    ));
                    for _ in 0..section.size as usize
                    {
                        line_offsets[current_byte] = lines.len() - 1;
                        current_byte += 1;
                    }
                }
            }
        }

        (line_offsets, lines)
    }

    pub(super) fn assembly_from_section(bytes: &[u8], header: &Header, starting_ip: usize, section_size: usize, starting_sections: usize) -> (Vec<usize>, Vec<AssemblyLine>)
    {
        let mut line_offsets = vec![0; section_size];
        let mut instructions = Vec::new();
        let mut current_byte = 0;
        let mut decoder = iced_x86::Decoder::new(header.bitness(), &bytes[starting_ip..starting_ip + section_size], iced_x86::DecoderOptions::NONE);
        decoder.set_ip(starting_ip as u64);
        for instruction in decoder
        {
            instructions.push(AssemblyLine::Instruction(instruction));
            for _ in 0..instruction.len()
            {
                line_offsets[current_byte] = starting_sections + instructions.len() - 1;
                current_byte += 1;
            }
        }
        (line_offsets, instructions)
    }

    pub(super) fn bytes_from_assembly(&self, assembly: &str) -> Result<Vec<u8>, String>
    {        
        let bytes = assemble(assembly, self.header.bitness());
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
        //self.color_instruction_bytes(&current_instruction, true);
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
        
        //self.assembly_view.lines[self.assembly_scroll].spans[0].style = self.color_settings.assembly_address;
        //self.assembly_view.lines[current_scroll].spans[0].style = self.color_settings.assembly_selected;
        self.assembly_scroll = current_scroll;
    }

    pub(super) fn get_assembly_view_scroll(&self) -> usize
    {
        let visible_lines = self.screen_size.1 - 3;
        let center_of_view = visible_lines / 2;
        let view_scroll = (self.assembly_scroll as isize - center_of_view as isize).clamp(0, (self.assembly_instructions.len() as isize - visible_lines as isize).max(0));
        
        return view_scroll as usize;
    }

    pub(super) fn get_current_instruction(&self) -> &AssemblyLine
    {
        let current_istruction_index =  self.assembly_offsets[self.get_cursor_position().global_byte_index];
        &self.assembly_instructions[current_istruction_index as usize]
    }

    pub(super) fn get_instruction_at(&self, index: usize) -> &AssemblyLine
    {
        let current_istruction_index =  self.assembly_offsets[index];
        &self.assembly_instructions[current_istruction_index as usize]
    }

    pub(super) fn edit_assembly(&mut self)
    {
        let from_byte = self.get_current_instruction().ip() as usize;
        let text_section = self.header.get_text_section();
        let (is_inside_text_section, maximum_code_byte) = 
        if let Some(text_section) = text_section 
        {
            (from_byte >= text_section.address as usize && 
                from_byte < text_section.address as usize + text_section.size as usize,
            text_section.address as usize + text_section.size as usize)
        }
        else
        {
            (true, self.data.len())
        };
        if !is_inside_text_section
        {
            return;
        }
        let mut decoder = iced_x86::Decoder::new(self.header.bitness(), &self.data[from_byte..maximum_code_byte], iced_x86::DecoderOptions::NONE);
        decoder.set_ip(from_byte as u64);
        let mut offsets = Vec::new();
        let mut instructions = Vec::new();
        let mut instruction_lines = Vec::new();
        let mut to_byte = self.data.len();

        let from_instruction = self.assembly_offsets[from_byte];

        for instruction in decoder
        {   
            let old_instruction = self.get_instruction_at(instruction.ip() as usize);
            if old_instruction == &AssemblyLine::Instruction(instruction) && old_instruction.ip() == instruction.ip()
            {
                to_byte = old_instruction.ip() as usize;
                break;
            }
            instructions.push(AssemblyLine::Instruction(instruction));
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
            if let AssemblyLine::Instruction(instruction) = &self.assembly_instructions[i]
            {
                self.log("Debug", &format!("Removing instruction \"{}\" at {:X}", instruction, self.assembly_instructions[i].ip()));    
            }
        }
        for i in 0..instructions.len()
        {
            if let AssemblyLine::Instruction(instruction) = &instructions[i]
            {
                self.log("Debug", &format!("Adding instruction \"{}\" at {:X}", instruction, instructions[i].ip()));
            }
        }

        self.assembly_instructions.splice(from_instruction..to_instruction, instructions);

        self.update_assembly_scroll();
    }
}