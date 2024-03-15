use std::{collections::HashMap, rc::Rc};

use iced_x86::{Formatter, Instruction, SymbolResolver};
use ratatui::text::{Line, Span};

use crate::asm::assembler::assemble;

use super::{app::App, color_settings::ColorSettings, notification::NotificationLevel};

use crate::headers::header::{Header, Section};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionTag
{
    pub name: String,
    pub file_address: u64,
    pub virtual_address: u64,
    pub size: usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstructionTag
{
    pub instruction: Instruction,
    pub file_address: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssemblyLine
{
    Instruction(InstructionTag),
    SectionTag(SectionTag)
}

pub struct CustomSymbolResolver
{
    symbol_table: Rc<HashMap<u64, String>>
}

impl CustomSymbolResolver
{
    pub fn new(symbol_table: Rc<HashMap<u64, String>>) -> Self
    {
        Self { symbol_table }
    }
}

impl SymbolResolver for CustomSymbolResolver
{
    fn symbol(
            &mut self, _instruction: &Instruction, _operand: u32, _instruction_operand: Option<u32>, address: u64, _address_size: u32,
        ) -> Option<iced_x86::SymbolResult<'_>> {
        self.symbol_table.get(&address).map(|symbol| iced_x86::SymbolResult::with_string(address, symbol.clone()))
    }
}


impl AssemblyLine
{
    pub fn ip(&self) -> u64
    {
        match self
        {
            AssemblyLine::Instruction(instruction) => instruction.file_address,
            AssemblyLine::SectionTag(section) => section.file_address,
        }
    }

    pub fn virtual_ip(&self) -> u64
    {
        match self
        {
            AssemblyLine::Instruction(instruction) => instruction.instruction.ip(),
            AssemblyLine::SectionTag(section) => section.virtual_address
        }
    }

    pub fn to_line(&self, color_settings: &ColorSettings, current_byte_index: usize, header: &Header) -> Line
    {
        match self
        {
            AssemblyLine::Instruction(instruction) => {
                let selected = current_byte_index >= instruction.file_address as usize && current_byte_index < instruction.file_address as usize + instruction.instruction.len();
                App::instruction_to_line(color_settings, instruction, selected, header)
            },
            AssemblyLine::SectionTag(section) => 
            {
                let selected = current_byte_index >= section.file_address as usize && current_byte_index < section.file_address as usize + section.size;
                let mut line = Line::default();
                let address_style = if selected
                {
                    color_settings.assembly_selected
                }
                else
                {
                    color_settings.assembly_address
                };
                line.spans.push(Span::styled(format!("{:16X}", section.file_address), address_style));
                line.spans.push(Span::raw(" "));
                line.spans.push(Span::styled(format!("[{} ({}B)]", section.name, section.size), color_settings.assembly_section));
                line.spans.push(Span::styled(format!(" @{:X}", section.virtual_address), color_settings.assembly_virtual_address));
                line
            }
        }
    }
}

impl <'a> App<'a>
{
    pub(super) fn find_symbols(&self, filter: &str) -> Vec<(u64, String)>
    {
        if filter.len() == 0
        {
            return Vec::new();
        }
        let symbol_table = self.header.get_symbols();
        if let Some(symbol_table) = symbol_table
        {
            return symbol_table.iter().filter(|(_, symbol)| symbol.contains(filter)).map(|(address, symbol)| (*address, symbol.clone())).collect()
        }
        else 
        {
            return Vec::new();    
        }
    }

    fn instruction_to_line (color_settings: &ColorSettings, instruction: &InstructionTag, selected: bool, header: &Header) -> Line<'a>
    {
        let symbol_table = header.get_symbols();
        let mut line = Line::default();
        line.spans.push(Span::styled(format!("{:16X}",instruction.file_address), 
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
        

        let instruction_string = if let Some(symbol_table) = &symbol_table
        {
            let symbol_resolver = CustomSymbolResolver::new(symbol_table.clone());
        let symbol_resolver_box = Box::new(symbol_resolver);
            let mut formatter = iced_x86::NasmFormatter::with_options(Some(symbol_resolver_box),None);
            let mut output = String::new();
            formatter.format(&instruction.instruction, &mut output);
            output
        } 
        else
        {
            instruction.instruction.to_string()
        };
        let mut instruction_pieces = instruction_string.split_whitespace();
        let mnemonic = instruction_pieces.next().unwrap().to_string();
        let args = instruction_pieces.collect::<Vec<&str>>().join(" ");
        let mnemonic_style = 
        match instruction.instruction.mnemonic() {
            iced_x86::Mnemonic::Nop => color_settings.assembly_nop,
            iced_x86::Mnemonic::INVALID => color_settings.assembly_bad,
            _ => color_settings.assembly_default,
        };
        

        line.spans.push(Span::styled(mnemonic, mnemonic_style));
        line.spans.push(Span::raw(" "));
        line.spans.push(Span::raw(args));
        if let Some(symbol_table) = symbol_table
        {
            if let Some(symbol) = symbol_table.get(&instruction.instruction.ip())
            {
                line.spans.push(Span::raw(" "));
                line.spans.push(Span::styled(format!("<{}>", symbol), color_settings.assembly_symbol));
            }
        }
        if instruction.instruction.ip() == header.entry_point()
        {
            line.spans.push(Span::raw(" "));
            line.spans.push(Span::styled("EntryPoint", color_settings.assembly_entry_point));
        }
        line.spans.push(Span::styled(format!(" @{:X}", instruction.instruction.ip()), color_settings.assembly_virtual_address));

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
            if section.address > current_byte as u64
            {
                lines.push(AssemblyLine::SectionTag(
                    SectionTag {
                        name: "Unknown".to_string(),
                        file_address: current_byte as u64,
                        virtual_address: 0,
                        size: section.address as usize - current_byte
                    }
                ));
                for _ in 0..section.address as usize - current_byte
                {
                    line_offsets[current_byte] = lines.len() - 1;
                    current_byte += 1;
                }
            }
            // if there are any overlapping sections, this should fix it
            current_byte = section.address as usize;
            match section.name.as_str()
            {
                ".text" => {
                    lines.push(
                        AssemblyLine::SectionTag(
                            SectionTag {
                                name: ".text".to_string(),
                                file_address: section.address,
                                virtual_address: section.virtual_address,
                                size: section.size as usize
                            }
                        )
                    );
                    let (offsets, instructions) = Self::assembly_from_section(bytes, header, section.virtual_address as usize, current_byte, section.size as usize, lines.len());
                    line_offsets.splice(section.address as usize..section.address as usize + section.size as usize, offsets);
                    lines.extend(instructions);
                    current_byte += section.size as usize;
                },
                name => {
                    lines.push(AssemblyLine::SectionTag(
                        SectionTag {
                            name: name.to_string(),
                            file_address: section.address,
                            virtual_address: section.virtual_address,
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
        if current_byte < bytes.len()
        {
            lines.push(AssemblyLine::SectionTag(
                SectionTag {
                    name: "Unknown".to_string(),
                    file_address: current_byte as u64,
                    virtual_address: 0,
                    size: bytes.len() - current_byte
                }
            ));
            for _ in current_byte..bytes.len()
            {
                line_offsets[current_byte] = lines.len() - 1;
                current_byte += 1;
            }
        }

        (line_offsets, lines)
    }

    pub(super) fn assembly_from_section(bytes: &[u8], header: &Header, starting_ip: usize, starting_file_address: usize, section_size: usize, starting_sections: usize) -> (Vec<usize>, Vec<AssemblyLine>)
    {
        let mut line_offsets = vec![0; section_size];
        let mut instructions = Vec::new();
        let mut current_byte = 0;
        let mut decoder = iced_x86::Decoder::new(header.bitness(), &bytes[starting_file_address..starting_file_address + section_size], iced_x86::DecoderOptions::NONE);
        decoder.set_ip(starting_ip as u64);
        for instruction in decoder
        {
            let instruction_tag = InstructionTag
            {
                instruction,
                file_address: current_byte as u64 + starting_file_address as u64
            };
            instructions.push(AssemblyLine::Instruction(instruction_tag));
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
        let current_instruction = self.get_current_instruction().clone();
        let current_ip = match current_instruction
        {
            AssemblyLine::Instruction(instruction) => instruction.file_address,
            AssemblyLine::SectionTag(_) => self.get_cursor_position().global_byte_index as u64
        };
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
                self.log(NotificationLevel::Error, &e);
            }
        }
    }

    pub(super) fn update_assembly_scroll(&mut self)
    {
        // TODO: remove this function because it's useless
        let cursor_position = self.get_cursor_position();
        let current_ip = cursor_position.global_byte_index.min(self.assembly_offsets.len() - 1);
        let current_scroll = self.assembly_offsets[current_ip];
        
        self.assembly_scroll = current_scroll;
    }

    pub(super) fn get_assembly_view_scroll(&self) -> usize
    {
        let visible_lines = self.screen_size.1 - self.vertical_margin;
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
        let virtual_address = self.get_current_instruction().virtual_ip();
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
        decoder.set_ip(virtual_address as u64);
        let mut offsets = Vec::new();
        let mut instructions = Vec::new();
        let mut instruction_lines = Vec::new();
        let mut to_byte = self.data.len();

        let from_instruction = self.assembly_offsets[from_byte];
        let mut current_byte = from_byte;
        for instruction in decoder
        {   
            let old_instruction = self.get_instruction_at(current_byte);
            let instruction_tag = InstructionTag
            {
                instruction,
                file_address: current_byte as u64
            };
            if old_instruction == &AssemblyLine::Instruction(instruction_tag)
            {
                to_byte = old_instruction.ip() as usize;
                break;
            }
            instructions.push(AssemblyLine::Instruction(instruction_tag));
            instruction_lines.push(Self::instruction_to_line(&self.color_settings, &instruction_tag, false, &self.header));
            for _ in 0..instruction.len()
            {
                offsets.push(from_instruction + instructions.len() - 1);
                current_byte += 1;
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
                self.log(NotificationLevel::Debug, &format!("Removing instruction \"{}\" at {:X}", instruction.instruction, self.assembly_instructions[i].ip()));    
            }
        }
        for i in 0..instructions.len()
        {
            if let AssemblyLine::Instruction(instruction) = &instructions[i]
            {
                self.log(NotificationLevel::Debug, &format!("Adding instruction \"{}\" at {:X}", instruction.instruction, instructions[i].ip()));
            }
        }

        self.assembly_instructions.splice(from_instruction..to_instruction, instructions);

        self.update_assembly_scroll();
    }
}