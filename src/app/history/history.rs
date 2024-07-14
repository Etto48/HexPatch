#![allow(clippy::module_inception)]

use std::collections::VecDeque;

use super::change::Change;
#[derive(Debug, Clone, Default)]
pub struct History {
    limit: usize,
    changes: VecDeque<Change>,
    current: usize,
}

impl History {
    /// If limit is 0, there is no limit.
    pub fn with_limit(limit: usize) -> Self {
        Self {
            limit,
            changes: VecDeque::with_capacity(limit),
            current: 0,
        }
    }

    pub fn push(&mut self, change: Change) {
        self.changes.truncate(self.current);
        if self.changes.len() >= self.limit && self.limit > 0 {
            self.changes.remove(0);
            self.current = self.current.saturating_sub(1);
        }
        self.changes.push_back(change);
        self.current += 1;
    }

    /// Undo the last change.
    /// Returns the change that was undone, if any.
    pub fn undo(&mut self, data: &mut Vec<u8>) -> Option<&Change> {
        if self.current == 0 {
            None
        } else {
            self.current -= 1;
            self.changes[self.current].revert(data);
            Some(&self.changes[self.current])
        }
    }

    /// Redo the last change.
    /// Returns the change that was redone, if any.
    pub fn redo(&mut self, data: &mut Vec<u8>) -> Option<&Change> {
        if self.current == self.changes.len() {
            None
        } else {
            self.changes[self.current].apply(data);
            self.current += 1;
            Some(&self.changes[self.current - 1])
        }
    }

    pub fn change_limit(&mut self, limit: usize) {
        self.limit = limit;
        if self.changes.len() > limit && limit > 0 {
            self.changes.drain(0..self.changes.len() - limit);
            self.current = limit;
        }
        if let Some(additional) = limit.checked_sub(self.changes.capacity()) {
            self.changes.reserve(additional);
        } else {
            self.changes.shrink_to_fit();
        }
    }

    pub fn clear(&mut self) {
        self.changes.clear();
        self.current = 0;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_history_with_limit() {
        let mut history = History::with_limit(2);
        assert_eq!(history.limit, 2);
        assert_eq!(history.changes.capacity(), 2);

        history.push(Change::new(0, &[0], &[1]));
        assert_eq!(history.current, 1);
        history.push(Change::new(0, &[1], &[0]));
        assert_eq!(history.current, 2);
        history.push(Change::new(0, &[0], &[1]));
        assert_eq!(history.current, 2);
        assert_eq!(history.changes.len(), 2);

        history.undo(&mut vec![0]);
        assert_eq!(history.current, 1);
        history.undo(&mut vec![0]);
        assert_eq!(history.current, 0);
        assert!(history.undo(&mut vec![0]).is_none());
        assert_eq!(history.current, 0);
    }

    #[test]
    fn test_history_change_limit() {
        let mut history = History::with_limit(2);
        history.push(Change::new(0, &[0], &[1]));
        history.push(Change::new(0, &[1], &[2]));
        history.push(Change::new(0, &[2], &[3]));
        assert_eq!(history.changes.len(), 2);
        assert_eq!(history.current, 2);

        history.change_limit(1);
        assert_eq!(history.changes.len(), 1);
        assert_eq!(history.current, 1);
        assert_eq!(history.changes[0], Change::new(0, &[2], &[3]));
    }
}
