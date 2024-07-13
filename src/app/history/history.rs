#![allow(clippy::module_inception)]

use super::change::Change;
#[derive(Debug, Clone, Default)]
pub struct History {
    changes: Vec<Change>,
    current: usize,
}

impl History {
    pub fn push(&mut self, change: Change) {
        self.changes.truncate(self.current);
        self.changes.push(change);
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

    pub fn clear(&mut self) {
        self.changes.clear();
        self.current = 0;
    }
}
