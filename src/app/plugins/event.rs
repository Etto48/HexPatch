use bitflags::bitflags;
use crossterm::event::{KeyEvent, MouseEvent};

pub enum Event<'app>
{
    Open {
        data: &'app mut Vec<u8>,
    },
    Edit {
        data: &'app mut Vec<u8>,
        starting_byte: usize,
        new_bytes: &'app mut Vec<u8>,
    },
    Save {
        data: &'app mut Vec<u8>,
    },
    Key {
        event: KeyEvent,
        data: &'app mut Vec<u8>,
        current_byte: usize,
    },
    // TODO: provide more abstract info about where the mouse event occurred
    Mouse {
        kind: String,
        row: u16,
        col: u16,
    },
}

impl<'app> Event<'app> 
{
    pub fn from_mouse_event(event: MouseEvent) -> Self
    {
        // TODO: make kind serializable and deserializable
        let kind = format!("{:?}", event.kind);
        let row = event.row;
        let col = event.column;

        Event::Mouse {
            kind,
            row,
            col,
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