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
                                if self.dirty
                                {
                                    self.popup = Some(PopupState::QuitDirtySave(false));
                                }
                                else
                                {
                                    self.needs_to_exit = true;
                                }
                            },
                            's' => {
                                self.popup = Some(PopupState::Save(false));
                            },
                            'x' => {
                                if self.dirty
                                {
                                    self.popup = Some(PopupState::SaveAndQuit(false));
                                }
                                else
                                {
                                    self.needs_to_exit = true;
                                }
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
                                self.popup = Some(PopupState::Help(0));
                            },
                            'l' => {
                                self.notificaiton.reset();
                                self.popup = Some(PopupState::Log(0));
                            },
                            's' => {
                                self.popup = Some(PopupState::FindSymbol { filter: String::new(), symbols: Vec::new(), cursor: 0, scroll: 0 });
                            },
                            'p' => {
                                self.popup = Some(PopupState::Patch { assembly: String::new(), preview: Ok(Vec::new()), cursor: 0});
                            },
                            'j' => {
                                self.popup = Some(PopupState::JumpToAddress { location: String::new(), cursor: 0});
                            },
                            'v' => {
                                match self.info_mode {
                                    super::info_mode::InfoMode::Text => 
                                    {
                                        self.info_mode = super::info_mode::InfoMode::Assembly;
                                        self.update_hex_cursor();
                                    },
                                    super::info_mode::InfoMode::Assembly => 
                                    {
                                        self.info_mode = super::info_mode::InfoMode::Text;
                                        self.update_hex_cursor();
                                    },
                                }
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
                        popup = None;
                    },
                    KeyCode::Char(_) | 
                    KeyCode::Backspace | 
                    KeyCode::Delete => {
                        match &mut popup
                        {
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