use super::{data::Data, info_mode::InfoMode, log::NotificationLevel, pane::Pane, App};

pub struct CursorPosition {
    pub cursor: Option<(u16, u16)>,
    pub local_x: usize,
    pub local_byte_index: usize,
    pub block_index: usize,
    pub local_block_index: usize,
    pub line_index: usize,
    pub line_byte_index: usize,
    pub global_byte_index: usize,
    pub high_byte: bool,
}

impl CursorPosition {
    pub fn get_high_byte_offset(&self) -> usize {
        match &self.high_byte {
            true => 0,
            false => 1,
        }
    }
}

impl App {
    pub(super) fn get_cursor_position_no_self(
        data: &Data,
        blocks_per_row: usize,
        block_size: usize,
        cursor: (u16, u16),
        scroll: usize,
    ) -> CursorPosition {
        if data.is_empty() || blocks_per_row == 0 {
            return CursorPosition {
                cursor: Some((0, 0)),
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
        let local_x = cursor.0 as usize % (block_size * 3 + 1);
        let high_byte = local_x.is_multiple_of(3);
        let local_byte_index = local_x / 3;
        let block_index = cursor.0 as usize / (block_size * 3 + 1)
            + (scroll + cursor.1 as usize) * blocks_per_row;
        let local_block_index = block_index % blocks_per_row;
        let line_index = block_index / blocks_per_row;
        let line_byte_index = local_byte_index + block_size * local_block_index;
        let global_byte_index = line_byte_index + line_index * block_size * blocks_per_row;

        CursorPosition {
            cursor: Some(cursor),
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
    pub(super) fn get_cursor_position(&self) -> CursorPosition {
        Self::get_cursor_position_no_self(
            &self.data,
            self.blocks_per_row,
            self.block_size,
            self.cursor,
            self.scroll,
        )
    }

    pub(super) fn get_expected_cursor_position_no_self(
        global_byte_index: usize,
        high_byte: bool,
        block_size: usize,
        blocks_per_row: usize,
        screen_size: (u16, u16),
        vertical_margin: u16,
        scroll: usize,
    ) -> CursorPosition {
        let block_index = global_byte_index / block_size;
        let line_index = block_index / blocks_per_row;
        let local_block_index = block_index % blocks_per_row;
        let local_byte_index = global_byte_index % block_size;
        let local_x = (local_byte_index + local_block_index * block_size) * 3 + local_block_index;
        let cursor_x = local_x as u16 + if high_byte { 0 } else { 1 };
        let cursor_y = line_index as isize - scroll as isize;
        let cursor =
            if cursor_y < 0 || cursor_y >= screen_size.1 as isize - vertical_margin as isize {
                None
            } else {
                Some((cursor_x, cursor_y as u16))
            };

        CursorPosition {
            cursor,
            local_x,
            local_byte_index,
            block_index,
            local_block_index,
            line_index,
            line_byte_index: local_byte_index + local_block_index * block_size,
            global_byte_index,
            high_byte,
        }
    }

    pub(super) fn get_expected_cursor_position(
        &self,
        global_byte_index: usize,
        high_byte: bool,
    ) -> CursorPosition {
        Self::get_expected_cursor_position_no_self(
            global_byte_index,
            high_byte,
            self.block_size,
            self.blocks_per_row,
            self.screen_size,
            self.vertical_margin,
            self.scroll,
        )
    }

    pub(super) fn jump_to_fuzzy_symbol(
        &mut self,
        symbol: &str,
        symbols: &[(u64, String)],
        scroll: usize,
    ) {
        if symbol.is_empty() {
            if let Some(symbols) = self.header.get_symbols() {
                if let Some(symbol) = symbols.iter().nth(scroll) {
                    let (address, name) = symbol;
                    let log_message =
                        t!("app.messages.jump_to_symbol", symbol = name, address = address : {:#X});
                    self.jump_to(*address as usize, true);
                    self.log(NotificationLevel::Debug, log_message);
                } else {
                    unreachable!("The scroll should not be greater than the number of symbols")
                }
            } else {
                self.log(NotificationLevel::Error, t!("errors.no_symbols_found"));
            }
            return;
        } else if symbols.is_empty() {
            self.log(NotificationLevel::Error, t!("errors.no_matching_symbols"));
            return;
        }

        let mut find_iter = symbols.iter().skip(scroll);
        if let Some(symbol) = find_iter.next() {
            let (address, name) = symbol;
            self.log(
                NotificationLevel::Debug,
                t!("app.messages.jump_to_symbol", symbol = name, address = address : {:#X}),
            );
            self.jump_to(*address as usize, true);
        } else {
            unreachable!("The scroll should not be greater than the number of symbols");
        }
    }

    pub(super) fn jump_to_fuzzy_comment(
        &mut self,
        comment: &str,
        comments: &[(u64, String)],
        scroll: usize,
    ) {
        if comment.is_empty() {
            if let Some(comment) = self.comments.iter().nth(scroll) {
                let (address, _comment) = comment;
                let log_message = t!("app.messages.jump_to_comment", address = address : {:#X});
                self.jump_to(*address as usize, false);
                self.log(NotificationLevel::Debug, log_message);
            } else {
                unreachable!("The scroll should not be greater than the number of comments")
            }
            return;
        } else if comments.is_empty() {
            self.log(NotificationLevel::Error, t!("errors.no_matching_comments"));
            return;
        }

        let mut find_iter = comments.iter().skip(scroll);
        if let Some(comment) = find_iter.next() {
            let (address, _comment) = comment;
            self.log(
                NotificationLevel::Debug,
                t!("app.messages.jump_to_comment", address = address : {:#X}),
            );
            self.jump_to(*address as usize, false);
        } else {
            unreachable!("The scroll should not be greater than the number of comments");
        }
    }

    pub(super) fn jump_to_symbol(&mut self, symbol: &str) {
        if let Some(address) = symbol.strip_prefix("0x") {
            if let Ok(address) = usize::from_str_radix(address, 16) {
                self.log(
                    NotificationLevel::Debug,
                    t!("app.messages.jump_to_address", address = address : {:#X}),
                );
                self.jump_to(address, false);
            } else {
                self.log(
                    NotificationLevel::Error,
                    t!("errors.invalid_address", address = symbol),
                );
            }
        } else if let Some(address) = symbol.strip_prefix("v0x") {
            if let Ok(address) = usize::from_str_radix(address, 16) {
                self.log(
                    NotificationLevel::Debug,
                    t!("app.messages.jump_to_virtual_address", address = address : {:#X}),
                );
                self.jump_to(address, true);
            } else {
                self.log(
                    NotificationLevel::Error,
                    t!("errors.invalid_virtual_address", address = symbol),
                );
            }
        } else if let Some(address) = self.header.symbol_to_address(symbol) {
            self.log(
                NotificationLevel::Debug,
                t!("app.messages.jump_to_symbol", symbol = symbol, address = address : {:#X}),
            );
            self.jump_to(address as usize, true);
        } else if let Some(address) = self
            .header
            .get_sections()
            .iter()
            .find(|x| x.name == symbol)
            .map(|x| x.file_offset)
        {
            self.log(
                NotificationLevel::Debug,
                t!("app.messages.jump_to_section", section = symbol, address = address : {:#X}),
            );
            self.jump_to(address as usize, false);
        } else {
            self.log(
                NotificationLevel::Error,
                t!("errors.symbol_not_found", symbol = symbol),
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn jump_to_no_self(
        mut address: usize,
        data: &Data,
        screen_size: (u16, u16),
        vertical_margin: u16,
        scroll: &mut usize,
        cursor: &mut (u16, u16),
        block_size: usize,
        blocks_per_row: usize,
    ) {
        if address >= data.len() {
            address = data.len().saturating_sub(1);
        }
        if screen_size.1 <= vertical_margin {
            *scroll = 0;
            *cursor = (0, 0);
            return;
        }

        let expected_cursor_position = Self::get_expected_cursor_position_no_self(
            address,
            false,
            block_size,
            blocks_per_row,
            screen_size,
            vertical_margin,
            *scroll,
        );
        let CursorPosition {
            local_x,
            line_index,
            ..
        } = expected_cursor_position;
        let y = line_index as isize - *scroll as isize;

        if y < 0 {
            *scroll = line_index;
            *cursor = (local_x as u16, 0);
        } else if y < screen_size.1 as isize - vertical_margin as isize {
            *cursor = (local_x as u16, y as u16);
        } else {
            *scroll = line_index - (screen_size.1 - vertical_margin - 1) as usize;
            *cursor = (local_x as u16, (screen_size.1 - vertical_margin - 1));
        }
    }

    pub(super) fn jump_to(&mut self, mut address: usize, is_virtual: bool) {
        if is_virtual {
            if let Some(physical_address) = self.header.virtual_to_physical_address(address as u64)
            {
                address = physical_address as usize;
            } else {
                self.log(
                    NotificationLevel::Error,
                    t!("errors.virtual_address_not_found", address = address : {:#X}),
                );
                return;
            }
        }
        Self::jump_to_no_self(
            address,
            &self.data,
            self.screen_size,
            self.vertical_margin,
            &mut self.scroll,
            &mut self.cursor,
            self.block_size,
            self.blocks_per_row,
        )
    }

    pub(super) fn move_cursor_in_selected_panel(&mut self, dx: isize, dy: isize) {
        match self.selected_pane {
            Pane::Hex => self.move_cursor(dx, dy, false),
            Pane::View => match self.info_mode {
                InfoMode::Text => self.move_cursor(dx * 2, dy, false),
                InfoMode::Assembly => self.move_cursor_to_near_instruction(dy),
            },
        }
    }

    pub(super) fn move_cursor(&mut self, dx: isize, dy: isize, best_effort: bool) {
        if self.screen_size.1 <= self.vertical_margin {
            return;
        }
        let current_position = self.get_cursor_position();
        let half_byte_delta =
            dx + (dy * self.block_size as isize * self.blocks_per_row as isize * 2);
        let half_byte_position =
            current_position.global_byte_index * 2 + if current_position.high_byte { 0 } else { 1 };

        let mut new_half_byte_position =
            (half_byte_position as isize).saturating_add(half_byte_delta);
        if !best_effort
            && (new_half_byte_position < 0
                || new_half_byte_position >= self.data.len() as isize * 2)
        {
            return;
        } else if best_effort {
            let max_half_byte_position = (self.data.len() as isize * 2 - 1).max(0);
            new_half_byte_position = new_half_byte_position.clamp(0, max_half_byte_position);
        }
        let new_global_byte_index = new_half_byte_position as usize / 2;
        let new_high_byte = new_half_byte_position % 2 == 0;

        let new_selected_row = new_global_byte_index / (self.block_size * self.blocks_per_row);
        let min_visible_row = self.scroll;
        let max_visible_row =
            self.scroll + (self.screen_size.1 - self.vertical_margin) as usize - 1;
        let new_scroll = if new_selected_row < min_visible_row {
            new_selected_row
        } else if new_selected_row > max_visible_row {
            new_selected_row - (self.screen_size.1 - self.vertical_margin) as usize + 1
        } else {
            self.scroll
        };

        self.scroll = new_scroll;

        self.cursor = self
            .get_expected_cursor_position(new_global_byte_index, new_high_byte)
            .cursor
            .expect(&t!("errors.cursor_position"));
    }

    pub(super) fn move_cursor_page_up(&mut self) {
        let screen_size_y = self.screen_size.1 as isize - self.vertical_margin as isize;
        if screen_size_y > 0 {
            self.move_cursor(0, -screen_size_y, true);
        }
    }

    pub(super) fn move_cursor_page_down(&mut self) {
        let screen_size_y = self.screen_size.1 as isize - self.vertical_margin as isize;
        if screen_size_y > 0 {
            self.move_cursor(0, screen_size_y, true);
        }
    }

    pub(super) fn move_cursor_to_end(&mut self) {
        let bytes = self.data.len();

        self.move_cursor(bytes as isize * 2, 0, true);
    }

    pub(super) fn move_cursor_to_start(&mut self) {
        let bytes = self.data.len();
        self.move_cursor(bytes as isize * -2, 0, true);
    }

    pub(super) fn move_cursor_to_near_instruction(&mut self, instruction_count: isize) {
        let current_offset = self.get_cursor_position().global_byte_index;
        if current_offset >= self.assembly_offsets.len() {
            return;
        }
        let current_instruction_index = self.assembly_offsets[current_offset];
        let mut next_instruction_index = (current_instruction_index as isize + instruction_count)
            .clamp(
                0,
                self.assembly_instructions.len().saturating_sub(1) as isize,
            ) as usize;
        while instruction_count < 0
            && next_instruction_index != 0
            && next_instruction_index != current_instruction_index
            && self.assembly_instructions[next_instruction_index].file_address()
                == self.assembly_instructions[current_instruction_index].file_address()
        {
            next_instruction_index = next_instruction_index.saturating_sub(1);
        }

        let target_address = self.assembly_instructions[next_instruction_index].file_address();
        self.jump_to(target_address as usize, false);
    }

    pub(super) fn switch_selected_pane(&mut self) {
        match self.selected_pane {
            Pane::Hex => self.selected_pane = Pane::View,
            Pane::View => self.selected_pane = Pane::Hex,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_move_cursor() {
        let data = vec![0; 0x100];
        let mut app = App::mockup(data);
        app.resize_to_size(80, 24);

        app.move_cursor(1, 0, false);
        assert_eq!(app.cursor, (1, 0));
        app.move_cursor(0, 1, false);
        assert_eq!(app.cursor, (1, 1));

        app.move_cursor(0, 0, false);
        assert_eq!(app.cursor, (1, 1));

        app.move_cursor(0, -1, false);
        assert_eq!(app.cursor, (1, 0));
        app.move_cursor(-1, 0, false);
        assert_eq!(app.cursor, (0, 0));

        app.move_cursor(0, -1, false);
        assert_eq!(app.cursor, (0, 0));
        app.move_cursor(-1, 0, false);
        assert_eq!(app.cursor, (0, 0));

        let current_position = app.get_cursor_position();
        assert_eq!(current_position.global_byte_index, 0);
        assert!(current_position.high_byte);

        app.move_cursor(81, 0, false);
        let current_position = app.get_cursor_position();
        assert_eq!(current_position.global_byte_index, 40);
        assert!(!current_position.high_byte);

        app.move_cursor(-1, -1, false);
        let bytes_per_line = app.block_size * app.blocks_per_row;
        let current_position = app.get_cursor_position();
        assert_eq!(current_position.global_byte_index, 40 - bytes_per_line);
        assert!(current_position.high_byte);
    }

    #[test]
    fn test_move_with_no_screen() {
        let data = vec![0; 0x100];
        let mut app = App::mockup(data);
        app.resize_to_size(0, 0);

        app.move_cursor(1, 0, false);
        assert_eq!(app.cursor, (0, 0));
        app.move_cursor(0, 1, false);
        assert_eq!(app.cursor, (0, 0));
        app.move_cursor(0, -1, false);
        assert_eq!(app.cursor, (0, 0));
        app.move_cursor(-1, 0, false);
        assert_eq!(app.cursor, (0, 0));
    }

    #[test]
    fn test_move_with_small_screen() {
        let data = vec![0; 0x100];
        let mut app = App::mockup(data);
        app.resize_to_size(1, 1);

        app.move_cursor(1, 0, false);
        assert_eq!(app.cursor, (0, 0));
        app.move_cursor(0, 1, false);
        assert_eq!(app.cursor, (0, 0));
        app.move_cursor(0, -1, false);
        assert_eq!(app.cursor, (0, 0));
        app.move_cursor(-1, 0, false);
        assert_eq!(app.cursor, (0, 0));
    }

    #[test]
    fn test_move_to_near_instruction() {
        let data = vec![
            0x90, 0x90, 0x90, 0x48, 0x89, 0xd8, 0xeb, 0xfe, 0x90, 0x90, 0x90,
        ];
        let mut app = App::mockup(data);
        app.resize_to_size(80, 24);

        app.move_cursor_to_near_instruction(1);
        let current_position = app.get_cursor_position().global_byte_index;
        assert_eq!(current_position, 1);

        app.move_cursor_to_near_instruction(2);
        let current_position = app.get_cursor_position().global_byte_index;
        assert_eq!(current_position, 3);

        app.move_cursor_to_near_instruction(-1);
        let current_position = app.get_cursor_position().global_byte_index;
        assert_eq!(current_position, 2);

        app.move_cursor_to_near_instruction(4);
        let current_position = app.get_cursor_position().global_byte_index;
        assert_eq!(current_position, 9);

        app.move_cursor_to_near_instruction(2);
        let current_position = app.get_cursor_position().global_byte_index;
        assert_eq!(current_position, 10);

        app.move_cursor_to_near_instruction(100);
        let current_position = app.get_cursor_position().global_byte_index;
        assert_eq!(current_position, 10);

        app.move_cursor_to_near_instruction(-100);
        let current_position = app.get_cursor_position().global_byte_index;
        assert_eq!(current_position, 0);
    }
}
