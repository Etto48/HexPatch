use super::{notification::NotificationLevel, App};

pub struct CursorPosition
{
    pub local_x: usize,
    pub local_byte_index: usize,
    pub block_index: usize,
    pub local_block_index: usize,
    pub line_index: usize,
    pub line_byte_index: usize,
    pub global_byte_index: usize,
    pub high_byte: bool,
}

impl CursorPosition
{
    pub fn get_high_byte_offset(&self) -> usize
    {
        match &self.high_byte {
            true => 0,
            false => 1,
        }
    }
}

impl <'a> App<'a>
{
    pub(super) fn get_cursor_position(&self) -> CursorPosition
    {
        if self.data.is_empty()
        {
            return CursorPosition {
                local_x: 0,
                local_byte_index: 0,
                block_index: 0,
                local_block_index: 0,
                line_index: 0,
                line_byte_index: 0,
                global_byte_index: 0,
                high_byte: false,
            };
        }
        let local_x = self.cursor.0 as usize % (self.block_size * 3 + 1);
        let high_byte = local_x % 3 == 0;
        let local_byte_index = local_x / 3;
        let block_index = self.cursor.0 as usize / (self.block_size * 3 + 1) + (self.scroll + self.cursor.1 as usize) * self.blocks_per_row;
        let local_block_index = block_index % self.blocks_per_row;
        let line_index = block_index / self.blocks_per_row;
        let line_byte_index = local_byte_index + self.block_size * local_block_index;
        let global_byte_index = line_byte_index + line_index * self.block_size * self.blocks_per_row;

        CursorPosition {
            local_x,
            local_byte_index,
            block_index,
            local_block_index,
            line_index,
            line_byte_index,
            global_byte_index,
            high_byte,
        }
    }

    pub(super) fn get_expected_cursor_position(&self, global_byte_index: usize, high_byte: bool) -> CursorPosition
    {
        let block_index = global_byte_index / self.block_size;
        let line_index = block_index / self.blocks_per_row;
        let local_block_index = block_index % self.blocks_per_row;
        let local_byte_index = global_byte_index % self.block_size;
        let local_x = (local_byte_index + local_block_index * self.block_size) * 3 + local_block_index;

        CursorPosition {
            local_x,
            local_byte_index,
            block_index,
            local_block_index,
            line_index,
            line_byte_index: local_byte_index + local_block_index * self.block_size,
            global_byte_index,
            high_byte,
        }
    }

    pub(super) fn jump_to_fuzzy_symbol(&mut self, symbol: &str, symbols: &Vec<(u64, String)>, scroll: usize)
    {
        if symbol.is_empty()
        {
            if let Some(symbols) = self.header.get_symbols()
            {
                if let Some(symbol) = symbols.iter().skip(scroll).next()
                {
                    let (address, name) = symbol;
                    let log_message = format!("Jumping to symbol {} at {:#X}", name, address);
                    self.jump_to(*address as usize, true);
                    self.log(NotificationLevel::Debug, &log_message);
                }
                else
                {
                    unreachable!("The scroll should not be greater than the number of symbols")
                }
            }
            else 
            {
                self.log(NotificationLevel::Error, "No symbols found");
            }
            return;
        }
        else if symbols.is_empty()
        {
            self.log(NotificationLevel::Error, "No symbols matching the search pattern found");
            return;
        }

        let mut find_iter = symbols.iter().skip(scroll);
        if let Some(symbol) = find_iter.next()
        {
            let (address, name) = symbol;
            self.log(NotificationLevel::Debug, &format!("Jumping to symbol {} at {:#X}", name, address));
            self.jump_to(*address as usize, true);
        }
        else 
        {
            unreachable!("The scroll should not be greater than the number of symbols");
        }
    }

    pub(super) fn jump_to_symbol(&mut self, symbol: &str)
    {
        if symbol.starts_with("0x")
        {
            if let Ok(address) = usize::from_str_radix(&symbol[2..], 16)
            {
                self.log(NotificationLevel::Debug, &format!("Jumping to address: {:#X}", address));
                self.jump_to(address, false);
            }
            else 
            {
                self.log(NotificationLevel::Error, &format!("Invalid address: {}", symbol));
            }
        }
        else if symbol.starts_with("v0x")
        {
            if let Ok(address) = usize::from_str_radix(&symbol[3..], 16)
            {
                self.log(NotificationLevel::Debug, &format!("Jumping to virtual address: {:#X}", address));
                self.jump_to(address, true);
            }
            else 
            {
                self.log(NotificationLevel::Error, &format!("Invalid virtual address: {}", symbol));
            }
        }
        else
        {
            if let Some(address) = self.header.symbol_to_address(symbol)
            {
                self.log(NotificationLevel::Debug, &format!("Jumping to symbol {} at {:#X}", symbol, address));
                self.jump_to(address as usize, true);
            }
            else if let Some(address) = self.header.get_sections().iter().find(|x|x.name == symbol).map(|x|x.address)
            {
                self.log(NotificationLevel::Debug, &format!("Jumping to section {} at {:#X}", symbol, address));
                self.jump_to(address as usize, false);
            }
            else 
            {
                self.log(NotificationLevel::Error, &format!("Symbol not found: {}", symbol));    
            }
        }
    }

