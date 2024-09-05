use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
pub enum NotificationLevel {
    #[default]
    None = 0,
    Debug = 1,
    Info = 2,
    Warning = 3,
    Error = 4,
}

impl NotificationLevel {
    pub fn bump_notification_level(&mut self, new_notification_level: NotificationLevel) {
        if new_notification_level.notification_level_as_u8() > self.notification_level_as_u8() {
            *self = new_notification_level;
        }
    }

    pub fn reset(&mut self) {
        *self = NotificationLevel::None;
    }

    pub fn notification_level_as_u8(&self) -> u8 {
        *self as u8
    }
}

impl From<u8> for NotificationLevel {
    fn from(value: u8) -> Self {
        match value {
            0 => NotificationLevel::None,
            1 => NotificationLevel::Debug,
            2 => NotificationLevel::Info,
            3 => NotificationLevel::Warning,
            4 => NotificationLevel::Error,
            _ => NotificationLevel::None,
        }
    }
}

impl Display for NotificationLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationLevel::None => write!(f, "None "),
            NotificationLevel::Debug => write!(f, "Debug"),
            NotificationLevel::Info => write!(f, "Info "),
            NotificationLevel::Warning => write!(f, "Warn "),
            NotificationLevel::Error => write!(f, "Error"),
        }
    }
}
