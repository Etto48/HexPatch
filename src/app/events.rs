use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::Backend, Terminal};

use super::{popup_state::PopupState, settings::key_settings::KeySettings, App};

impl App
{
    fn handle_event_normal(&mut self, event: event::Event) -> Result<(), Box<dyn std::error::Error>>
    {
        match event
        {
            event::Event::Key(event) if event.kind == event::KeyEventKind::Press => {
                if event == self.settings.key.up
                {
                    self.move_cursor(0, -1, false);
                }
                else if event == self.settings.key.down 
                {
                    self.move_cursor(0, 1, false);
                }
                else if event == self.settings.key.left 
                {
                        self.move_cursor(-1, 0, false);
                }
                else if event == self.settings.key.right
                {
                    self.move_cursor(1, 0, false);
                }
                else if event == self.settings.key.next
                {
                    match self.info_mode
                    {
                        super::info_mode::InfoMode::Text => 
                        {
                            self.move_cursor(16, 0, true);
                        },
                        super::info_mode::InfoMode::Assembly => 
                        {
                            self.move_cursor_to_near_instruction(1);
                        },
                    }
                }
                else if event == self.settings.key.previous
                {
                    match self.info_mode
                    {
                        super::info_mode::InfoMode::Text => 
                        {
                            self.move_cursor(-16, 0, true);
                        },
                        super::info_mode::InfoMode::Assembly => 
                        {
                            self.move_cursor_to_near_instruction(-1);
                        },
                    }
                }
                else if event == self.settings.key.page_up
                {
                    self.move_cursor_page_up();
                }
                else if event == self.settings.key.page_down
                {
                    self.move_cursor_page_down();
                }
                else if event == self.settings.key.goto_start
                {
                    self.move_cursor_to_start();
                }
                else if event == self.settings.key.goto_end
                {
                    self.move_cursor_to_end();
                }
                else if event == self.settings.key.quit
                {
                    self.request_quit();
                }
                else if event == self.settings.key.save
                {
                    self.request_save();
                }
                else if event == self.settings.key.save_and_quit
                {
                    self.request_save_and_quit();
                }
                else if event == self.settings.key.open
                {
                    self.request_open()?;
                }
                else if event == self.settings.key.help
                {
                    self.request_popup_help();
                }
                else if event == self.settings.key.log
                {
                    self.request_popup_log();
                }
                else if event == self.settings.key.run
                {
                    self.request_popup_run();
                }
                else if event == self.settings.key.find_text
                {
                    self.request_popup_find_text();
                }
                else if event == self.settings.key.find_symbol
                {
                    self.request_popup_find_symbol();
                }
                else if event == self.settings.key.patch_text
                {
                    self.request_popup_text();
                }
                else if event == self.settings.key.patch_assembly
                {
                    self.request_popup_patch();
                }
                else if event == self.settings.key.jump
                {
                    self.request_popup_jump();
                }
                else if event == self.settings.key.change_view
                {
                    self.request_view_change();
                }
                else if let KeyCode::Char(c) = event.code
                {
                    match c
                    {
                        '0'..='9' | 'A'..='F' | 'a'..='f' => {
                            self.edit_data(c);
                        },
                        _ => {}
                    }
                }
            },
            event::Event::Mouse(event) => {
                match event.kind
                {
                    event::MouseEventKind::ScrollUp => {
                        self.move_cursor(0, -1, false);
                    },
                    event::MouseEventKind::ScrollDown => {
                        self.move_cursor(0, 1, false);
                    },
                    event::MouseEventKind::ScrollLeft => {
                        self.move_cursor(-1, 0, false);
                    },
                    event::MouseEventKind::ScrollRight => {
                        self.move_cursor(1, 0, false);
                    },
                    _ => {}
                }
            },
            event::Event::Resize(width, height) => {
                self.resize_to_size(width, height);
            },
            _ => {}
        }
        Ok(())
    }

