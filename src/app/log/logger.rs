use std::{collections::VecDeque, ops::Index};

use crate::app::App;

use super::{log_line::LogLine, notification::NotificationLevel};

#[derive(Debug, Clone, Default)]
pub struct Logger {
    pub(super) limit: usize,
    pub(super) log: VecDeque<LogLine>,
    pub(super) notification: NotificationLevel,
}

impl Logger {
    pub fn with_limit(limit: usize) -> Self {
        Self {
            limit,
            log: VecDeque::with_capacity(limit),
            notification: NotificationLevel::None,
        }
    }

    pub fn log(&mut self, level: NotificationLevel, message: &str) {
        self.notification.bump_notification_level(level);
        if self.log.len() >= self.limit && self.limit > 0 {
            self.log.pop_front();
        }
        self.log.push_back(LogLine::new(level, message.to_string()));
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

    pub fn change_limit(&mut self, limit: usize) {
        self.limit = limit;
        if self.log.len() > limit && limit > 0 {
            self.log.drain(0..self.log.len() - limit);
        }
        if let Some(additional) = limit.checked_sub(self.log.capacity()) {
            self.log.reserve(additional);
        } else {
            self.log.shrink_to_fit();
        }
    }

    pub fn merge(&mut self, other: &Self) {
        for log_line in &other.log {
            if self.log.len() >= self.limit && self.limit > 0 {
                self.log.pop_front();
            }
            self.log.push_back(log_line.clone());
        }
        self.notification
            .bump_notification_level(other.notification);
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
        let mut logger = Logger::default();
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
        let mut logger1 = Logger::default();
        let mut logger2 = Logger::default();
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

    #[test]
    fn test_logger_with_limit() {
        let mut logger = Logger::with_limit(5);
        for i in 0..10 {
            logger.log(
                NotificationLevel::Error,
                &format!("Test error message {}", i),
            );
        }
        assert_eq!(logger.len(), 5);
        assert_eq!(logger[0].message, "Test error message 5");
        assert_eq!(logger[4].message, "Test error message 9");
    }

    #[test]
    fn test_logger_change_limit() {
        let mut logger = Logger::with_limit(2);
        logger.log(NotificationLevel::Error, "Test error message 1");
        logger.log(NotificationLevel::Error, "Test error message 2");
        logger.log(NotificationLevel::Error, "Test error message 3");
        assert_eq!(logger.len(), 2);
        assert_eq!(logger[0].message, "Test error message 2");
        assert_eq!(logger[1].message, "Test error message 3");
        logger.change_limit(3);
        assert_eq!(logger.len(), 2);
        assert_eq!(logger[0].message, "Test error message 2");
        assert_eq!(logger[1].message, "Test error message 3");
        logger.change_limit(1);
        assert_eq!(logger.len(), 1);
        assert_eq!(logger[0].message, "Test error message 3");
    }
}
