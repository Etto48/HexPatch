use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeySettings
{
    pub up: KeyEvent,
    pub down: KeyEvent,
    pub left: KeyEvent,
    pub right: KeyEvent,

    pub next: KeyEvent,
    pub previous: KeyEvent,

    pub page_up: KeyEvent,
    pub page_down: KeyEvent,

    pub goto_start: KeyEvent,
    pub goto_end: KeyEvent,

    pub quit: KeyEvent,
    pub save_and_quit: KeyEvent,
    pub save: KeyEvent,
    pub open: KeyEvent,

    pub help: KeyEvent,
    pub log: KeyEvent,
    pub run: KeyEvent,
    pub find_text: KeyEvent,
    pub find_symbol: KeyEvent,
    pub patch_text: KeyEvent,
    pub patch_assembly: KeyEvent,
    pub jump: KeyEvent,
    pub change_view: KeyEvent,

    pub confirm: KeyEvent,
    pub close_popup: KeyEvent,

    pub new_line: KeyEvent,
    pub clear_log: KeyEvent,

}

impl Default for KeySettings
{
    fn default() -> Self
    {
        Self
        {
            up: KeyEvent::new(KeyCode::Up, KeyModifiers::empty()),
            down: KeyEvent::new(KeyCode::Down, KeyModifiers::empty()),
            left: KeyEvent::new(KeyCode::Left, KeyModifiers::empty()),
            right: KeyEvent::new(KeyCode::Right, KeyModifiers::empty()),

            next: KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL),
            previous: KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL),

            page_up: KeyEvent::new(KeyCode::PageUp, KeyModifiers::empty()),
            page_down: KeyEvent::new(KeyCode::PageDown, KeyModifiers::empty()),

            goto_start: KeyEvent::new(KeyCode::Home, KeyModifiers::empty()),
            goto_end: KeyEvent::new(KeyCode::End, KeyModifiers::empty()),

            quit: KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            save_and_quit: KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL),
            save: KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
            open: KeyEvent::new(KeyCode::Char('o'), KeyModifiers::CONTROL),

            help: KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty()),
            log: KeyEvent::new(KeyCode::Char('l'), KeyModifiers::empty()),
            run: KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()),
            find_text: KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty()),
            find_symbol: KeyEvent::new(KeyCode::Char('s'), KeyModifiers::empty()),
            patch_text: KeyEvent::new(KeyCode::Char('t'), KeyModifiers::empty()),
            patch_assembly: KeyEvent::new(KeyCode::Char('p'), KeyModifiers::empty()),
            jump: KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty()),
            change_view: KeyEvent::new(KeyCode::Char('v'), KeyModifiers::empty()),

            confirm: KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
            close_popup: KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()),

            new_line: KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT),
            clear_log: KeyEvent::new(KeyCode::Delete, KeyModifiers::empty()),
        }
    }
}