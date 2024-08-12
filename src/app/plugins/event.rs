use bitflags::bitflags;
use crossterm::event::{KeyEvent, MouseEvent};

use super::ui_location::ui_location::UiLocation;

pub enum Event<'app> {
    Open,
    Edit {
        new_bytes: &'app mut Vec<u8>,
    },
    Save,
    Key {
        event: KeyEvent,
    },
    // TODO: provide more abstract info about where the mouse event occurred
    Mouse {
        event: MouseEvent,
        location: Option<UiLocation>,
    },
    Focus,
    Blur,
    Paste {
        text: String,
    },
    Resize {
        width: u16,
        height: u16,
    },
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Events: u16
    {
        const ON_OPEN   = 0b0000_0000_0000_0001;
        const ON_EDIT   = 0b0000_0000_0000_0010;
        const ON_SAVE   = 0b0000_0000_0000_0100;
        const ON_KEY    = 0b0000_0000_0000_1000;
        const ON_MOUSE  = 0b0000_0000_0001_0000;
        const ON_FOCUS  = 0b0000_0000_0010_0000;
        const ON_BLUR   = 0b0000_0000_0100_0000;
        const ON_PASTE  = 0b0000_0000_1000_0000;
        const ON_RESIZE = 0b0000_0001_0000_0000;

        const NONE      = 0b0000_0000_0000_0000;
    }
}
