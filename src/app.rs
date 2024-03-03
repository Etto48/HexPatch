use std::{path::PathBuf, time::Duration};

use crossterm::event::{self, KeyCode};
use tui::{backend::Backend, layout::Rect, style::{Color, Modifier, Style}, text::{Span, Spans, Text}, widgets::Block};

pub struct App<'a> 
{
    path: PathBuf,
    output: String,
    address_view: Text<'a>,
    data: Text<'a>,
    text_view: Text<'a>,
    scroll: u16,
    cursor: (u16, u16),
    poll_time: Duration,
    needs_to_exit: bool,
    screen_size: (u16, u16),

    block_size: usize,
    blocks_per_row: usize,
}

impl <'a> App<'a>
{
    pub fn new(file_path: PathBuf) -> Result<Self,String>
    {
        let data = std::fs::read(&file_path).map_err(|e| e.to_string())?;
        let block_size = 8;
        let blocks_per_row = 3;
        Ok(App{
            path: file_path,
            output: "^C: quit, ^S: save, ^X: save and quit".to_string(),
            address_view: Self::addresses(data.len(), block_size, blocks_per_row),
            data: Self::bytes_to_styled_hex(&data, block_size, blocks_per_row),
            text_view: Self::bytes_to_styled_text(&data, block_size, blocks_per_row),
            scroll: 0,
            cursor: (0,0),
            poll_time: Duration::from_millis(1000),
            needs_to_exit: false,
            screen_size: (1,1),
            block_size,
            blocks_per_row,
        })
    }

    fn get_style_for_byte(byte: u8) -> Style
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

    fn bytes_to_styled_hex(bytes: &[u8], block_size: usize, blocks_per_row: usize) -> Text<'a>
    {
        let mut ret = Text::raw("");
        let mut current_line = Spans::default();
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
            current_line.0.push(span);

            if next_line
            {
                let new_line = std::mem::replace(&mut current_line, Spans::default());
                ret.lines.push(new_line);
            }
        }
        if current_line.0.len() > 0
        {
            ret.lines.push(current_line);
        }

