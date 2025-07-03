use ratatui::text::{Line, Span};

use crate::{
    app::{
        instruction::Instruction, log::NotificationLevel, settings::color_settings::ColorSettings,
        App,
    },
    asm::assembler::assemble,
    get_app_context,
    headers::{section::Section, Header},
};

use super::{
    assembly_line::AssemblyLine, instruction_tag::InstructionTag, section_tag::SectionTag,
};

impl App {
    pub(in crate::app) fn find_symbols(&self, filter: &str) -> Vec<(u64, String)> {
        if filter.is_empty() {
            return Vec::new();
        }
        let symbol_table = self.header.get_symbols();
        if let Some(symbol_table) = symbol_table {
            let mut symbols: Vec<(u64, String)> = symbol_table
                .iter()
                .filter(|(_, symbol)| symbol.contains(filter))
                .map(|(address, symbol)| (*address, symbol.clone()))
                .collect();
            symbols.sort_by_key(|(_, symbol)| symbol.len());
            symbols
        } else {
            Vec::new()
        }
    }

    pub(super) fn instruction_to_line(
        color_settings: &ColorSettings,
        instruction: &InstructionTag,
        selected: bool,
        header: &Header,
        address_min_width: usize,
        comment: Option<&str>,
    ) -> Line<'static> {
        let symbol_table = header.get_symbols();
        let mut line = Line::default();
        line.spans.push(Span::styled(
            format!("{:>address_min_width$X}", instruction.file_address),
            if selected {
                color_settings.assembly_selected
            } else {
                color_settings.assembly_address
            },
        ));
        line.spans.push(Span::raw(" "));

        let mnemonic = instruction.instruction.mnemonic();
        let args = instruction.instruction.operands();
        let mnemonic_style = match instruction.instruction.mnemonic() {
            "nop" => color_settings.assembly_nop,
            ".byte" => color_settings.assembly_bad,
            _ => color_settings.assembly_default,
        };

        line.spans
            .push(Span::styled(mnemonic.to_string(), mnemonic_style));
        line.spans.push(Span::raw(" "));
        line.spans.push(Span::raw(args.to_string()));
        if let Some(symbol_table) = symbol_table {
            if let Some(symbol) = symbol_table.get(&instruction.instruction.ip()) {
                line.spans.push(Span::raw(" "));
                line.spans.push(Span::styled(
                    format!("<{symbol}>"),
                    color_settings.assembly_symbol,
                ));
            }
        }
        if instruction.instruction.ip() == header.entry_point() {
            line.spans.push(Span::raw(" "));
            line.spans.push(Span::styled(
                t!("app.entry_point"),
                color_settings.assembly_entry_point,
            ));
        }
        line.spans.push(Span::styled(
            format!(" @{:X}", instruction.instruction.ip()),
            color_settings.assembly_virtual_address,
        ));
        if let Some(comment) = comment {
            line.spans.push(Span::raw(" "));
            line.spans.push(Span::styled(
                format!("; {comment}"),
                color_settings.assembly_comment,
            ));
        }

