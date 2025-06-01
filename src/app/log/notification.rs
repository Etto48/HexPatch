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
        let max_len = t!("app.log_levels.none")
            .chars()
            .count()
            .max(t!("app.log_levels.debug").chars().count())
            .max(t!("app.log_levels.info").chars().count())
            .max(t!("app.log_levels.warn").chars().count())
            .max(t!("app.log_levels.error").chars().count());

        match self {
            NotificationLevel::None => write!(f, "{:<max_len$}", t!("app.log_levels.none")),
            NotificationLevel::Debug => write!(f, "{:<max_len$}", t!("app.log_levels.debug")),
            NotificationLevel::Info => write!(f, "{:<max_len$}", t!("app.log_levels.info")),
            NotificationLevel::Warning => write!(f, "{:<max_len$}", t!("app.log_levels.warn")),
            NotificationLevel::Error => write!(f, "{:<max_len$}", t!("app.log_levels.error")),
        }
    }
}