        ret
    }

    fn bytes_to_styled_text(bytes: &'_[u8], block_size: usize, blocks_per_row: usize) -> Text<'a>
    {
        let mut ret = Text::raw("");
        let mut current_line = Spans::default();
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
            current_line.0.push(span);

            if next_line
            {
                let new_line = std::mem::replace(&mut current_line, Spans::default());
                ret.lines.push(new_line);
            }
        }
        if current_line.0.len() > 0
        {
            ret.lines.push(current_line);
        }

        ret
    }

    fn resize_if_needed(&mut self, width: u16)
    {
        let blocks_per_row = self.calc_blocks_per_row(width);
        if self.blocks_per_row != blocks_per_row
        {
            self.resize(blocks_per_row);
        }
    }

    fn resize(&mut self, blocks_per_row: usize)
    {
        self.blocks_per_row = blocks_per_row;
        let bin_data = self.data_to_vec_u8();
        self.address_view = Self::addresses(bin_data.len(), self.block_size, self.blocks_per_row);
        self.data = Self::bytes_to_styled_hex(&bin_data, self.block_size, self.blocks_per_row);
        self.text_view = Self::bytes_to_styled_text(&bin_data, self.block_size, self.blocks_per_row);
    }

    fn calc_blocks_per_row(&self, width: u16) -> usize
    {
        let block_characters_hex = self.block_size * 3 + 1;
        let block_characters_text = self.block_size * 2 + 1;
        let available_width = width - 18 - 2 - 2;
        let complessive_chars_per_block = block_characters_hex + block_characters_text;
        let blocks_per_row = (available_width + 2) / complessive_chars_per_block as u16;
        blocks_per_row as usize
    }

    fn u8_to_hex(input: u8) -> [char; 2]
    {
        let symbols = ['0','1','2','3','4','5','6','7','8','9','A','B','C','D','E','F'];
        let low = input & 0x0f;
        let high = (input & 0xf0) >> 4;
        [symbols[high as usize], symbols[low as usize]]
    }

    fn u8_to_char(input: u8) -> char
    {
        match input
        {
            0x20..=0x7E => input as char,
            0x0A => '⤶',
            _ => '.'
        }
    }

    fn addresses(size: usize, block_size: usize, blocks_per_row: usize) -> Text<'a>
    {
        let mut result = Text::raw("");

        for i in 0..=size/(block_size * blocks_per_row)
        {
            let mut spans = Spans::default();
            spans.0.push(Span::styled(format!("{:16X}", i * block_size * blocks_per_row), Style::default().fg(Color::DarkGray)));
            result.lines.push(spans);
        }
        result
    }

    fn move_cursor(&mut self, dx: i16, dy: i16)
    {
        let (x, y) = self.cursor;
        let mut x = x as i16 + dx;
        let mut y = y as i16 + dy;
        let block_count = x / (self.block_size as i16*3 + 1);
        let local_x = x - block_count * (self.block_size as i16 * 3 + 1);
        if local_x % 3 == 2
        {
            x += dx.signum();
        }
        while x % (self.block_size as i16 * 3 + 1) == (self.block_size as i16 * 3) || x % (self.block_size as i16 * 3 + 1) == (self.block_size as i16 * 3 - 1)
        {
            x += dx.signum();
        }
        
        if x < 0
        {
            x = (self.block_size * 3 * self.blocks_per_row + self.blocks_per_row) as i16 - 3;
            y -= 1;
        }
        else if x > (self.block_size * 3 * self.blocks_per_row + self.blocks_per_row) as i16 - 3
        {
            x = 0;
            y += 1;
        }
        if y >= (self.data.lines.len() as i16 - 1)
        {
            y = self.data.lines.len() as i16 - 1;
        }
        if y < 0
        {
            y = 0;
            if self.scroll > 0
            {
                self.scroll -= 1;
            }
        }
        else if y >= (self.screen_size.1 - 3) as i16
        {
            y = self.screen_size.1 as i16 - 4;
            if self.scroll < self.data.lines.len() as u16 - (self.screen_size.1 - 3)
            {
                self.scroll += 1;
            }
        }

        self.cursor = (x as u16, y as u16);

    }

    fn edit_data(&mut self, mut value: char)
    {
        value = value.to_uppercase().next().unwrap(); 

        if value >= '0' && value <= '9' || value >= 'A' && value <= 'F'
        {   
            let local_x = self.cursor.0 % (self.block_size as u16 * 3 + 1);
            let local_byte_index = local_x / 3;
            let block_index = self.cursor.0 / (self.block_size as u16 * 3 + 1) + (self.scroll + self.cursor.1) * self.blocks_per_row as u16;
            let local_block_index = block_index % self.blocks_per_row as u16;
            let line_index = block_index / self.blocks_per_row as u16;
            let line_byte_index = local_byte_index + self.block_size as u16 * local_block_index;

            let mut old_str = self.data.lines[line_index as usize].0[line_byte_index as usize].content.to_string();

            unsafe {
                old_str.as_bytes_mut()[(local_x % 3) as usize] = value as u8;
            }
            
            let hex = old_str.chars().filter(|c| c.is_whitespace() == false).collect::<String>();

            let byte = u8::from_str_radix(&hex, 16).unwrap();
            let style = Self::get_style_for_byte(byte);
            self.data.lines[line_index as usize].0[line_byte_index as usize] = Span::styled(old_str, style);
            
            let text = App::u8_to_char(byte);
            let old_str = self.text_view.lines[line_index as usize].0[line_byte_index as usize].content.to_string();
            let text_iterator = old_str.chars().filter(|c| c.is_whitespace());
            let mut new_str = text.to_string();
            new_str.extend(text_iterator);

            self.text_view.lines[line_index as usize].0[line_byte_index as usize] = Span::styled(new_str, style);
        }
    }

    fn data_to_vec_u8(&self) -> Vec<u8>
    {
        
        let mut output = Vec::with_capacity(self.data.lines.len() * self.block_size * self.blocks_per_row);
        for lines in &self.data.lines
        {
            for span in &lines.0
            {
                let mut hex = span.content.to_string();
                hex.retain(|c| c.is_whitespace() == false);
                let byte = u8::from_str_radix(&hex, 16).unwrap();
                output.push(byte);
            }
        }
        output
    }

    fn save_data(&mut self)
    {
        self.output = "Converting data...".to_string();
        let data = self.data_to_vec_u8();
        self.output = "Saving...".to_string();
        std::fs::write(&self.path, &data).unwrap();
        self.output = format!("Saved to {}", self.path.to_str().unwrap());
    }

    fn handle_event(&mut self, event: event::Event) -> Result<(),Box<dyn std::error::Error>>
    {
        match event
        {
            event::Event::Key(event) if event.kind == event::KeyEventKind::Press => {
                match event.code
                {
                    KeyCode::Up => {
                        self.move_cursor(0, -1);
                    },
                    KeyCode::Down => {
                        self.move_cursor(0, 1);
                    },
                    KeyCode::Left => {
                        self.move_cursor(-1, 0);
                    },
                    KeyCode::Right => {
                        self.move_cursor(1, 0);
                    },
                    KeyCode::Char(c) if event.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        match c
                        {
                            'c' => {
                                self.needs_to_exit = true;
                            },
                            's' => {
                                self.save_data();
                            },
                            'x' => {
                                self.needs_to_exit = true;
                                self.save_data();
                            }
                            _ => {}
                        }
                    },
                    KeyCode::Char(c) => {
                        self.edit_data(c);
                    },
                    _ => {}
                }
            },
            event::Event::Mouse(event) => {
                match event.kind
                {
                    event::MouseEventKind::ScrollUp => {
                        self.move_cursor(0, -1);
                    },
                    event::MouseEventKind::ScrollDown => {
                        self.move_cursor(0, 1);
                    },
                    event::MouseEventKind::ScrollLeft => {
                        self.move_cursor(-1, 0);
                    },
                    event::MouseEventKind::ScrollRight => {
                        self.move_cursor(1, 0);
                    },
                    _ => {}
                }
            },
            event::Event::Resize(width, _height) => {
                self.resize_if_needed(width);
            },
            _ => {}
        }

        Ok(())
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut tui::Terminal<B>) -> Result<(),Box<dyn std::error::Error>>
    {
        self.screen_size = (terminal.size()?.width, terminal.size()?.height);
        self.resize_if_needed(self.screen_size.0);

        while self.needs_to_exit == false {
            if event::poll(self.poll_time)?
            {
                while event::poll(Duration::from_millis(0))?
                {
                    let event = event::read()?;
                    self.handle_event(event)?;
                }
            }

            terminal.draw(|f| {
                self.screen_size = (f.size().width, f.size().height);
                let output_rect = Rect::new(0, f.size().height - 1, f.size().width, 1);
                let address_rect = Rect::new(0, 0, 18, f.size().height - output_rect.height);
                let hex_editor_rect = Rect::new(address_rect.width, 0, (self.block_size * 3 * self.blocks_per_row + self.blocks_per_row) as u16, f.size().height - output_rect.height);
                let text_view_rect = Rect::new(address_rect.width + hex_editor_rect.width, 0, (self.block_size * 2 * self.blocks_per_row + self.blocks_per_row) as u16, f.size().height - output_rect.height);
                let output_block = tui::widgets::Paragraph::new(Text::raw(&self.output))
                    .block(Block::default().borders(tui::widgets::Borders::LEFT));
                let address_block = tui::widgets::Paragraph::new(self.address_view.clone())
                    .block(Block::default().title("Address").borders(tui::widgets::Borders::ALL))
                    .scroll((self.scroll, 0));
                let hex_editor_block = tui::widgets::Paragraph::new(self.data.clone())
                    .block(Block::default().title("Hex Editor").borders(tui::widgets::Borders::ALL))
                    .scroll((self.scroll, 0));
                let text_view_block = tui::widgets::Paragraph::new(self.text_view.clone())
                    .block(Block::default().title("Text View").borders(tui::widgets::Borders::ALL))
                    .scroll((self.scroll, 0));
                f.render_widget(output_block, output_rect);
                f.render_widget(address_block, address_rect);
                f.render_widget(hex_editor_block, hex_editor_rect);
                f.render_widget(text_view_block, text_view_rect);
            })?;
            
            terminal.set_cursor(self.cursor.0 + 19, self.cursor.1 + 1)?;
            terminal.show_cursor()?;
        }

        Ok(())
    }
}

