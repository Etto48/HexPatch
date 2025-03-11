use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MediaKeyCode, ModifierKeyCode,
};
use serde::{Deserialize, Serialize};

use crate::RegisterKeySettings;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
#[derive(RegisterKeySettings!)]
pub struct KeySettings {
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
    pub save_as: KeyEvent,
    pub save: KeyEvent,
    pub open: KeyEvent,

    pub help: KeyEvent,
    pub log: KeyEvent,
    pub run: KeyEvent,
    pub find_text: KeyEvent,
    pub find_symbol: KeyEvent,
    pub edit_comment: KeyEvent,
    pub find_comment: KeyEvent,
    pub patch_text: KeyEvent,
    pub patch_assembly: KeyEvent,
    pub jump: KeyEvent,
    pub change_view: KeyEvent,
    pub change_selected_pane: KeyEvent,
    pub fullscreen: KeyEvent,

    pub confirm: KeyEvent,
    pub close_popup: KeyEvent,

    pub new_line: KeyEvent,
    pub clear_log: KeyEvent,

    pub undo: KeyEvent,
    pub redo: KeyEvent,
}

impl KeySettings {
    pub fn key_code_to_string(code: KeyCode) -> String {
        match code {
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Left => "Left".to_string(),
            KeyCode::Right => "Right".to_string(),
            KeyCode::Up => "Up".to_string(),
            KeyCode::Down => "Down".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::PageUp => "PageUp".to_string(),
            KeyCode::PageDown => "PageDown".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::BackTab => "BackTab".to_string(),
            KeyCode::Delete => "Delete".to_string(),
            KeyCode::Insert => "Insert".to_string(),
            KeyCode::F(n) => format!("F{}", n),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Null => "Null".to_string(),
            KeyCode::Esc => "Esc".to_string(),
            KeyCode::CapsLock => "CapsLock".to_string(),
            KeyCode::ScrollLock => "ScrollLock".to_string(),
            KeyCode::NumLock => "NumLock".to_string(),
            KeyCode::PrintScreen => "PrintScreen".to_string(),
            KeyCode::Pause => "Pause".to_string(),
            KeyCode::Menu => "Menu".to_string(),
            KeyCode::KeypadBegin => "KeypadBegin".to_string(),
            KeyCode::Media(mcode) => match mcode {
                MediaKeyCode::Play => "Media(Play)".to_string(),
                MediaKeyCode::Pause => "Media(Pause)".to_string(),
                MediaKeyCode::PlayPause => "Media(PlayPause)".to_string(),
                MediaKeyCode::Reverse => "Media(Reverse)".to_string(),
                MediaKeyCode::Stop => "Media(Stop)".to_string(),
                MediaKeyCode::FastForward => "Media(FastForward)".to_string(),
                MediaKeyCode::Rewind => "Media(Rewind)".to_string(),
                MediaKeyCode::TrackNext => "Media(TrackNext)".to_string(),
                MediaKeyCode::TrackPrevious => "Media(TrackPrevious)".to_string(),
                MediaKeyCode::Record => "Media(Record)".to_string(),
                MediaKeyCode::LowerVolume => "Media(LowerVolume)".to_string(),
                MediaKeyCode::RaiseVolume => "Media(RaiseVolume)".to_string(),
                MediaKeyCode::MuteVolume => "Media(MuteVolume)".to_string(),
            },
            KeyCode::Modifier(modifier) => match modifier {
                ModifierKeyCode::LeftShift => "Modifier(LeftShift)".to_string(),
                ModifierKeyCode::LeftControl => "Modifier(LeftControl)".to_string(),
                ModifierKeyCode::LeftAlt => "Modifier(LeftAlt)".to_string(),
                ModifierKeyCode::LeftSuper => "Modifier(LeftSuper)".to_string(),
                ModifierKeyCode::LeftHyper => "Modifier(LeftHyper)".to_string(),
                ModifierKeyCode::LeftMeta => "Modifier(LeftMeta)".to_string(),
                ModifierKeyCode::RightShift => "Modifier(RightShift)".to_string(),
                ModifierKeyCode::RightControl => "Modifier(RightControl)".to_string(),
                ModifierKeyCode::RightAlt => "Modifier(RightAlt)".to_string(),
                ModifierKeyCode::RightSuper => "Modifier(RightSuper)".to_string(),
                ModifierKeyCode::RightHyper => "Modifier(RightHyper)".to_string(),
                ModifierKeyCode::RightMeta => "Modifier(RightMeta)".to_string(),
                ModifierKeyCode::IsoLevel3Shift => "Modifier(IsoLevel3Shift)".to_string(),
                ModifierKeyCode::IsoLevel5Shift => "Modifier(IsoLevel5Shift)".to_string(),
            },
        }
    }