    pub(super) fn jump_to(&mut self, mut address: usize, is_virtual: bool)
    {
        if is_virtual
        {
            if let Some(physical_address) = self.header.virtual_to_physical_address(address as u64)
            {
                address = physical_address as usize;
            }
            else 
            {
                self.log(NotificationLevel::Error, &format!("Virtual address {:#X} not found", address));
                return;
            }
        }
        if address >= self.data.len()
        {
            address = self.data.len().saturating_sub(1);
        }

        let expected_cursor_position = self.get_expected_cursor_position(address, false);
        let CursorPosition { local_x, line_index, .. } = expected_cursor_position;
        let y = line_index as isize - self.scroll as isize;

        if y < 0
        {
            self.scroll = line_index;
            self.cursor = (local_x as u16, 0);
        }
        else if y < self.screen_size.1 as isize - self.vertical_margin as isize
        {
            self.cursor = (local_x as u16, y as u16);
        }
        else
        {
            self.scroll = line_index - (self.screen_size.1 - self.vertical_margin - 1) as usize;
            self.cursor = (local_x as u16, (self.screen_size.1 - self.vertical_margin - 1) as u16);
        }
        self.update_cursors();
    }

    pub(super) fn move_cursor(&mut self, dx: isize, dy: isize)
    {
        // TODO: check that the cursor does not overflow the data
        let (x, y) = self.cursor;
        let mut x = x as isize + dx;
        let mut y = y as isize + dy;
        
        let view_size_y = self.screen_size.1 - self.vertical_margin;

        let viewed_block_size = (self.block_size * 3 + 1) as isize;
        let viewed_line_size = viewed_block_size * self.blocks_per_row as isize + self.blocks_per_row as isize - 3;

        let block_count = x / viewed_block_size;
        let local_x = x - block_count * viewed_block_size;
        if local_x % 3 == 2
        {
            x += dx.signum();
        }
        while x % viewed_block_size == viewed_block_size - 1 || x % viewed_block_size == viewed_block_size - 2
        {
            x += dx.signum();
        }

        if x < 0
        {
            if self.scroll > 0 || y > 0
            {
                x = viewed_line_size - self.blocks_per_row as isize;
                y -= 1;
            }
            else 
            {
                x = 0;
            }
        }
        else if x > viewed_line_size - self.blocks_per_row as isize
        {
            x = 0;
            y += 1;
        }
        if y >= (self.hex_view.lines.len() as isize - 1)
        {
            y = self.hex_view.lines.len() as isize - 1;
        }
        if y < 0
        {
            y = 0;
            if self.scroll > 0
            {
                self.scroll -= 1;
            }
        }
        else if y >= view_size_y as isize
        {
            y = view_size_y as isize - 1;
            if self.scroll < self.hex_view.lines.len() - view_size_y as usize
            {
                self.scroll += 1;
            }
        }

        let data_len = self.data.len() as isize;
        let bytes_per_row = self.block_size as isize * self.blocks_per_row as isize;
        let characters_in_last_row = (data_len % bytes_per_row) * 3 + (data_len % bytes_per_row) / self.block_size as isize - 2;
        if y + self.scroll as isize == data_len / bytes_per_row 
        {
            x = x.min(characters_in_last_row);
        }

        self.cursor = (x as u16, y as u16);

        self.update_cursors();
    }

    pub(super) fn move_cursor_page_up(&mut self)
    {
        if self.scroll == 0
        {
            self.cursor.1 = 0;
        }
        self.scroll = self.scroll.saturating_sub((self.screen_size.1 - self.vertical_margin) as usize);
        self.update_cursors();
    }

    pub(super) fn move_cursor_page_down(&mut self)
    {
        if self.scroll == self.hex_view.lines.len() - (self.screen_size.1 - self.vertical_margin) as usize
        {
            self.cursor.1 = self.screen_size.1 - self.vertical_margin - 1;
        }
        self.scroll = (self.scroll + (self.screen_size.1 - self.vertical_margin) as usize).min(self.hex_view.lines.len() - (self.screen_size.1 - self.vertical_margin) as usize);
        self.update_cursors();
    }

    pub(super) fn move_cursor_to_end(&mut self)
    {
        self.scroll = (self.hex_view.lines.len() as isize - (self.screen_size.1 as isize - self.vertical_margin as isize)).max(0) as usize;
        let x = self.blocks_per_row as u16 * 3 * self.block_size as u16 + self.blocks_per_row as u16 - 3;
        let y = (self.screen_size.1 - self.vertical_margin - 1).min(self.hex_view.lines.len() as u16 - 1);
        self.cursor = (x, y);
        self.update_cursors();
    }

    pub(super) fn move_cursor_to_start(&mut self)
    {
        self.cursor = (0, 0);
        self.scroll = 0;
        self.update_cursors();
    }

    pub(super) fn update_cursors(&mut self)
    {
        self.update_address_cursor();
        self.update_hex_cursor();
        self.update_text_cursor();
    }
}