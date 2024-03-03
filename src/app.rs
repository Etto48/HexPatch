use std::{path::PathBuf, time::Duration};

use crossterm::event::{self, KeyCode};
use tui::{backend::Backend, layout::Rect, text::Text, widgets::Block};

use crate::color::Color;

pub struct App 
{
    path: PathBuf,
    output: String,
    address_view: String,
    data: String,
    text_view: String,
    row_count: usize,
    scroll: u16,
    cursor: (u16, u16),
    poll_time: Duration,
    needs_to_exit: bool,
    screen_size: (u16, u16),

    block_size: usize,
    blocks_per_row: usize,
}

impl App
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
            data: Self::vec_u8_to_hex_string(&data, block_size, blocks_per_row),
            text_view: Self::vec_u8_to_string(&data, block_size, blocks_per_row),
            row_count: Self::row_count(data.len(), block_size, blocks_per_row),
            scroll: 0,
            cursor: (0,0),
            poll_time: Duration::from_millis(1000),
            needs_to_exit: false,
            screen_size: (1,1),
            block_size,
            blocks_per_row,
        })
    }

    pub fn byte_color(byte: u8) -> Color
    {
        match byte
        {
            // non-printable
            0..=31 | 127..=255 => Color::White,
            // printable
            32..=126 => Color::Green,
        }
    }

    pub fn row_count(data_len: usize, block_size: usize, blocks_per_row: usize) -> usize
    {
        data_len / (block_size * blocks_per_row) + 1
    }

    pub fn resize_if_needed(&mut self, width: u16)
    {
        let blocks_per_row = self.calc_blocks_per_row(width);
        if self.blocks_per_row != blocks_per_row
        {
            self.resize(blocks_per_row);
        }
    }

    pub fn resize(&mut self, blocks_per_row: usize)
    {
        self.blocks_per_row = blocks_per_row;
        let bin_data = self.data_to_vec_u8();
        self.address_view = Self::addresses(bin_data.len(), self.block_size, self.blocks_per_row);
        self.data = Self::vec_u8_to_hex_string(&bin_data, self.block_size, self.blocks_per_row);
        self.text_view = Self::vec_u8_to_string(&bin_data, self.block_size, self.blocks_per_row);
        self.row_count = Self::row_count(bin_data.len(), self.block_size, self.blocks_per_row);
    }

    pub fn calc_blocks_per_row(&self, width: u16) -> usize
    {
        let block_characters_hex = self.block_size * 3 + 1;
        let block_characters_text = self.block_size * 2 + 1;
        let available_width = width - 18 - 2 - 2;
        let complessive_chars_per_block = block_characters_hex + block_characters_text;
        let blocks_per_row = (available_width + 2) / complessive_chars_per_block as u16;
        blocks_per_row as usize
    }

    pub fn u8_to_hex(input: u8) -> [char; 2]
    {
        let symbols = ['0','1','2','3','4','5','6','7','8','9','A','B','C','D','E','F'];
        let low = input & 0x0f;
        let high = (input & 0xf0) >> 4;
        [symbols[high as usize], symbols[low as usize]]
    }

    pub fn u8_to_char(input: u8) -> char
    {
        if input >= 32 && input <= 126
        {
            input as char
        }
        else
        {
            '.'
        }
    }

    pub fn vec_u8_to_hex_string(input: &[u8], block_size: usize, blocks_per_row: usize) -> String
    {
        let mut result = String::new();

        let mut block_counter = 0;
        let mut i = 0;
        for byte in input
        {
            let hex = App::u8_to_hex(*byte);
            result.push(hex[0]);
            result.push(hex[1]);
            result.push(' ');
            i += 1;
            if i % block_size == 0
            {
                i = 0;
                result.push(' ');
                block_counter += 1;
                if block_counter == blocks_per_row
                {
                    block_counter = 0;
                    result.push('\n');
                }
                
            }
        }
        result
    }

    pub fn vec_u8_to_string(input: &[u8], block_size: usize, blocks_per_row: usize) -> String
    {
        let mut result = String::new();

        let mut block_counter = 0;
        let mut i = 0;
        for byte in input
        {
            let char = App::u8_to_char(*byte);
            result.push(char);
            result.push(' ');
            i += 1;
            if i % block_size == 0
            {
                i = 0;
                result.push(' ');
                block_counter += 1;
                if block_counter == blocks_per_row
                {
                    block_counter = 0;
                    result.push('\n');
                }
                
            }
        }
        result
    }

    pub fn addresses(size: usize, block_size: usize, blocks_per_row: usize) -> String
    {
        let mut result = String::with_capacity(size/(block_size * blocks_per_row) + 1);

        for i in 0..=size/(block_size * blocks_per_row)
        {
            result.push_str(format!("{:16X}\n", i * block_size * blocks_per_row).as_str());
        }
        result
    }

    pub fn move_cursor(&mut self, dx: i16, dy: i16)
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
            x = self.block_size as i16 * 3 * self.blocks_per_row as i16 - 1;
            y -= 1;
        }
        else if x >= (self.block_size * 3 * self.blocks_per_row) as i16
        {
            x = 0;
            y += 1;
        }
        if y >= (self.row_count as i16 - 1)
        {
            y = self.row_count as i16 - 1;
        }
        if y < 0
        {
            y = 0;
            if self.scroll > 0
            {
                self.scroll -= 1;
            }
        }
        else if y >= (self.screen_size.1 - 2) as i16
        {
            y = self.screen_size.1 as i16 - 3;
            if self.scroll < self.row_count as u16 - (self.screen_size.1 - 2)
            {
                self.scroll += 1;
            }
        }

        self.cursor = (x as u16, y as u16);

    }

    pub fn edit_data(&mut self, mut value: char)
    {
        value = value.to_uppercase().next().unwrap(); 

        if value >= '0' && value <= '9' || value >= 'A' && value <= 'F'
        {   
            let local_x = self.cursor.0 % (self.block_size as u16 * 3 + 1);
            let local_byte_index = local_x / 3;
            let block_index = self.cursor.0 / (self.block_size as u16 * 3 + 1) + (self.scroll + self.cursor.1) * self.blocks_per_row as u16;
            let line_index = block_index / self.blocks_per_row as u16;
            let byte_index = ((block_index * self.block_size as u16 + local_byte_index) * 3 + block_index + line_index) as usize;

            let byte_char_index = byte_index + (local_x % 3) as usize;

            if byte_char_index >= self.data.len()
            {
                return;
            }

            self.data.replace_range(byte_char_index..byte_char_index+1, value.to_string().as_str());

            let byte = u8::from_str_radix(&self.data[byte_index..byte_index+2], 16).unwrap();

            let text_index = ((block_index * self.block_size as u16 + local_byte_index) * 2 + block_index + line_index) as usize;
            
            let text = App::u8_to_char(byte);
            self.text_view.replace_range(text_index..text_index+1, text.to_string().as_str());
        }
    }

    pub fn data_to_vec_u8(&self) -> Vec<u8>
    {
        let mut char_index = 0;
        let mut output = Vec::with_capacity(self.data.len() / 2);
        if self.data.len() != 0
        {
            'outer_loop: loop {
                let byte = u8::from_str_radix(&self.data[char_index..char_index+2], 16).unwrap();
                output.push(byte);
                char_index += 2;
                while self.data.as_bytes()[char_index].is_ascii_whitespace()
                {
                    char_index += 1;
                    if char_index >= self.data.len()
                    {
                        break 'outer_loop;
                    }
                }
            }
        }
        output
    }

    pub fn save_data(&mut self)
    {
        self.output = "Converting data...".to_string();
        let data = self.data_to_vec_u8();
        self.output = "Saving...".to_string();
        std::fs::write(&self.path, &data).unwrap();
        self.output = format!("Saved to {}", self.path.to_str().unwrap());
    }

    pub fn handle_event(&mut self, event: event::Event) -> Result<(),Box<dyn std::error::Error>>
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
        self.resize_if_needed(terminal.size()?.width);

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
                let address_block = tui::widgets::Paragraph::new(Text::raw(&self.address_view))
                    .block(Block::default().title("Address").borders(tui::widgets::Borders::ALL))
                    .scroll((self.scroll, 0));
                let hex_editor_block = tui::widgets::Paragraph::new(Text::raw(&self.data))
                    .block(Block::default().title("Hex Editor").borders(tui::widgets::Borders::ALL))
                    .scroll((self.scroll, 0));
                let text_view_block = tui::widgets::Paragraph::new(Text::raw(&self.text_view))
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

