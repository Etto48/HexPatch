use super::App;

pub struct CursorPosition
{
    pub local_x: u16,
    pub local_byte_index: u16,
    pub block_index: u16,
    pub local_block_index: u16,
    pub line_index: u16,
    pub line_byte_index: u16,
    pub global_byte_index: u16,
}

impl <'a> App<'a>
{
    pub(super) fn get_cursor_position(&self) -> CursorPosition
    {
        let local_x = self.cursor.0 % (self.block_size as u16 * 3 + 1);
        let local_byte_index = local_x / 3;
        let block_index = self.cursor.0 / (self.block_size as u16 * 3 + 1) + (self.scroll + self.cursor.1) * self.blocks_per_row as u16;
        let local_block_index = block_index % self.blocks_per_row as u16;
        let line_index = block_index / self.blocks_per_row as u16;
        let line_byte_index = local_byte_index + self.block_size as u16 * local_block_index;
        let global_byte_index = line_byte_index + line_index * self.block_size as u16 * self.blocks_per_row as u16;

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
            if self.scroll < self.hex_view.lines.len() as u16 - (self.screen_size.1 - 3)
            {
                self.scroll += 1;
            }
        }

        self.cursor = (x as u16, y as u16);
        self.update_assembly_scroll();
    }

    pub(super) fn move_cursor_to_end(&mut self)
    {
        self.scroll = self.hex_view.lines.len() as u16 - (self.screen_size.1 - 3);
        let x = self.blocks_per_row as u16 * 3 * self.block_size as u16 - 1;
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