    pub fn string_to_key_code(string: &str) -> Result<KeyCode, String> {
        match string {
            "Backspace" => Ok(KeyCode::Backspace),
            "Enter" => Ok(KeyCode::Enter),
            "Left" => Ok(KeyCode::Left),
            "Right" => Ok(KeyCode::Right),
            "Up" => Ok(KeyCode::Up),
            "Down" => Ok(KeyCode::Down),
            "Home" => Ok(KeyCode::Home),
            "End" => Ok(KeyCode::End),
            "PageUp" => Ok(KeyCode::PageUp),
            "PageDown" => Ok(KeyCode::PageDown),
            "Tab" => Ok(KeyCode::Tab),
            "BackTab" => Ok(KeyCode::BackTab),
            "Delete" => Ok(KeyCode::Delete),
            "Insert" => Ok(KeyCode::Insert),
            "F1" => Ok(KeyCode::F(1)),
            "F2" => Ok(KeyCode::F(2)),
            "F3" => Ok(KeyCode::F(3)),
            "F4" => Ok(KeyCode::F(4)),
            "F5" => Ok(KeyCode::F(5)),
            "F6" => Ok(KeyCode::F(6)),
            "F7" => Ok(KeyCode::F(7)),
            "F8" => Ok(KeyCode::F(8)),
            "F9" => Ok(KeyCode::F(9)),
            "F10" => Ok(KeyCode::F(10)),
            "F11" => Ok(KeyCode::F(11)),
            "F12" => Ok(KeyCode::F(12)),
            c if c.len() == 1 && c.chars().next().is_some() => {
                Ok(KeyCode::Char(c.chars().next().unwrap()))
            }
            "Null" => Ok(KeyCode::Null),
            "Esc" => Ok(KeyCode::Esc),
            "CapsLock" => Ok(KeyCode::CapsLock),
            "ScrollLock" => Ok(KeyCode::ScrollLock),
            "NumLock" => Ok(KeyCode::NumLock),
            "PrintScreen" => Ok(KeyCode::PrintScreen),
            "Pause" => Ok(KeyCode::Pause),
            "Menu" => Ok(KeyCode::Menu),
            "KeypadBegin" => Ok(KeyCode::KeypadBegin),
            "Media(Play)" => Ok(KeyCode::Media(MediaKeyCode::Play)),
            "Media(Pause)" => Ok(KeyCode::Media(MediaKeyCode::Pause)),
            "Media(PlayPause)" => Ok(KeyCode::Media(MediaKeyCode::PlayPause)),
            "Media(Reverse)" => Ok(KeyCode::Media(MediaKeyCode::Reverse)),
            "Media(Stop)" => Ok(KeyCode::Media(MediaKeyCode::Stop)),
            "Media(FastForward)" => Ok(KeyCode::Media(MediaKeyCode::FastForward)),
            "Media(Rewind)" => Ok(KeyCode::Media(MediaKeyCode::Rewind)),
            "Media(TrackNext)" => Ok(KeyCode::Media(MediaKeyCode::TrackNext)),
            "Media(TrackPrevious)" => Ok(KeyCode::Media(MediaKeyCode::TrackPrevious)),
            "Media(Record)" => Ok(KeyCode::Media(MediaKeyCode::Record)),
            "Media(LowerVolume)" => Ok(KeyCode::Media(MediaKeyCode::LowerVolume)),
            "Media(RaiseVolume)" => Ok(KeyCode::Media(MediaKeyCode::RaiseVolume)),
            "Media(MuteVolume)" => Ok(KeyCode::Media(MediaKeyCode::MuteVolume)),
            "Modifier(LeftShift)" => Ok(KeyCode::Modifier(ModifierKeyCode::LeftShift)),
            "Modifier(LeftControl)" => Ok(KeyCode::Modifier(ModifierKeyCode::LeftControl)),
            "Modifier(LeftAlt)" => Ok(KeyCode::Modifier(ModifierKeyCode::LeftAlt)),
            "Modifier(LeftSuper)" => Ok(KeyCode::Modifier(ModifierKeyCode::LeftSuper)),
            "Modifier(LeftHyper)" => Ok(KeyCode::Modifier(ModifierKeyCode::LeftHyper)),
            "Modifier(LeftMeta)" => Ok(KeyCode::Modifier(ModifierKeyCode::LeftMeta)),
            "Modifier(RightShift)" => Ok(KeyCode::Modifier(ModifierKeyCode::RightShift)),
            "Modifier(RightControl)" => Ok(KeyCode::Modifier(ModifierKeyCode::RightControl)),
            "Modifier(RightAlt)" => Ok(KeyCode::Modifier(ModifierKeyCode::RightAlt)),
            "Modifier(RightSuper)" => Ok(KeyCode::Modifier(ModifierKeyCode::RightSuper)),
            "Modifier(RightHyper)" => Ok(KeyCode::Modifier(ModifierKeyCode::RightHyper)),
            "Modifier(RightMeta)" => Ok(KeyCode::Modifier(ModifierKeyCode::RightMeta)),
            "Modifier(IsoLevel3Shift)" => Ok(KeyCode::Modifier(ModifierKeyCode::IsoLevel3Shift)),
            "Modifier(IsoLevel5Shift)" => Ok(KeyCode::Modifier(ModifierKeyCode::IsoLevel5Shift)),
            _ => Err(format!("Invalid KeyCode: {}", string)),
        }
    }

