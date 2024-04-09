use super::{notification::NotificationLevel, App};

pub struct CursorPosition
{
    pub cursor: (u16, u16),
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

impl App
{
    pub(super) fn get_cursor_position(&self) -> CursorPosition
    {
        if self.data.is_empty() || self.blocks_per_row == 0
        {
            return CursorPosition {
                cursor: (0, 0),
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
            cursor: self.cursor,
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
        let cursor_x = local_x as u16 + if high_byte { 0 } else { 1 };
        let cursor_y = (line_index - self.scroll) as u16;

        CursorPosition {
            cursor: (cursor_x, cursor_y),
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
            else if let Some(address) = self.header.get_sections().iter().find(|x|x.name == symbol).map(|x|x.file_offset)
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
    }

    pub(super) fn move_cursor(&mut self, dx: isize, dy: isize)
    {
        let current_position = self.get_cursor_position();
        let half_byte_delta = dx + (dy * self.block_size as isize * self.blocks_per_row as isize * 2);
        let half_byte_position = current_position.global_byte_index * 2 + if current_position.high_byte {0} else {1};

        let new_half_byte_position = (half_byte_position as isize).saturating_add(half_byte_delta);
        if new_half_byte_position < 0 || new_half_byte_position >= self.data.len() as isize * 2
        {
            return;
        }
        let new_global_byte_index = new_half_byte_position as usize / 2;
        let new_high_byte = new_half_byte_position % 2 == 0;
        self.log(NotificationLevel::Debug, &format!("Half byte delta: {}, new half byte position: {}, new global byte index: {}, new high byte: {}", half_byte_delta, new_half_byte_position, new_global_byte_index, new_high_byte));

        let new_selected_row = new_global_byte_index / (self.block_size * self.blocks_per_row);
        let min_visible_row = self.scroll;
        let max_visible_row = self.scroll + (self.screen_size.1 - self.vertical_margin) as usize - 1;
        let new_scroll = if new_selected_row < min_visible_row
        {
            new_selected_row
        }
        else if new_selected_row > max_visible_row
        {
            new_selected_row - (self.screen_size.1 - self.vertical_margin) as usize + 1
        }
        else
        {
            self.scroll
        };

        self.scroll = new_scroll;

        self.cursor = self.get_expected_cursor_position(new_global_byte_index, new_high_byte).cursor;
    }

    pub(super) fn move_cursor_page_up(&mut self)
    {
        if self.scroll == 0
        {
            self.cursor.1 = 0;
        }
        self.scroll = self.scroll.saturating_sub((self.screen_size.1 - self.vertical_margin) as usize);
    }

    pub(super) fn move_cursor_page_down(&mut self)
    {
        let hex_view_lines = self.get_hex_view_lines();
        if self.scroll == hex_view_lines - (self.screen_size.1 - self.vertical_margin) as usize
        {
            self.cursor.1 = self.screen_size.1 - self.vertical_margin - 1;
        }
        self.scroll = (self.scroll + (self.screen_size.1 - self.vertical_margin) as usize).min(hex_view_lines - (self.screen_size.1 - self.vertical_margin) as usize);
    }

    pub(super) fn get_hex_view_lines(&self) -> usize
    {
        if self.data.is_empty() || self.blocks_per_row == 0
        {
            return 0;
        }
        let hex_view_lines = self.data.len() / (self.block_size * self.blocks_per_row) + if self.data.len() % (self.block_size * self.blocks_per_row) == 0 { 0 } else { 1 };
        hex_view_lines
    }

    pub(super) fn move_cursor_to_end(&mut self)
    {
        let hex_view_lines = self.get_hex_view_lines();
        self.scroll = (hex_view_lines as isize - (self.screen_size.1 as isize - self.vertical_margin as isize)).max(0) as usize;
        let x = self.blocks_per_row as u16 * 3 * self.block_size as u16 + self.blocks_per_row as u16 - 3;
        let y = (self.screen_size.1 - self.vertical_margin - 1).min(hex_view_lines as u16 - 1);
        self.cursor = (x, y);
    }

    pub(super) fn move_cursor_to_start(&mut self)
    {
        self.cursor = (0, 0);
        self.scroll = 0;
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    #[test]
    fn test_move_cursor()
    {
        let data = vec![0; 0x100];
        let mut app = App::mockup(data);
        app.screen_size = (80, 24);
        app.resize_if_needed(80);
        
        app.move_cursor(1,0);
        assert_eq!(app.cursor,(1, 0));
        app.move_cursor(0,1);
        assert_eq!(app.cursor,(1, 1));

        app.move_cursor(0, 0);
        assert_eq!(app.cursor,(1, 1));

        app.move_cursor(0, -1);
        assert_eq!(app.cursor,(1, 0));
        app.move_cursor(-1, 0);
        assert_eq!(app.cursor,(0, 0));

        app.move_cursor(0, -1);
        assert_eq!(app.cursor,(0, 0));
        app.move_cursor(-1, 0);
        assert_eq!(app.cursor,(0, 0));

        let current_position = app.get_cursor_position();
        assert_eq!(current_position.global_byte_index, 0);
        assert_eq!(current_position.high_byte, true);

        app.move_cursor(81, 0);
        let current_position = app.get_cursor_position();
        assert_eq!(current_position.global_byte_index, 40);
        assert_eq!(current_position.high_byte, false);

        app.move_cursor(-1, -1);
        let bytes_per_line = app.block_size * app.blocks_per_row;
        let current_position = app.get_cursor_position();
        assert_eq!(current_position.global_byte_index, 40 - bytes_per_line);
        assert_eq!(current_position.high_byte, true);
    }
}