use crossterm::event::{self, KeyCode, KeyModifiers};

use super::{popup_state::PopupState, App};

impl <'a> App<'a>
{
    fn handle_event_normal(&mut self, event: event::Event) -> Result<(), Box<dyn std::error::Error>>
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
                    KeyCode::PageUp => {
                        self.move_cursor_page_up();
                    },
                    KeyCode::PageDown => {
                        self.move_cursor_page_down();
                    },
                    KeyCode::Home => 
                    {
                        self.move_cursor_to_start();
                    }
                    KeyCode::End => 
                    {
                        self.move_cursor_to_end();
                    }
                    KeyCode::Char(c) if event.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        match c
                        {
                            'c' => {
                                self.request_quit();
                            },
                            's' => {
                                self.request_save();
                            },
                            'x' => {
                                self.request_quit_without_save();
                            },
                            'o' => {
                                self.request_open()?;
                            },
                            _ => {}
                        }
                    },
                    KeyCode::Char(c) => {
                        match c
                        {
                            '0'..='9' | 'A'..='F' | 'a'..='f' => {
                                self.edit_data(c);
                            },
                            'h' => {
                                self.request_popup_help();
                            },
                            'l' => {
                                self.request_popup_log();
                            },
                            ' ' => {
                                self.request_popup_run();
                            }
                            's' => {
                                self.request_popup_find_symbol();
                            },
                            'p' => {
                                self.request_popup_patch();
                            },
                            'j' => {
                                self.request_popup_jump();
                            },
                            'v' => {
                                self.request_view_change();
                            }
                            _ => {}
                        }
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

    fn handle_string_edit(
        string: &mut String, 
        cursor: &mut usize, 
        event: &event::Event, 
        charset: Option<&str>, 
        capitalize: bool, 
        max_len: Option<usize>, 
        multiline: bool
    ) -> Result<(), Box<dyn std::error::Error>>
    {
        match event
        {
            event::Event::Key(event) if event.kind == event::KeyEventKind::Press => {
                match event.code
                {
                    KeyCode::Backspace => {
                        if *cursor > 0
                        {
                            string.remove(*cursor - 1);
                            *cursor -= 1;
                        }
                    },
                    KeyCode::Delete => {
                        if *cursor < string.len()
                        {
                            string.remove(*cursor);
                        }
                    },
                    KeyCode::Left => {
                        if *cursor > 0
                        {
                            *cursor -= 1;
                        }
                    },
                    KeyCode::Right => {
                        if *cursor < string.len()
                        {
                            *cursor += 1;
                        }
                    },
                    KeyCode::Up if multiline => {
                        let line = string.chars().rev().skip(string.len() - *cursor).take_while(|c| *c != '\n').count();
                        *cursor = cursor.saturating_sub(line + 1);
                    },
                    KeyCode::Down if multiline => {
                        let line = string.chars().skip(*cursor).take_while(|c| *c != '\n').count();
                        *cursor = cursor.saturating_add(line + 1).min(string.len());
                    },
                    KeyCode::Char(mut c) => {
                        if capitalize
                        {
                            c = c.to_ascii_uppercase();
                        }
                        if (max_len == None || string.len() < max_len.expect("Just checked")) &&
                            (charset.is_none() || charset.expect("Just checked").contains(c))
                        {
                            string.insert(*cursor, c);
                            *cursor += 1;
                        }
                    },
                    KeyCode::End => {
                        *cursor = string.len();
                    },
                    KeyCode::Home => {
                        *cursor = 0;
                    },
                    KeyCode::Enter if multiline && event.modifiers.contains(KeyModifiers::SHIFT) => {
                        string.insert(*cursor, '\n');
                        *cursor += 1;
                    },
                    _ => {}
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
            else 
            {
                if *scroll as isize >= len as isize
                {
                    *scroll = len.saturating_sub(1);
                }    
            }
            
        }
        else
        {
            *scroll = (*scroll).saturating_sub(1);
        }
    }

    fn handle_event_popup(&mut self, event: event::Event) -> Result<(), Box<dyn std::error::Error>>
    {
        let mut popup = self.popup.clone();
        match &mut popup
        {
            Some(PopupState::Open {currently_open_path, path, cursor, results, scroll: _scroll}) => 
            {
                Self::handle_string_edit(path, cursor, &event, None, false, None, false)?;
                *results = Self::find_dir_contents(&currently_open_path, &path)?;
            }
            Some(PopupState::Run {command, cursor, results, scroll: _scroll}) => 
            {
                Self::handle_string_edit(command, cursor, &event, None, false, None, false)?;
                *results = self.find_commands(command);
            }
            Some(PopupState::FindSymbol {filter, symbols, cursor, scroll: _scroll}) =>
            {
                Self::handle_string_edit(filter, cursor, &event, None, false, None, false)?;
                *symbols = self.find_symbols(filter);
            }
            Some(PopupState::Patch {assembly, preview, cursor}) =>
            {
                Self::handle_string_edit(assembly, cursor, &event, None, false, None, true)?;
                *preview = self.bytes_from_assembly(&assembly, self.get_current_instruction().virtual_ip());
            }
            Some(PopupState::JumpToAddress {location: address, cursor}) =>
            {
                Self::handle_string_edit(address, cursor, &event, None, false, None, false)?;
            }
            _ => {}
        }

        match event
        {
            event::Event::Key(event) if event.kind == event::KeyEventKind::Press => {
                match event.code
                {
                    KeyCode::Left |
                    KeyCode::Right => {
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
                    },
                    KeyCode::Enter if !event.modifiers.contains(KeyModifiers::SHIFT) => {
                        match &mut popup
                        {
                            Some(PopupState::Open { currently_open_path, path, cursor: _cursor, results: _results, scroll }) =>
                            {
                                let mut new_popup = None;
                                self.go_to_path(&currently_open_path, &path, *scroll, &mut new_popup)?;
                                popup = new_popup;
                            }
                            Some(PopupState::Run { command, cursor: _cursor, results: _results, scroll }) =>
                            {
                                self.run_command(command, *scroll)?;
                                popup = self.popup.clone();
                            }
                            Some(PopupState::FindSymbol {filter, symbols, cursor: _cursor, scroll}) =>
                            {
                                self.jump_to_fuzzy_symbol(&filter, &symbols, *scroll);
                                popup = None;
                            }
                            Some(PopupState::Log(_)) =>
                            {
                                popup = None;
                            }
                            Some(PopupState::Patch {assembly, preview: _preview, cursor: _cursor}) =>
                            {
                                self.patch(&assembly);
                                popup = None;
                            }
                            Some(PopupState::JumpToAddress {location, cursor: _cursor}) =>
                            {
                                self.jump_to_symbol(&location);
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
                    },
                    KeyCode::Down => {
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
                                Self::handle_popup_scroll(scroll, self.log.len(), Some(self.get_scrollable_popup_line_count()?), -1);
                            }
                            Some(PopupState::Help(scroll)) =>
                            {
                                Self::handle_popup_scroll(scroll, self.help_list.len(), Some(self.get_scrollable_popup_line_count()?), 1);
                            }
                            _ => {}
                        }
                    },
                    KeyCode::Up => {
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
                                Self::handle_popup_scroll(scroll, self.log.len(), Some(self.get_scrollable_popup_line_count()?), 1);
                            }
                            Some(PopupState::Help(scroll)) =>
                            {
                                Self::handle_popup_scroll(scroll, self.help_list.len(), Some(self.get_scrollable_popup_line_count()?), -1);
                            }
                            _ => {}
                        }
                    },
                    KeyCode::Esc => {
                        if !self.path.is_dir() // Prevent closing the open file popup if no file is open
                        {
                            popup = None;
                        }
                        else 
                        {
                            self.needs_to_exit = true;    
                        }
                    },
                    KeyCode::Char(_) | 
                    KeyCode::Backspace | 
                    KeyCode::Delete => {
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
                            Some(PopupState::Log(scroll)) if event.code == KeyCode::Delete =>
                            {
                                *scroll = 0;
                                self.log.clear();
                            }
                            _ => {}
                        
                        }
                    }
                    _ => {}
                }
            },
            event::Event::Resize(width, _height) =>
            {
                Self::resize_popup_if_needed(&mut popup);
                self.resize_if_needed(width);
            },
            _ => {}
        }
        self.popup = popup;
        Ok(())
    }

    pub(super) fn handle_event(&mut self, event: event::Event) -> Result<(),Box<dyn std::error::Error>>
    {
        if self.popup.is_some()
        {
            self.handle_event_popup(event)?;
        }
        else 
        {
            self.handle_event_normal(event)?;
        }

        Ok(())
    }
}