        line
    }

    pub(super) fn section_to_line(
        color_settings: &ColorSettings,
        section: &SectionTag,
        selected: bool,
        address_min_width: usize,
        comment: Option<&str>,
    ) -> Line<'static> {
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
        if let Some(comment) = comment {
            line.spans.push(Span::raw(" "));
            line.spans.push(Span::styled(
                format!("; {comment}"),
                color_settings.assembly_comment,
            ));
        }
        line
    }

    pub(in crate::app) fn sections_from_bytes(
        bytes: &[u8],
        header: &Header,
    ) -> (Vec<usize>, Vec<AssemblyLine>) {
        let mut line_offsets = vec![0; bytes.len()];
        let mut lines = Vec::new();
        let mut sections = header.get_sections();
        if sections.is_empty() {
            sections.push(Section {
                name: ".text".to_string(),
                virtual_address: 0,
                file_offset: 0,
                size: bytes.len() as u64,
            });
        }

        let mut current_byte = 0;
        for section in sections {
            if section.file_offset > current_byte as u64 {
                lines.push(AssemblyLine::SectionTag(SectionTag {
                    name: t!("app.unknown_section").to_string(),
                    file_address: current_byte as u64,
                    virtual_address: 0,
                    size: section.file_offset as usize - current_byte,
                }));
                for _ in 0..section.file_offset as usize - current_byte {
                    line_offsets[current_byte] = lines.len() - 1;
                    current_byte += 1;
                }
            }
            // if there are any overlapping sections, this should fix it
            current_byte = section.file_offset as usize;
            match section.name.as_str() {
                ".text" | "__text" => {
                    lines.push(AssemblyLine::SectionTag(SectionTag {
                        name: section.name.clone(),
                        file_address: section.file_offset,
                        virtual_address: section.virtual_address,
                        size: section.size as usize,
                    }));
                    let (offsets, instructions) = Self::assembly_from_section(
                        bytes,
                        header,
                        section.virtual_address as usize,
                        current_byte,
                        section.size as usize,
                        lines.len(),
                    );
                    line_offsets.splice(
                        section.file_offset as usize
                            ..section.file_offset as usize + section.size as usize,
                        offsets,
                    );
                    lines.extend(instructions);
                    current_byte += section.size as usize;
                }
                name => {
                    lines.push(AssemblyLine::SectionTag(SectionTag {
                        name: name.to_string(),
                        file_address: section.file_offset,
                        virtual_address: section.virtual_address,
                        size: section.size as usize,
                    }));
                    for _ in 0..section.size as usize {
                        line_offsets[current_byte] = lines.len() - 1;
                        current_byte += 1;
                    }
                }
            }
        }
        if current_byte < bytes.len() {
            lines.push(AssemblyLine::SectionTag(SectionTag {
                name: t!("app.unknown_section").to_string(),
                file_address: current_byte as u64,
                virtual_address: 0,
                size: bytes.len() - current_byte,
            }));
            let initial_current_byte = current_byte;
            for _ in initial_current_byte..bytes.len() {
                line_offsets[current_byte] = lines.len() - 1;
                current_byte += 1;
            }
        }

        (line_offsets, lines)
    }

    pub(in crate::app) fn assembly_from_section(
        bytes: &[u8],
        header: &Header,
        starting_ip: usize,
        starting_file_address: usize,
        section_size: usize,
        starting_sections: usize,
    ) -> (Vec<usize>, Vec<AssemblyLine>) {
        let mut line_offsets = vec![0; section_size];
        let mut instructions = Vec::new();
        let mut current_byte = 0;
        let decoder = header.get_decoder().expect(&t!("errors.create_decoder"));
        let decoded = decoder
            .disasm_all(
                &bytes[starting_file_address..starting_file_address + section_size],
                starting_ip as u64,
            )
            .expect(&t!("errors.disassemble"));
        for instruction in decoded.iter() {
            let instruction_tag = InstructionTag {
                instruction: Instruction::new(instruction, header.get_symbols()),
                file_address: current_byte as u64 + starting_file_address as u64,
            };
            instructions.push(AssemblyLine::Instruction(instruction_tag));
            for _ in 0..instruction.len() {
                line_offsets[current_byte] = starting_sections + instructions.len() - 1;
                current_byte += 1;
            }
        }
        (line_offsets, instructions)
    }

    pub(in crate::app) fn bytes_from_assembly(
        &self,
        assembly: &str,
        starting_virtual_address: u64,
    ) -> Result<Vec<u8>, String> {
        let bytes = assemble(assembly, starting_virtual_address, &self.header);
        match bytes {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(e.to_string()),
        }
    }

    pub(in crate::app) fn patch_bytes(
        &mut self,
        bytes: &[u8],
        start_from_beginning_of_instruction: bool,
    ) {
        let current_instruction = self.get_current_instruction();
        if let Some(current_instruction) = current_instruction {
            let current_instruction = current_instruction.clone();
            let current_ip = match &current_instruction {
                AssemblyLine::Instruction(instruction) => instruction.file_address,
                AssemblyLine::SectionTag(_) => self.get_cursor_position().global_byte_index as u64,
            };
            let instruction_offset = if start_from_beginning_of_instruction {
                0
            } else {
                self.get_cursor_position().global_byte_index - current_ip as usize
            };
            let offset = current_ip as usize + instruction_offset;
            let mut bytes = bytes.to_vec();
            let mut app_context = get_app_context!(self);
            app_context.offset = offset;
            self.plugin_manager.on_edit(&mut bytes, &mut app_context);

            let modified_bytes = self.data.push_change(offset, bytes);

            self.edit_assembly(modified_bytes + instruction_offset);
        }
    }

    pub(in crate::app) fn patch(&mut self, assembly: &str) {
        if let Some(current_instruction) = self.get_current_instruction() {
            let current_virtual_address =
                if let AssemblyLine::Instruction(instruction) = current_instruction {
                    instruction.instruction.ip()
                } else {
                    self.get_cursor_position().global_byte_index as u64
                };
            let bytes = self.bytes_from_assembly(assembly, current_virtual_address);
            match bytes {
                Ok(bytes) => self.patch_bytes(&bytes, true),
                Err(e) => {
                    self.log(NotificationLevel::Error, &e);
                }
            }
        }
    }

    pub(in crate::app) fn get_assembly_view_scroll(&self) -> usize {
        let cursor_position = self.get_cursor_position();
        let current_ip = cursor_position
            .global_byte_index
            .min(self.assembly_offsets.len() - 1);
        let current_scroll = self.assembly_offsets[current_ip];

        let visible_lines = self.screen_size.1 - self.vertical_margin;
        let center_of_view = visible_lines / 2;
        let view_scroll = (current_scroll as isize - center_of_view as isize).clamp(
            0,
            (self.assembly_instructions.len() as isize - visible_lines as isize).max(0),
        );

        view_scroll as usize
    }

    pub(in crate::app) fn get_current_instruction(&self) -> Option<&AssemblyLine> {
        let global_byte_index = self.get_cursor_position().global_byte_index;
        if global_byte_index >= self.assembly_offsets.len() {
            return None;
        }
        let current_instruction_index = self.assembly_offsets[global_byte_index];
        Some(&self.assembly_instructions[current_instruction_index])
    }

    pub(in crate::app) fn get_instruction_at(&self, index: usize) -> &AssemblyLine {
        let current_instruction_index = self.assembly_offsets[index];
        &self.assembly_instructions[current_instruction_index]
    }

    pub(in crate::app) fn edit_assembly(&mut self, modifyied_bytes: usize) {
        let current_instruction = self.get_current_instruction();
        if let Some(current_instruction) = current_instruction {
            let from_byte = current_instruction.file_address() as usize;
            let virtual_address = current_instruction.virtual_address();
            let text_section = self.header.get_text_section();
            let (is_inside_text_section, maximum_code_byte) =
                if let Some(text_section) = text_section {
                    (
                        from_byte >= text_section.file_offset as usize
                            && from_byte
                                < text_section.file_offset as usize + text_section.size as usize,
                        text_section.file_offset as usize + text_section.size as usize,
                    )
                } else {
                    (true, self.data.len())
                };
            if !is_inside_text_section {
                return;
            }
            let decoder = self
                .header
                .get_decoder()
                .expect(&t!("errors.create_decoder"));
            let mut offsets = Vec::new();
            let mut instructions = Vec::new();
            let mut to_byte = self.data.len();

            let from_instruction = self.assembly_offsets[from_byte];
            let mut current_byte = from_byte;
            let mut ip_offset = 0;

            loop {
                if current_byte >= maximum_code_byte {
                    to_byte = maximum_code_byte;
                    break;
                }
                let bytes = &self.data.bytes()[current_byte..maximum_code_byte];
                let decoded = decoder
                    .disasm_count(bytes, virtual_address + ip_offset, 1)
                    .expect(&t!("errors.disassemble"));
                if decoded.is_empty() {
                    break;
                }
                let instruction = decoded.iter().next().unwrap();
                ip_offset += instruction.len() as u64;
                let old_instruction = self.get_instruction_at(current_byte);
                let instruction_tag = InstructionTag {
                    instruction: Instruction::new(instruction, self.header.get_symbols()),
                    file_address: current_byte as u64,
                };
                let new_assembly_line = AssemblyLine::Instruction(instruction_tag.clone());
                if old_instruction.is_same_instruction(&new_assembly_line)
                    && current_byte - from_byte >= modifyied_bytes
                {
                    to_byte = old_instruction.file_address() as usize;
                    break;
                }
                instructions.push(new_assembly_line);
                for _ in 0..instruction.len() {
                    offsets.push(from_instruction + instructions.len() - 1);
                    current_byte += 1;
                }
            }
            if from_byte == to_byte {
                return;
            }

            let to_instruction = self
                .assembly_offsets
                .get(to_byte)
                .cloned()
                .unwrap_or(self.assembly_instructions.len());

            let mut original_instruction_count = 1;
            let mut original_instruction_ip = self.assembly_offsets[from_byte];
            for i in from_byte..to_byte {
                if self.assembly_offsets[i] != original_instruction_ip {
                    original_instruction_count += 1;
                    original_instruction_ip = self.assembly_offsets[i];
                }
            }

            let new_instruction_count = instructions.len();

            let delta = new_instruction_count as isize - original_instruction_count as isize;

            self.assembly_offsets.splice(from_byte..to_byte, offsets);
            if delta != 0 {
                for offset in self.assembly_offsets.iter_mut().skip(to_byte) {
                    *offset = (*offset as isize + delta) as usize;
                }
            }

            for i in from_instruction..to_instruction {
                if let AssemblyLine::Instruction(instruction) = &self.assembly_instructions[i] {
                    self.log(
                        NotificationLevel::Debug,
                        t!(
                            "app.messages.removing_instruction",
                            instruction = instruction.instruction,
                            address = self.assembly_instructions[i].file_address() : {:#X}
                        ),
                    );
                } else {
                    break;
                }
            }
            for instruction in instructions.iter() {
                if let AssemblyLine::Instruction(instruction_tag) = instruction {
                    self.log(
                        NotificationLevel::Debug,
                        t!(
                            "app.messages.adding_instruction",
                            instruction = instruction_tag.instruction,
                            address = instruction.file_address() : {:#X}
                        ),
                    );
                }
            }

            self.assembly_instructions
                .splice(from_instruction..to_instruction, instructions);
        }
    }

    pub(in crate::app) fn parse_header(&mut self) -> Header {
        let mut app_context = get_app_context!(self);
        match self.plugin_manager.try_parse_header(&mut app_context) {
            Some(header) => Header::CustomHeader(header),
            None => {
                Header::parse_header(self.data.bytes(), self.filesystem.pwd(), &self.filesystem)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::app::comments::Comments;

    use super::*;
    #[test]
    fn test_assembly_line() {
        let file_address = 0xdeadbeef;
        let virtual_address = 0xcafebabe;

        let al = AssemblyLine::Instruction(InstructionTag {
            instruction: Instruction {
                mnemonic: "mov".to_string(),
                operands: "rax, rbx".to_string(),
                virtual_address,
                bytes: vec![0x48, 0x89, 0xd8],
            },
            file_address,
        });
        let mut comments = Comments::default();
        comments.insert(file_address, "This is a comment".into());
        let line = al.to_line(
            &ColorSettings::get_default_dark_theme(),
            0,
            &Header::None,
            0,
            &comments,
        );

        let contains_mnemonic = line.spans.iter().any(|span| span.content.contains("mov"));
        assert!(contains_mnemonic);
        let contains_operands = line
            .spans
            .iter()
            .any(|span| span.content.contains("rax, rbx"));
        assert!(contains_operands);
        let comma_count = line
            .spans
            .iter()
            .map(|span| span.content.chars().filter(|c| *c == ',').count())
            .sum::<usize>();
        assert_eq!(comma_count, 1);
        let contains_virtual_address = line
            .spans
            .iter()
            .any(|span| span.content.contains(&format!("{virtual_address:X}")));
        assert!(contains_virtual_address);
        let contains_file_address = line
            .spans
            .iter()
            .any(|span| span.content.contains(&format!("{file_address:X}")));
        assert!(contains_file_address);
        let contains_comment = line
            .spans
            .iter()
            .any(|span| span.content.contains(comments.get(&file_address).unwrap()));
        assert!(
            contains_comment,
            "Comment {} not found in line {:?}",
            comments.get(&0).unwrap(),
            line
        );

        let section_size = 0x1000;

        let al = AssemblyLine::SectionTag(SectionTag {
            name: ".text".to_string(),
            file_address,
            virtual_address,
            size: section_size,
        });

        let line = al.to_line(
            &ColorSettings::get_default_dark_theme(),
            0,
            &Header::None,
            0,
            &comments,
        );

        let contains_section_name = line.spans.iter().any(|span| span.content.contains(".text"));
        assert!(contains_section_name);
        let contains_virtual_address = line
            .spans
            .iter()
            .any(|span| span.content.contains(&format!("{virtual_address:X}")));
        assert!(contains_virtual_address);
        let contains_file_address = line
            .spans
            .iter()
            .any(|span| span.content.contains(&format!("{file_address:X}")));
        assert!(contains_file_address);
        let contains_size = line
            .spans
            .iter()
            .any(|span| span.content.contains(&format!("{section_size}B")));
        assert!(contains_size);
        let contains_comment = line
            .spans
            .iter()
            .any(|span| span.content.contains(comments.get(&file_address).unwrap()));
        assert!(contains_comment);
    }

    #[test]
    fn test_disassemble_and_patch() {
        let data = vec![0x48, 0x89, 0xd8, 0x48, 0x89, 0xc1, 0x48, 0x89, 0xc0];
        let mut app = App::mockup(data);
        app.resize_to_size(80, 24);
        let mut expected_instructions = vec!["mov rax, rbx", "mov rcx, rax", "mov rax, rax"];
        expected_instructions.reverse();
        let mut text_found = false;
        for line in app.assembly_instructions.iter() {
            match line {
                AssemblyLine::Instruction(instruction) => {
                    assert!(text_found, "Instructions must be after .text section");
                    let instruction_text = expected_instructions
                        .pop()
                        .expect("There are too many instructions in assembly_instructions");
                    assert!(instruction
                        .instruction
                        .to_string()
                        .contains(instruction_text));
                }
                AssemblyLine::SectionTag(section) => {
                    if text_found {
                        panic!("There are too many .text sections in assembly_instructions");
                    }
                    assert_eq!(section.name, ".text");
                    text_found = true;
                }
            }
        }
        assert!(text_found);

        app.patch("nop; nop; nop;");
        let expected_data = vec![0x90, 0x90, 0x90, 0x48, 0x89, 0xc1, 0x48, 0x89, 0xc0];
        let mut expected_instructions = vec!["nop", "nop", "nop", "mov rcx, rax", "mov rax, rax"];
        expected_instructions.reverse();
        assert_eq!(app.data.bytes(), expected_data);
        text_found = false;
        for line in app.assembly_instructions.iter() {
            match line {
                AssemblyLine::Instruction(instruction) => {
                    assert!(text_found, "Instructions must be after .text section");
                    let instruction_text = expected_instructions
                        .pop()
                        .expect("There are too many instructions in assembly_instructions");
                    assert!(instruction
                        .instruction
                        .to_string()
                        .contains(instruction_text));
                }
                AssemblyLine::SectionTag(section) => {
                    if text_found {
                        panic!("There are too many .text sections in assembly_instructions");
                    }
                    assert_eq!(section.name, ".text");
                    text_found = true;
                }
            }
        }
        assert!(text_found);

        // move one byte forward
        app.move_cursor(2, 0, false);

        app.patch("jmp rax");
        let expected_data = vec![0x90, 0xff, 0xe0, 0x48, 0x89, 0xc1, 0x48, 0x89, 0xc0];
        let mut expected_instructions = vec!["nop", "jmp rax", "mov rcx, rax", "mov rax, rax"];
        expected_instructions.reverse();
        assert_eq!(app.data.bytes(), expected_data);
        text_found = false;
        for line in app.assembly_instructions.iter() {
            match line {
                AssemblyLine::Instruction(instruction) => {
                    assert!(text_found, "Instructions must be after .text section");
                    let instruction_text = expected_instructions
                        .pop()
                        .expect("There are too many instructions in assembly_instructions");
                    assert!(instruction
                        .instruction
                        .to_string()
                        .contains(instruction_text));
                }
                AssemblyLine::SectionTag(section) => {
                    if text_found {
                        panic!("There are too many .text sections in assembly_instructions");
                    }
                    assert_eq!(section.name, ".text");
                    text_found = true;
                }
            }
        }
        assert!(text_found);
    }

    #[test]
    fn test_bad_instruction() {
        let data = vec![0x06, 0x0e, 0x07];
        let app = App::mockup(data);
        for line in app.assembly_instructions.iter() {
            if let AssemblyLine::Instruction(instruction) = line {
                let contains_bad_instruction =
                    instruction.instruction.to_string().contains(".byte");
                assert!(
                    contains_bad_instruction,
                    "Found {} instead of .byte ...",
                    instruction.instruction
                );
            }
        }
    }
}
