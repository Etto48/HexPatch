use super::App;

pub struct CursorPosition
{
    pub local_x: usize,
    pub local_byte_index: usize,
    pub block_index: usize,
    pub local_block_index: usize,
    pub line_index: usize,
    pub line_byte_index: usize,
    pub global_byte_index: usize,
}

impl <'a> App<'a>
{
    pub(super) fn get_cursor_position(&self) -> CursorPosition
    {
        let local_x = self.cursor.0 as usize % (self.block_size * 3 + 1);
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
        }
    }

    pub(super) fn move_cursor(&mut self, dx: i16, dy: i16)
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
        if y >= (self.hex_view.lines.len() as i16 - 1)
        {
            y = self.hex_view.lines.len() as i16 - 1;
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
            if self.scroll < self.hex_view.lines.len() - (self.screen_size.1 - 3) as usize
            {
                self.scroll += 1;
            }
        }

        self.cursor = (x as u16, y as u16);
        self.update_assembly_scroll();
    }

    pub(super) fn move_cursor_page_up(&mut self)
    {
        if self.scroll == 0
        {
            self.cursor.1 = 0;
        }
        self.scroll = self.scroll.saturating_sub((self.screen_size.1 - 3) as usize);
        self.update_assembly_scroll();
    }

    pub(super) fn move_cursor_page_down(&mut self)
    {
        if self.scroll == self.hex_view.lines.len() - (self.screen_size.1 - 3) as usize
        {
            self.cursor.1 = (self.hex_view.lines.len() % self.screen_size.1 as usize - 3) as u16;
        }
        self.scroll = (self.scroll + (self.screen_size.1 - 3) as usize).min(self.hex_view.lines.len() - (self.screen_size.1 - 3) as usize);
        self.update_assembly_scroll();
    }

    pub(super) fn move_cursor_to_end(&mut self)
    {
        self.scroll = self.hex_view.lines.len() - (self.screen_size.1 - 3) as usize;
        let x = self.blocks_per_row as u16 * 3 * self.block_size as u16 + self.blocks_per_row as u16 - 3;
        let y = self.screen_size.1 - 4;
        self.cursor = (x, y);
        self.update_assembly_scroll();
    }

    pub(super) fn move_cursor_to_start(&mut self)
    {
        self.cursor = (0, 0);
        self.scroll = 0;
        self.update_assembly_scroll();
    }
}