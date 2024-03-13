use crossterm::event::{self, KeyCode};

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
                                self.popup = Some(PopupState::Help);
                            },
                            'l' => {
                                self.notificaiton.reset();
                                self.popup = Some(PopupState::Log(0));
                            },
                            's' => {
                                self.popup = Some(PopupState::FindSymbol { filter: String::new(), cursor: 0, scroll: 0 });
                            },
                            'p' => {
                                self.popup = Some(PopupState::Patch { assembly: String::new(), cursor: 0});
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

    fn handle_string_edit(string: &mut String, cursor: &mut usize, event: &event::Event, charset: Option<&str>, capitalize: bool, max_len: Option<usize>) -> Result<(), Box<dyn std::error::Error>>
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
                    _ => {}
                }
            },
            _ => {}
        }
        Ok(())
    }

    fn handle_event_popup(&mut self, event: event::Event) -> Result<(), Box<dyn std::error::Error>>
    {
        match &mut self.popup
        {
            Some(PopupState::FindSymbol {filter, cursor, scroll: _scroll}) =>
            {
                Self::handle_string_edit(filter, cursor, &event, None, false, None)?;
            }
            Some(PopupState::Patch {assembly, cursor}) =>
            {
                Self::handle_string_edit(assembly, cursor, &event, None, false, None)?;
            }
            Some(PopupState::JumpToAddress {location: address, cursor}) =>
            {
                Self::handle_string_edit(address, cursor, &event, None, false, None)?;
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
                        match &self.popup
                        {
                            Some(PopupState::Save(yes_selected)) =>
                            {
                                self.popup = Some(PopupState::Save(!yes_selected));
                            }
                            Some(PopupState::SaveAndQuit(yes_selected)) =>
                            {
                                self.popup = Some(PopupState::SaveAndQuit(!yes_selected));
                            }
                            Some(PopupState::QuitDirtySave(yes_selected)) =>
                            {
                                self.popup = Some(PopupState::QuitDirtySave(!yes_selected));
                            },
                            _ => {}
                        }
                    },
                    KeyCode::Enter => {
                        let popup = self.popup.clone();
                        match popup
                        {
                            Some(PopupState::FindSymbol {filter, cursor: _cursor, scroll}) =>
                            {
                                self.jump_to_fuzzy_symbol(&filter, scroll);
                                self.popup = None;
                            }
                            Some(PopupState::Log(_)) =>
                            {
                                self.popup = None;
                            }
                            Some(PopupState::Patch {assembly, cursor: _cursor}) =>
                            {
                                self.patch(&assembly);
                                self.popup = None;
                            }
                            Some(PopupState::JumpToAddress {location, cursor: _cursor}) =>
                            {
                                self.jump_to_symbol(&location);
                                self.popup = None;
                            }
                            Some(PopupState::Save(yes_selected)) =>
                            {
                                if yes_selected
                                {
                                    self.save_data();
                                }
                                self.popup = None;
                            },
                            Some(PopupState::SaveAndQuit(yes_selected)) =>
                            {
                                if yes_selected
                                {
                                    self.save_data();
                                    self.needs_to_exit = true;
                                }
                                self.popup = None;
                            },
                            Some(PopupState::QuitDirtySave(yes_selected)) =>
                            {
                                if yes_selected
                                {
                                    self.save_data();
                                    self.needs_to_exit = true;
                                }
                                else
                                {
                                    self.needs_to_exit = true;
                                }
                                self.popup = None;
                            },
                            Some(PopupState::Help) => 
                            {
                                self.popup = None;
                            }
                            None => {}
                        }
                    },
                    KeyCode::Down => {
                        match &mut self.popup
                        {
                            Some(PopupState::FindSymbol { filter: _filter, cursor: _cursor, scroll }) =>
                            {
                                *scroll += 1;
                            },
                            Some(PopupState::Log(scroll)) =>
                            {
                                *scroll = scroll.saturating_sub(1);
                                self.popup = Some(PopupState::Log(*scroll));
                            }
                            _ => {}
                        }
                    },
                    KeyCode::Up => {
                        match &mut self.popup
                        {
                            Some(PopupState::FindSymbol { filter: _filter, cursor: _cursor, scroll }) =>
                            {
                                *scroll = scroll.saturating_sub(1);
                            },
                            Some(PopupState::Log(scroll)) =>
                            {
                                *scroll += 1;
                                let lines = 8;
                                if *scroll as isize >= self.log.len() as isize - lines as isize
                                {
                                    *scroll = self.log.len().saturating_sub(lines);
                                }
                                self.popup = Some(PopupState::Log(*scroll));
                            }
                            _ => {}
                        }
                    },
                    KeyCode::Esc => {
                        self.popup = None;
                    },
                    KeyCode::Char(_c) => {
                        match &self.popup
                        {
                            Some(PopupState::FindSymbol { filter, cursor, scroll: _scroll }) => 
                            {
                                self.popup = Some(PopupState::FindSymbol { filter: filter.clone(), cursor: *cursor, scroll: 0});
                            }
                            _ => {}
                        
                        }
                    }
                    _ => {}
                }
            },
            event::Event::Resize(width, _height) =>
            {
                self.resize_if_needed(width);
            },
            _ => {}
        }
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