use std::ops::Index;

use crate::app::App;

use super::{log_line::LogLine, notification::NotificationLevel};

pub struct Logger {
    pub(super) log: Vec<LogLine>,
    pub(super) notification: NotificationLevel,
}

impl Logger {
    pub fn new() -> Self
    {
        Logger
        {
            log: Vec::new(),
            notification: NotificationLevel::None,
        }
    }

    pub fn log(&mut self, level: NotificationLevel, message: &str)
    {
        self.notification.bump_notification_level(level);
        self.log.push(LogLine::new(level, message.to_string()));
    }

    pub fn clear(&mut self)
    {
        self.log.clear();
        self.notification.reset();
    }

    pub fn get_notification_level(&self) -> NotificationLevel
    {
        self.notification
    }

    pub fn len(&self) -> usize
    {
        self.log.len()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a LogLine> + DoubleEndedIterator + ExactSizeIterator
    {
        self.log.iter()
    }

    pub fn is_empty(&self) -> bool
    {
        self.log.is_empty()
    }

    pub fn reset_notification_level(&mut self)
    {
        self.notification.reset();
    }
}

impl Index<usize> for Logger
{
    type Output = LogLine;

    fn index(&self, index: usize) -> &Self::Output
    {
        &self.log[index]
    }
}

impl App
{
    pub(in crate::app) fn log(&mut self, level: NotificationLevel, message: &str)
    {
        self.logger.log(level, message);
    }
}