    pub fn key_event_kind_to_string(kind: KeyEventKind) -> String {
        match kind {
            KeyEventKind::Press => "Press".to_string(),
            KeyEventKind::Repeat => "Repeat".to_string(),
            KeyEventKind::Release => "Release".to_string(),
        }
    }

    pub fn string_to_key_event_kind(string: &str) -> Result<KeyEventKind, String> {
        match string {
            "Press" => Ok(KeyEventKind::Press),
            "Repeat" => Ok(KeyEventKind::Repeat),
            "Release" => Ok(KeyEventKind::Release),
            _ => Err(format!("Invalid KeyEventKind: {}", string)),
        }
    }
}

impl Default for KeySettings {
    fn default() -> Self {
        Self {
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
            save_as: KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL),
            save: KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
            open: KeyEvent::new(KeyCode::Char('o'), KeyModifiers::CONTROL),

            help: KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty()),
            log: KeyEvent::new(KeyCode::Char('l'), KeyModifiers::empty()),
            run: KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()),
            find_text: KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty()),
            find_symbol: KeyEvent::new(KeyCode::Char('s'), KeyModifiers::empty()),
            edit_comment: KeyEvent::new(KeyCode::Char(';'), KeyModifiers::empty()),
            find_comment: KeyEvent::new(KeyCode::Char(':'), KeyModifiers::empty()),
            patch_text: KeyEvent::new(KeyCode::Char('t'), KeyModifiers::empty()),
            patch_assembly: KeyEvent::new(KeyCode::Char('p'), KeyModifiers::empty()),
            jump: KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty()),
            change_view: KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL),
            change_selected_pane: KeyEvent::new(KeyCode::Char('v'), KeyModifiers::empty()),
            fullscreen: KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL),

            confirm: KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
            close_popup: KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()),

            new_line: KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT),
            clear_log: KeyEvent::new(KeyCode::Delete, KeyModifiers::empty()),

            undo: KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL),
            redo: KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL),
        }
    }
}
