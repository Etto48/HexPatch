use bitflags::bitflags;
use crossterm::event::{KeyEvent, MouseEvent};

use crate::app::settings::key_settings::KeySettings;

pub enum Event<'a>
{
    Open {
        data: &'a mut Vec<u8>
    },
    Edit {
        data: &'a mut Vec<u8>,
        starting_byte: usize,
        new_bytes: &'a mut Vec<u8>
    },
    Save {
        data: &'a mut Vec<u8>
    },
    Key {
        code: String,
        modifiers: u8,
        kind: String,
        state: u8,
    },
    // TODO: provide more abstract info about where the mouse event occurred
    Mouse {
        kind: String,
        row: u16,
        col: u16,
    },
}

impl<'a> Event<'a> 
{
    pub fn from_key_event(event: KeyEvent) -> Self
    {
        let code = KeySettings::key_code_to_string(event.code);
        let kind = KeySettings::key_event_kind_to_string(event.kind);

        Event::Key {
            code,
            modifiers: event.modifiers.bits(),
            kind,
            state: event.state.bits(),
        }
    }

    pub fn from_mouse_event(event: MouseEvent) -> Self
    {
        let kind = format!("{:?}", event.kind);
        let row = event.row;
        let col = event.column;

        Event::Mouse {
            kind,
            row,
            col
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Events: u8
    {
        const ON_OPEN   = 0b0000_0001;
        const ON_EDIT   = 0b0000_0010;
        const ON_SAVE   = 0b0000_0100;
        const ON_KEY    = 0b0000_1000;
        const ON_MOUSE  = 0b0001_0000;

        const NONE      = 0b0000_0000;
    }
}