    fn handle_string_edit(
        string: &mut String, 
        cursor: &mut usize, 
        event: &event::Event, 
        charset: Option<&str>, 
        max_len: Option<usize>, 
        multiline: bool,
        key_settings: &KeySettings
    ) -> Result<(), Box<dyn std::error::Error>>
    {
        match event
        {
            event::Event::Key(event) if event.kind == event::KeyEventKind::Press => {
                if *event == KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty())
                {
                    if *cursor > 0
                    {
                        string.remove(*cursor - 1);
                        *cursor -= 1;
                    }
                }
                else if *event == KeyEvent::new(KeyCode::Delete, KeyModifiers::empty())
                {
                    if *cursor < string.len()
                    {
                        string.remove(*cursor);
                    }
                }
                else if *event == KeyEvent::new(KeyCode::Left, KeyModifiers::empty())
                {
                    if *cursor > 0
                    {
                        *cursor -= 1;
                    }
                }
                else if *event == KeyEvent::new(KeyCode::Right, KeyModifiers::empty())
                {
                    if *cursor < string.len()
                    {
                        *cursor += 1;
                    }
                }
                else if *event == KeyEvent::new(KeyCode::Up, KeyModifiers::empty()) &&
                multiline
                {
                    let line = string.chars().rev().skip(string.len() - *cursor).take_while(|c| *c != '\n').count();
                    *cursor = cursor.saturating_sub(line + 1);
                }
                else if *event == KeyEvent::new(KeyCode::Down, KeyModifiers::empty()) &&
                multiline
                {
                    let line = string.chars().skip(*cursor).take_while(|c| *c != '\n').count();
                    *cursor = cursor.saturating_add(line + 1).min(string.len());
                }
                else if *event == KeyEvent::new(KeyCode::Home, KeyModifiers::empty())
                {
                    *cursor = 0;
                }
                else if *event == KeyEvent::new(KeyCode::End, KeyModifiers::empty())
                {
                    *cursor = string.len();
                }
                else if *event == key_settings.new_line &&
                multiline
                {
                    string.insert(*cursor, '\n');
                    *cursor += 1;
                }
                else if let KeyCode::Char(c) = event.code
                {
                    if (max_len.is_none() || string.len() < max_len.expect("Just checked")) &&
                        (charset.is_none() || charset.expect("Just checked").contains(c))
                    {
                        string.insert(*cursor, c);
                        *cursor += 1;
                    }
                }
            },
            _ => {}
        }
        Ok(())
    }

    fn handle_popup_scroll(scroll: &mut usize, len: usize, lines: Option<usize>, direction: i8)
    {
        if direction > 0
        {
            *scroll = (*scroll).saturating_add(1);
            if let Some(lines) = lines
            {
                if *scroll as isize >= len as isize - lines as isize
                {
                    *scroll = len.saturating_sub(lines);
                }
            }
            else if *scroll as isize >= len as isize
            {
                *scroll = len.saturating_sub(1);  
            }
            
        }
        else
        {
            *scroll = (*scroll).saturating_sub(1);
        }
    }

    fn handle_event_popup<B: Backend>(&mut self, event: event::Event, terminal: &mut Terminal<B>) -> Result<(), Box<dyn std::error::Error>>
    {
        let mut popup = self.popup.clone();
        match &mut popup
        {
            Some(PopupState::Open {currently_open_path, path, cursor, results, scroll: _scroll}) => 
            {
                let old_path = path.clone();
                Self::handle_string_edit(path, cursor, &event, None, None, false, &self.settings.key)?;
                if old_path != *path || results.is_empty()
                {
                    *results = Self::find_dir_contents(currently_open_path, path, &self.filesystem)?;
                }
            }
            Some(PopupState::Run {command, cursor, results, scroll: _scroll}) => 
            {
                let old_command = command.clone();
                Self::handle_string_edit(command, cursor, &event, None, None, false, &self.settings.key)?;
                if old_command != *command || results.is_empty()
                {
                    *results = self.find_commands(command);
                }
            }
            Some(PopupState::FindText {text, cursor}) =>
            {
                Self::handle_string_edit(text, cursor, &event, None, None, false, &self.settings.key)?;
            }
            Some(PopupState::FindSymbol {filter, symbols, cursor, scroll: _scroll}) =>
            {
                let old_filter = filter.clone();
                Self::handle_string_edit(filter, cursor, &event, None, None, false, &self.settings.key)?;
                if old_filter != *filter || symbols.is_empty()
                {
                    *symbols = self.find_symbols(filter);
                }
            }
            Some(PopupState::InsertText { text, cursor }) =>
            {
                Self::handle_string_edit(text, cursor, &event, None, None, true, &self.settings.key)?;
            }
            Some(PopupState::Patch {assembly, preview, cursor}) =>
            {
                Self::handle_string_edit(assembly, cursor, &event, None, None, true, &self.settings.key)?;
                if let Some(current_instruction) = self.get_current_instruction()
                {
                    *preview = self.bytes_from_assembly(assembly, current_instruction.virtual_address());
                }
            }
            Some(PopupState::JumpToAddress {location: address, cursor}) =>
            {
                Self::handle_string_edit(address, cursor, &event, None, None, false, &self.settings.key)?;
            }
            _ => {}
        }

        match event
        {
            event::Event::Key(event) if event.kind == event::KeyEventKind::Press => {
                if event == self.settings.key.left || event == self.settings.key.right
                {
                    match &mut popup
                    {
                        Some(PopupState::Save(yes_selected)) =>
                        {
                            *yes_selected = !*yes_selected;
                        }
                        Some(PopupState::SaveAndQuit(yes_selected)) =>
                        {
                            *yes_selected = !*yes_selected;
                        }
                        Some(PopupState::QuitDirtySave(yes_selected)) =>
                        {
                            *yes_selected = !*yes_selected;
                        },
                        _ => {}
                    }
                }
                else if event == self.settings.key.confirm
                {
                    match &mut popup
                    {
                        Some(PopupState::Open { currently_open_path, path, cursor: _cursor, results: _results, scroll }) =>
                        {
                            let mut new_popup = None;
                            self.go_to_path(currently_open_path, path, *scroll, &mut new_popup, terminal)?;
                            popup = new_popup;
                        }
                        Some(PopupState::Run { command, cursor: _cursor, results: _results, scroll }) =>
                        {
                            self.run_command(command, *scroll)?;
                            popup.clone_from(&self.popup);
                        }
                        Some(PopupState::FindText { text, cursor: _cursor}) =>
                        {
                            self.find_text(text);
                            // Maybe removing the popup is not a good idea, more testing needed
                            popup = None;
                        }
                        Some(PopupState::FindSymbol {filter, symbols, cursor: _cursor, scroll}) =>
                        {
                            self.jump_to_fuzzy_symbol(filter, symbols, *scroll);
                            popup = None;
                        }
                        Some(PopupState::Log(_)) =>
                        {
                            popup = None;
                        }
                        Some(PopupState::InsertText { text, cursor: _cursor }) =>
                        {
                            self.insert_text(text);
                            popup = None;
                        }
                        Some(PopupState::Patch {assembly, preview: _preview, cursor: _cursor}) =>
                        {
                            self.patch(assembly);
                            popup = None;
                        }
                        Some(PopupState::JumpToAddress {location, cursor: _cursor}) =>
                        {
                            self.jump_to_symbol(location);
                            popup = None;
                        }
                        Some(PopupState::Save(yes_selected)) =>
                        {
                            if *yes_selected
                            {
                                self.save_data()?;
                            }
                            popup = None;
                        },
                        Some(PopupState::SaveAndQuit(yes_selected)) =>
                        {
                            if *yes_selected
                            {
                                self.save_data()?;
                                self.needs_to_exit = true;
                            }
                            popup = None;
                        },
                        Some(PopupState::QuitDirtySave(yes_selected)) =>
                        {
                            if *yes_selected
                            {
                                self.save_data()?;
                                self.needs_to_exit = true;
                            }
                            else
                            {
                                self.needs_to_exit = true;
                            }
                            popup = None;
                        },
                        Some(PopupState::Help(_)) => 
                        {
                            popup = None;
                        }
                        None => {}
                    }
                }
                else if event == self.settings.key.down
                {
                    match &mut popup
                    {
                        Some(PopupState::Open { currently_open_path: _currently_open_path, path: _path, cursor: _cursor, results, scroll }) =>
                        {
                            Self::handle_popup_scroll(scroll, results.len(), None, 1);
                        }
                        Some(PopupState::Run { command: _command, cursor: _cursor, results, scroll }) =>
                        {
                            Self::handle_popup_scroll(scroll, results.len(), None, 1);
                        }
                        Some(PopupState::FindSymbol { filter: _filter, symbols, cursor: _cursor, scroll }) =>
                        {
                            if symbols.is_empty()
                            {
                                if let Some(symbols) = self.header.get_symbols()
                                {
                                    Self::handle_popup_scroll(scroll, symbols.len(), None, 1);
                                }
                                else
                                {
                                    *scroll = 0;
                                }
                            }
                            else
                            {
                                Self::handle_popup_scroll(scroll, symbols.len(), None, 1);
                            }
                        },
                        Some(PopupState::Log(scroll)) =>
                        {
                            Self::handle_popup_scroll(scroll, self.log.len(), Some(self.get_scrollable_popup_line_count()), -1);
                        }
                        Some(PopupState::Help(scroll)) =>
                        {
                            Self::handle_popup_scroll(scroll, self.help_list.len(), Some(self.get_scrollable_popup_line_count()), 1);
                        }
                        _ => {}
                    }
                }
                else if event == self.settings.key.up
                {
                    match &mut popup
                    {
                        Some(PopupState::Open { currently_open_path: _currently_open_path, path: _path, cursor: _cursor, results, scroll }) =>
                        {
                            Self::handle_popup_scroll(scroll, results.len(), None, -1);
                        }
                        Some(PopupState::Run { command: _command, cursor: _cursor, results: _results, scroll }) =>
                        {
                            Self::handle_popup_scroll(scroll, _results.len(), None, -1);
                        }
                        Some(PopupState::FindSymbol { filter: _filter, symbols, cursor: _cursor, scroll }) =>
                        {
                            Self::handle_popup_scroll(scroll, symbols.len(), None, -1);
                        },
                        Some(PopupState::Log(scroll)) =>
                        {
                            Self::handle_popup_scroll(scroll, self.log.len(), Some(self.get_scrollable_popup_line_count()), 1);
                        }
                        Some(PopupState::Help(scroll)) =>
                        {
                            Self::handle_popup_scroll(scroll, self.help_list.len(), Some(self.get_scrollable_popup_line_count()), -1);
                        }
                        _ => {}
                    }
                }
                else if event == self.settings.key.close_popup
                {
                    if self.filesystem.is_file(self.filesystem.pwd()) // if no file is open, close the program instead of the popup
                    {
                        popup = None;
                    }
                    else
                    {
                        self.needs_to_exit = true;    
                    }
                }
                else if event == self.settings.key.clear_log
                {
                    if let Some(PopupState::Log(scroll)) = &mut popup
                    {
                        *scroll = 0;
                        self.log.clear();
                    }
                }
                else if let KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Delete = event.code
                {
                    if event.modifiers.is_empty()
                    {
                        match &mut popup
                        {
                            Some(PopupState::Open { currently_open_path: _currently_open_path, path: _path, cursor: _cursor, results: _results, scroll }) => 
                            {
                                *scroll = 0;
                            }
                            Some(PopupState::Run { command: _command, cursor: _cursor, results: _results, scroll }) => 
                            {
                                *scroll = 0;
                            }
                            Some(PopupState::FindSymbol { filter: _, symbols: _, cursor: _, scroll }) => 
                            {
                                *scroll = 0;
                            }
                            _ => {}
                        }
                    }
                }
            },
            event::Event::Resize(width, height) =>
            {
                Self::resize_popup_if_needed(&mut popup);
                self.resize_to_size(width, height);
            },
            _ => {}
        }
        self.popup = popup;
        Ok(())
    }

    pub(super) fn handle_event<B: Backend>(&mut self, event: event::Event, terminal: &mut Terminal<B>) -> Result<(),Box<dyn std::error::Error>>
    {
        if self.popup.is_some()
        {
            self.handle_event_popup(event, terminal)?;
        }
        else 
        {
            self.handle_event_normal(event)?;
        }

        Ok(())
    }
}