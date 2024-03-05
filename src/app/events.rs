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
                            'v' => {
                                match self.info_mode {
                                    super::info_mode::InfoMode::Text => 
                                    {
                                        self.info_mode = super::info_mode::InfoMode::Assembly;
                                    },
                                    super::info_mode::InfoMode::Assembly => 
                                    {
                                        self.info_mode = super::info_mode::InfoMode::Text;
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

    fn handle_event_popup(&mut self, event: event::Event) -> Result<(), Box<dyn std::error::Error>>
    {
        match event
        {
            event::Event::Key(event) if event.kind == event::KeyEventKind::Press => {
                match event.code
                {
                    KeyCode::Left |
                    KeyCode::Right => {
                        match self.popup
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
                            Some(PopupState::Help) => {}
                            None => {}
                        }
                    },
                    KeyCode::Enter => {
                        match self.popup
                        {
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
                    KeyCode::Esc => {
                        self.popup = None;
                    },
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