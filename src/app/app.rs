use std::{path::PathBuf, time::Duration};

use crossterm::event;
use ratatui::{backend::Backend, layout::Rect, style::{Color, Modifier, Style}, text::{Line, Span, Text}, widgets::{Block, Borders, ScrollbarState}, Frame};

use super::{info_mode::InfoMode, paragraph::Paragraph, popup_state::PopupState};

pub struct App<'a> 
{
    pub(super) path: PathBuf,
    pub(super) output: String,
    pub(super) dirty: bool,
    pub(super) data: Vec<u8>,
    pub(super) address_view: Text<'a>,
    pub(super) hex_view: Text<'a>,
    pub(super) text_view: Text<'a>,
    pub(super) assembly_view: Text<'a>,
    pub(super) assembly_offsets: Vec<usize>,
    pub(super) assembly_scroll: usize,
    pub(super) info_mode: InfoMode,
    pub(super) scroll: u16,
    pub(super) cursor: (u16, u16),
    pub(super) poll_time: Duration,
    pub(super) needs_to_exit: bool,
    pub(super) screen_size: (u16, u16),

    pub(super) popup: Option<PopupState>,

    pub(super) block_size: usize,
    pub(super) blocks_per_row: usize,
}

impl <'a> App<'a>
{
    pub fn new(file_path: PathBuf) -> Result<Self,String>
    {
        let data = std::fs::read(&file_path).map_err(|e| e.to_string())?;
        if data.len() > 0xFFFF
        {
            return Err("File is too large (size > 0xFFFF Bytes), ratatui does not support this many lines for some reason.".to_string());
        }
        let block_size = 8;
        let blocks_per_row = 3;
        let address_view = Self::addresses(data.len(), block_size, blocks_per_row);
        let hex_view = Self::bytes_to_styled_hex(&data, block_size, blocks_per_row);
        let text_view = Self::bytes_to_styled_text(&data, block_size, blocks_per_row);
        let (assembly_view, assembly_offsets) = Self::assembly_from_bytes(&data);
        Ok(App{
            path: file_path,
            data,
            output: "Press H to view a help page.".to_string(),
            dirty: false,
            address_view,
            hex_view,
            text_view, 
            assembly_view,
            assembly_offsets,
            assembly_scroll: 0,
            info_mode: InfoMode::Text,
            scroll: 0,
            cursor: (0,0),
            poll_time: Duration::from_millis(1000),
            needs_to_exit: false,
            screen_size: (1,1),

            popup: None,

            block_size,
            blocks_per_row,
        })
    }

    pub(super) fn get_style_for_byte(byte: u8) -> Style
    {
        match byte
        {
            // null
            0x00 => Style::default().fg(Color::DarkGray),
            // newline
            0x0A | 0x0C | 0x0D => Style::default().fg(Color::LightRed),
            // whitespace
            0x20 | 0x09 | 0x0B => Style::default().fg(Color::Rgb(244, 202, 183)),
            // numbers
            0x30..=0x39 => Style::default().fg(Color::Rgb(204, 152, 113)).add_modifier(Modifier::BOLD),
            // uppercase
            0x41..=0x5A => Style::default().fg(Color::Rgb(204, 152, 113)).add_modifier(Modifier::BOLD),
            // lowercase
            0x61..=0x7A => Style::default().fg(Color::Rgb(204, 152, 113)).add_modifier(Modifier::BOLD),
            // special characters
            0x20..=0x7E => Style::default().fg(Color::Rgb(204, 152, 113)).add_modifier(Modifier::DIM),
            _ => Style::default()
        }
    }

