use iced_x86::Instruction;
use ratatui::{style::{Color, Style}, text::{Line, Span, Text}};

use super::app::App;

impl <'a> App<'a>
{
    fn instruction_to_line(instruction: &Instruction, selected: bool) -> Line<'a>
    {
        let mut line = Line::default();
        line.spans.push(Span::styled(format!("{:16X}",instruction.ip()), 
            if selected
            {
                Style::default().fg(Color::Black).bg(Color::White)
            }
            else 
            {    
                Style::default()
            }
        ));
        line.spans.push(Span::raw(" "));
        let instruction_string = instruction.to_string();
        let mut instruction_pieces = instruction_string.split_whitespace();
        let mnemonic = instruction_pieces.next().unwrap().to_string();
        let args = instruction_pieces.collect::<Vec<&str>>().join(" ");
        
        line.spans.push(Span::styled(mnemonic, Style::default().fg(Color::Yellow)));
        line.spans.push(Span::raw(" "));
        line.spans.push(Span::raw(args));
        line
    }

    pub(super) fn assembly_from_bytes(bytes: &[u8]) -> (Text<'a>, Vec<usize>)
    {
        let mut output = Text::default();
        let mut line_offsets = vec![0; bytes.len()];
        let decoder = iced_x86::Decoder::new(64, bytes, iced_x86::DecoderOptions::NONE);
        let mut byte_index = 0;
        let mut line_index = 0;
        for instruction in decoder {
            
            let line = Self::instruction_to_line(&instruction, line_index == 0);
            
            for _ in 0..instruction.len() {
                line_offsets[byte_index] = line_index;
                byte_index += 1;
            }
            line_index += 1;
            output.lines.push(line);
        }
        (output, line_offsets)
    }

    pub(super) fn update_assembly_scroll(&mut self)
    {
        let cursor_position = self.get_cursor_position();
        let current_ip = cursor_position.global_byte_index as usize;
        let current_scroll = self.assembly_offsets[current_ip];
        
        self.assembly_view.lines[self.assembly_scroll].spans[0].style = Style::default();
        self.assembly_view.lines[current_scroll].spans[0].style = 
            Style::default().fg(Color::Black).bg(Color::White);
        self.assembly_scroll = current_scroll;
    }

    pub(super) fn get_assembly_view_scroll(&self) -> usize
    {
        let center_of_view = (self.screen_size.1 - 3) as isize / 2;
        let view_scroll = (self.assembly_scroll as isize - center_of_view).max(0);
        
        return view_scroll as usize;
    }

    pub(super) fn edit_assembly(&mut self)
    {
        (self.assembly_view, self.assembly_offsets) = Self::assembly_from_bytes(&self.data);
        self.assembly_scroll = 0;
        self.update_assembly_scroll();
    }
}