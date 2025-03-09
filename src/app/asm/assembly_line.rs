use std::collections::HashMap;

use ratatui::text::{Line, Span};

use crate::{
    app::{settings::color_settings::ColorSettings, App},
    headers::Header,
};

use super::{instruction_tag::InstructionTag, section_tag::SectionTag};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssemblyLine {
    Instruction(InstructionTag),
    SectionTag(SectionTag),
}

impl AssemblyLine {
    pub fn file_address(&self) -> u64 {
        match self {
            AssemblyLine::Instruction(instruction) => instruction.file_address,
            AssemblyLine::SectionTag(section) => section.file_address,
        }
    }

    pub fn virtual_address(&self) -> u64 {
        match self {
            AssemblyLine::Instruction(instruction) => instruction.instruction.ip(),
            AssemblyLine::SectionTag(section) => section.virtual_address,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            AssemblyLine::Instruction(instruction) => instruction.instruction.len(),
            AssemblyLine::SectionTag(section) => section.size,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            AssemblyLine::Instruction(instruction) => instruction.instruction.is_empty(),
            AssemblyLine::SectionTag(section) => section.size == 0,
        }
    }

    pub fn to_line(
        &self,
        color_settings: &ColorSettings,
        current_byte_index: usize,
        header: &Header,
        address_min_width: usize,
        comments: &HashMap<u64, String>,
    ) -> Line {
        match self {
            AssemblyLine::Instruction(instruction) => {
                let selected = current_byte_index >= instruction.file_address as usize
                    && current_byte_index
                        < instruction.file_address as usize + instruction.instruction.len();
                App::instruction_to_line(
                    color_settings,
                    instruction,
                    selected,
                    header,
                    address_min_width,
                    comments.get(&instruction.file_address).map(|s| s.as_str()),
                )
            }
            AssemblyLine::SectionTag(section) => {
                let selected = current_byte_index >= section.file_address as usize
                    && current_byte_index < section.file_address as usize + section.size;
                let mut line = Line::default();
                let address_style = if selected {
                    color_settings.assembly_selected
                } else {
                    color_settings.assembly_address
                };
                line.spans.push(Span::styled(
                    format!("{:>address_min_width$X}", section.file_address),
                    address_style,
                ));
                line.spans.push(Span::raw(" "));
                line.spans.push(Span::styled(
                    format!("[{} ({}B)]", section.name, section.size),
                    color_settings.assembly_section,
                ));
                line.spans.push(Span::styled(
                    format!(" @{:X}", section.virtual_address),
                    color_settings.assembly_virtual_address,
                ));
                line
            }
        }
    }

    pub fn is_same_instruction(&self, other: &AssemblyLine) -> bool {
        match (self, other) {
            (
                AssemblyLine::Instruction(instruction),
                AssemblyLine::Instruction(other_instruction),
            ) => {
                instruction.instruction.bytes == other_instruction.instruction.bytes
                    && instruction.instruction.virtual_address
                        == other_instruction.instruction.virtual_address
            }
            _ => false,
        }
    }
}