    pub(super) fn bytes_to_styled_hex(bytes: &[u8], block_size: usize, blocks_per_row: usize) -> Text<'a>
    {
        let mut ret = Text::default();
        let mut current_line = Line::default();
        let mut local_block = 0;
        let mut local_byte = 0;
        for b in bytes
        {
            let mut next_line = false;
            let hex_chars = Self::u8_to_hex(*b);
            let mut hex_string = hex_chars.iter().collect::<String>();
            hex_string.push(' ');
            local_byte += 1;
            if local_byte % block_size == 0
            {
                local_byte = 0;
                hex_string.push(' ');

                local_block += 1;
                if local_block % blocks_per_row == 0
                {
                    local_block = 0;
                    next_line = true;
                }
            }

            let style = Self::get_style_for_byte(*b);
            let span = Span::styled(hex_string, style);
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

    pub(super) fn bytes_to_styled_text(bytes: &'_[u8], block_size: usize, blocks_per_row: usize) -> Text<'a>
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

            let style = Self::get_style_for_byte(*b);
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

    pub(super) fn resize_if_needed(&mut self, width: u16)
    {
        let blocks_per_row = self.calc_blocks_per_row(width);
        if self.blocks_per_row != blocks_per_row
        {
            self.resize(blocks_per_row);
        }
    }

    pub(super) fn resize(&mut self, blocks_per_row: usize)
    {
        self.blocks_per_row = blocks_per_row;
        self.address_view = Self::addresses(self.data.len(), self.block_size, self.blocks_per_row);
        self.hex_view = Self::bytes_to_styled_hex(&self.data, self.block_size, self.blocks_per_row);
        self.text_view = Self::bytes_to_styled_text(&self.data, self.block_size, self.blocks_per_row);
    }

    pub(super) fn calc_blocks_per_row(&self, width: u16) -> usize
    {
        let block_characters_hex = self.block_size * 3 + 1;
        let block_characters_text = self.block_size * 2 + 1;
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

    pub(super) fn addresses(size: usize, block_size: usize, blocks_per_row: usize) -> Text<'a>
    {
        let mut result = Text::default();

        for i in 0..=size/(block_size * blocks_per_row)
        {
            let mut line = Line::default();
            line.spans.push(Span::styled(format!("{:16X}", i * block_size * blocks_per_row), if i % 2 == 0 {Style::default().fg(Color::DarkGray)} else {Style::default()}));
            result.lines.push(line);
        }
        result
    }

    pub(super) fn edit_data(&mut self, mut value: char)
    {
        value = value.to_uppercase().next().unwrap(); 

        if value >= '0' && value <= '9' || value >= 'A' && value <= 'F'
        {   
            let cursor_position = self.get_cursor_position();

            let mut old_str = self.hex_view.lines[cursor_position.line_index as usize]
                .spans[cursor_position.line_byte_index as usize].content.to_string();

            if old_str.as_bytes()[(cursor_position.local_x % 3) as usize] != value as u8
            {
                self.dirty = true;
            }

            unsafe {
                old_str.as_bytes_mut()[(cursor_position.local_x % 3) as usize] = value as u8;
            }
            
            let hex = old_str.chars().filter(|c| c.is_whitespace() == false).collect::<String>();

            let byte = u8::from_str_radix(&hex, 16).unwrap();

            self.data[cursor_position.global_byte_index as usize] = byte;

            let style = Self::get_style_for_byte(byte);
            self.hex_view.lines[cursor_position.line_index as usize]
                .spans[cursor_position.line_byte_index as usize] = Span::styled(old_str, style);
            
            let text = App::u8_to_char(byte);
            let old_str = self.text_view.lines[cursor_position.line_index as usize]
                .spans[cursor_position.line_byte_index as usize].content.to_string();
            let text_iterator = old_str.chars().filter(|c| c.is_whitespace());
            let mut new_str = text.to_string();
            new_str.extend(text_iterator);

            self.text_view.lines[cursor_position.line_index as usize]
                .spans[cursor_position.line_byte_index as usize] = Span::styled(new_str, style);
        }
        self.edit_assembly();
    }

    pub(super) fn save_data(&mut self)
    {
        self.output = "Converting data...".to_string();
        self.output = "Saving...".to_string();
        std::fs::write(&self.path, &self.data).unwrap();
        self.dirty = false;
        self.output = format!("Saved to {}", self.path.to_str().unwrap());
    }

    pub(super) fn fill_popup(popup_state: &PopupState, f: &Frame, popup_title: &mut &str, popup_text: &mut Text, popup_rect: &mut Rect)
    {
        match &popup_state
        {
            PopupState::SaveAndQuit(yes_selected) =>
            {
                *popup_title = "Save and Quit";
                popup_text.lines.extend(
                    vec![
                        Line::raw("The file will be saved and the program will quit."),
                        Line::raw("Are you sure?"),
                        Line::from(vec![
                            Span::styled("Yes", Style::default().fg(Color::Green)),
                            Span::raw("  "),
                            Span::styled("No", Style::default().fg(Color::Red))
                        ])
                    ]
                );
                if *yes_selected
                {
                    popup_text.lines[2].spans[0].style = Style::default().fg(Color::White).bg(Color::Green);
                }
                else
                {
                    popup_text.lines[2].spans[2].style = Style::default().fg(Color::White).bg(Color::Red);
                }
            },
            PopupState::Save(yes_selected) =>
            {
                *popup_title = "Save";
                popup_text.lines.extend(
                    vec![
                        Line::raw("The file will be saved."),
                        Line::raw("Are you sure?"),
                        Line::from(vec![
                            Span::styled("Yes", Style::default().fg(Color::Green)),
                            Span::raw("  "),
                            Span::styled("No", Style::default().fg(Color::Red))
                        ])
                    ]
                );
                if *yes_selected
                {
                    popup_text.lines[2].spans[0].style = Style::default().fg(Color::White).bg(Color::Green);
                }
                else
                {
                    popup_text.lines[2].spans[2].style = Style::default().fg(Color::White).bg(Color::Red);
                }
            },
            PopupState::QuitDirtySave(yes_selected) =>
            {
                *popup_title = "Quit";
                popup_text.lines.extend(
                    vec![
                        Line::raw("The file has been modified."),
                        Line::raw("Do you want to save before quitting?"),
                        Line::from(vec![
                            Span::styled("Yes", Style::default().fg(Color::Green)),
                            Span::raw("  "),
                            Span::styled("No", Style::default().fg(Color::Red))
                        ])
                    ]
                );
                if *yes_selected
                {
                    popup_text.lines[2].spans[0].style = Style::default().fg(Color::White).bg(Color::Green);
                }
                else
                {
                    popup_text.lines[2].spans[2].style = Style::default().fg(Color::White).bg(Color::Red);
                }
            },
            PopupState::Help =>
            {
                *popup_rect = Rect::new(f.size().width / 2 - 15, f.size().height / 2 - 4, 30, 8);
                *popup_title = "Help";
                popup_text.lines.extend(
                    vec![
                        Line::from(
                            vec![
                                Span::styled("^S", Style::default().fg(Color::Green)),
                                Span::raw(": Save")
                            ]
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled("^X", Style::default().fg(Color::Green)),
                                Span::raw(": Save and Quit")
                            ]
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled("^C", Style::default().fg(Color::Green)),
                                Span::raw(": Quit")
                            ]
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled(" V", Style::default().fg(Color::Green)),
                                Span::raw(": Switch info view")
                            ]
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled(" H", Style::default().fg(Color::Green)),
                                Span::raw(": Help")
                            ]
                        ).left_aligned(),
                        Line::from(
                            vec![
                                Span::styled("Ok", Style::default().fg(Color::Black).bg(Color::White)),
                            ]
                        )
                    ]
                );
            }
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut ratatui::Terminal<B>) -> Result<(),Box<dyn std::error::Error>>
    {
        self.screen_size = (terminal.size()?.width, terminal.size()?.height);
        self.resize_if_needed(self.screen_size.0);

        while self.needs_to_exit == false 
        {
            if event::poll(self.poll_time)?
            {
                while event::poll(Duration::from_millis(0))?
                {
                    let event = event::read()?;
                    self.handle_event(event)?;
                }
            }

            terminal.hide_cursor()?;

            terminal.draw(|f| {
                self.screen_size = (f.size().width, f.size().height);
                let output_rect = Rect::new(0, f.size().height - 1, f.size().width, 1);
                let address_rect = Rect::new(0, 0, 17, f.size().height - output_rect.height);
                let hex_editor_rect = Rect::new(address_rect.width, 0, (self.block_size * 3 * self.blocks_per_row + self.blocks_per_row) as u16, f.size().height - output_rect.height);
                let mut info_view_rect = Rect::new(address_rect.width + hex_editor_rect.width, 0, (self.block_size * 2 * self.blocks_per_row + self.blocks_per_row) as u16 - 1, f.size().height - output_rect.height);

                let output_block = ratatui::widgets::Paragraph::new(Text::raw(&self.output))
                    .block(Block::default().borders(Borders::LEFT));
                let address_block = Paragraph::new(&self.address_view)
                    .block(Block::default().title("Address").borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM))
                    .scroll((self.scroll, 0));
                
                let editor_title = format!("Hex Editor{}", if self.dirty { " *"} else {""});

                let hex_editor_block = Paragraph::new(&self.hex_view)
                    .block(Block::default().title(editor_title).borders(Borders::LEFT | Borders::TOP | Borders::RIGHT | Borders::BOTTOM))
                    .scroll((self.scroll, 0));
                
                let info_view_block = 
                match &self.info_mode 
                {
                    InfoMode::Text =>
                    {
                        Paragraph::new(&self.text_view)
                            .block(Block::default().title("Text View").borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM))
                            .scroll((self.scroll, 0))
                    },
                    InfoMode::Assembly =>
                    {
                        info_view_rect.width = f.size().width - address_rect.width - hex_editor_rect.width - 2;
                        Paragraph::new(&self.assembly_view)
                            .block(Block::default().title("Assembly View").borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM))
                            .scroll((self.get_assembly_view_scroll() as u16, 0))
                    }
                };

                let scrollbar = ratatui::widgets::Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight)
                    .track_symbol(Some("█"))
                    .track_style(Style::default().fg(Color::DarkGray))
                    .begin_symbol(None)
                    .end_symbol(None);
                let mut scrollbar_state = ScrollbarState::new(self.hex_view.lines.len()).position(self.scroll as usize + self.cursor.1 as usize);

                f.render_widget(output_block, output_rect);
                f.render_widget(address_block, address_rect);
                f.render_widget(hex_editor_block, hex_editor_rect);
                f.render_widget(info_view_block, info_view_rect);
                f.render_stateful_widget(scrollbar, f.size(), &mut scrollbar_state);

                if let Some(popup_state) = &self.popup 
                {
                    let clear = ratatui::widgets::Clear::default();

                    let mut popup_text = Text::default();
                    let mut popup_title = "Popup";

                    let mut popup_rect = Rect::new(f.size().width / 2 - 27, f.size().height / 2 - 2, 54, 5);

                    Self::fill_popup(popup_state, f, &mut popup_title, &mut popup_text, &mut popup_rect);

                    let popup = ratatui::widgets::Paragraph::new(popup_text)
                        .block(Block::default().title(popup_title).borders(Borders::ALL))
                        .alignment(ratatui::layout::Alignment::Center);
                    f.render_widget(clear, popup_rect);
                    f.render_widget(popup, popup_rect);
                }

            })?;
            
            if self.popup.is_none()
            {
                terminal.set_cursor(self.cursor.0 + 18, self.cursor.1 + 1)?;
                terminal.show_cursor()?;   
            }
        }

        Ok(())
    }
}

