use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotificationLevel {
    #[default]
    None,
    Debug,
    Info,
    Warning,
    Error,
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
        match self {
            NotificationLevel::None => 0,
            NotificationLevel::Debug => 1,
            NotificationLevel::Info => 2,
            NotificationLevel::Warning => 3,
            NotificationLevel::Error => 4,
        }
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
