use std::ops::Index;

use crate::app::App;

use super::{log_line::LogLine, notification::NotificationLevel};

#[derive(Debug, Clone)]
pub struct Logger {
    pub(super) log: Vec<LogLine>,
    pub(super) notification: NotificationLevel,
}

impl Logger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn log(&mut self, level: NotificationLevel, message: &str) {
        self.notification.bump_notification_level(level);
        self.log.push(LogLine::new(level, message.to_string()));
    }

    pub fn clear(&mut self) {
        self.log.clear();
        self.notification.reset();
    }

    pub fn get_notification_level(&self) -> NotificationLevel {
        self.notification
    }

    pub fn len(&self) -> usize {
        self.log.len()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &LogLine> + ExactSizeIterator {
        self.log.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.log.is_empty()
    }

    pub fn reset_notification_level(&mut self) {
        self.notification.reset();
    }

    pub fn merge(&mut self, other: &Self) {
        for log_line in &other.log {
            self.log.push(log_line.clone());
        }
        self.notification
            .bump_notification_level(other.notification);
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            log: Vec::new(),
            notification: NotificationLevel::None,
        }
    }
}

impl Index<usize> for Logger {
    type Output = LogLine;

    fn index(&self, index: usize) -> &Self::Output {
        &self.log[index]
    }
}

impl App {
    pub(in crate::app) fn log(&mut self, level: NotificationLevel, message: &str) {
        self.logger.log(level, message);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_logger() {
        let mut logger = Logger::new();
        assert_eq!(logger.len(), 0);
        assert!(logger.is_empty());
        logger.log(NotificationLevel::Error, "Test error message");
        assert_eq!(logger.len(), 1);
        assert!(!logger.is_empty());
        assert_eq!(logger[0].level, NotificationLevel::Error);
        assert_eq!(logger[0].message, "Test error message");
        logger.clear();
        assert_eq!(logger.len(), 0);
        assert!(logger.is_empty());
    }

    #[test]
    fn test_logger_merge() {
        let mut logger1 = Logger::new();
        let mut logger2 = Logger::new();
        logger1.log(NotificationLevel::Error, "Test error message");
        logger2.log(NotificationLevel::Warning, "Test warning message");
        logger1.merge(&logger2);
        assert_eq!(logger1.len(), 2);
        assert_eq!(logger1[0].level, NotificationLevel::Error);
        assert_eq!(logger1[0].message, "Test error message");
        assert_eq!(logger1[1].level, NotificationLevel::Warning);
        assert_eq!(logger1[1].message, "Test warning message");
        assert_eq!(logger1.get_notification_level(), NotificationLevel::Error);
    }
}
