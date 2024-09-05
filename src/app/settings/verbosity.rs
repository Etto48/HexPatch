use serde::{Deserialize, Serialize};

use crate::app::log::NotificationLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Verbosity {
    Debug,
    #[default]
    Info,
    Warning,
    Error,
}

impl Verbosity {
    pub fn as_notification_level(&self) -> NotificationLevel {
        match self {
            Verbosity::Debug => NotificationLevel::Debug,
            Verbosity::Info => NotificationLevel::Info,
            Verbosity::Warning => NotificationLevel::Warning,
            Verbosity::Error => NotificationLevel::Error,
        }
    }
}
