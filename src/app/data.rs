use super::history::{change::Change, History};

#[derive(Debug, Clone, Default)]
pub struct Data {
    bytes: Vec<u8>,
    history: History,
    dirty: bool,
}

impl Data {
    pub fn new(bytes: Vec<u8>, history_limit: usize) -> Self {
        Self {
            bytes,
            history: History::with_limit(history_limit),
            dirty: false,
        }
    }

    pub fn get(&self, i: usize) -> Option<u8> {
        self.bytes.get(i).copied()
    }

    pub fn set(&mut self, i: usize, byte: u8) -> Result<(), mlua::Error> {
        match self.bytes.get_mut(i) {
            Some(b) => {
                self.history.push(Change::new(i, &[*b], &[byte]));
                *b = byte;
                self.dirty = true;
                Ok(())
            }
            None => Err(mlua::Error::external("index out of bounds")),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn reset_dirty(&mut self) {
        self.dirty = false;
    }

    /// Pushes a change to the history and updates the data.
    /// Returns the number of bytes changed.
    /// Panics if the offset is out of bounds.
    pub fn push_change(&mut self, offset: usize, mut new: Vec<u8>) -> usize {
        if offset >= self.bytes.len() {
            panic!(
                "Offset {} out of bounds for data of length {}",
                offset,
                self.bytes.len()
            );
        }
        new.truncate(self.bytes.len().checked_sub(offset).unwrap());
        let old = &self.bytes[offset..offset + new.len()];
        if old == new.as_slice() {
            return 0;
        }
        self.history.push(Change::new(offset, old, &new));
        self.bytes[offset..offset + new.len()].copy_from_slice(&new);
        self.dirty = true;
        new.len()
    }

    /// Undo the last change.
    /// Returns the change that was undone, if any.
    pub fn undo(&mut self) -> Option<&Change> {
        self.history.undo(&mut self.bytes)
    }

    /// Redo the last change.
    /// Returns the change that was redone, if any.
    pub fn redo(&mut self) -> Option<&Change> {
        self.history.redo(&mut self.bytes)
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_push_change() {
        let mut data = Data::new(vec![0, 1, 2, 3, 4], 0);
        assert_eq!(data.push_change(2, vec![9, 8, 7]), 3);
        assert_eq!(data.bytes(), &[0, 1, 9, 8, 7]);
        assert_eq!(data.push_change(2, vec![9, 8, 7]), 0);
        assert_eq!(data.bytes(), &[0, 1, 9, 8, 7]);
        assert_eq!(data.push_change(2, vec![9, 8, 7, 6]), 0);
        assert_eq!(data.bytes(), &[0, 1, 9, 8, 7]);
        assert_eq!(data.push_change(2, vec![9, 8, 7, 6, 5]), 0);
        assert_eq!(data.bytes(), &[0, 1, 9, 8, 7]);
        assert_eq!(data.push_change(2, vec![9, 8]), 0);
        assert_eq!(data.bytes(), &[0, 1, 9, 8, 7]);
        assert_eq!(data.push_change(2, vec![9, 8, 7]), 0);
        assert_eq!(data.bytes(), &[0, 1, 9, 8, 7]);
        assert_eq!(data.push_change(1, vec![9, 8, 7, 6]), 4);
        assert_eq!(data.bytes(), &[0, 9, 8, 7, 6]);
        assert_eq!(data.push_change(1, vec![9, 8, 7, 6, 5]), 0);
        assert_eq!(data.bytes(), &[0, 9, 8, 7, 6]);
        assert_eq!(data.push_change(1, vec![1, 2, 3, 4, 5]), 4);
        assert_eq!(data.bytes(), &[0, 1, 2, 3, 4]);
    }

    #[test]
    #[should_panic]
    fn test_data_push_change_out_of_bounds() {
        let mut data = Data::new(vec![0, 1, 2, 3, 4], 0);
        data.push_change(5, vec![9, 8, 7]);
    }

    #[test]
    fn test_data_undo_redo() {
        let mut data = Data::new(vec![0, 1, 2, 3, 4], 0);
        data.push_change(2, vec![9, 8, 7]);
        assert_eq!(data.bytes(), &[0, 1, 9, 8, 7]);
        data.push_change(0, vec![9, 8]);
        assert_eq!(data.bytes(), &[9, 8, 9, 8, 7]);
        data.push_change(4, vec![9]);
        assert_eq!(data.bytes(), &[9, 8, 9, 8, 9]);
        data.undo();
        assert_eq!(data.bytes(), &[9, 8, 9, 8, 7]);
        data.undo();
        assert_eq!(data.bytes(), &[0, 1, 9, 8, 7]);
        data.redo();
        assert_eq!(data.bytes(), &[9, 8, 9, 8, 7]);
        data.redo();
        assert_eq!(data.bytes(), &[9, 8, 9, 8, 9]);
    }

    #[test]
    fn test_data_clear_history() {
        let mut data = Data::new(vec![0, 1, 2, 3, 4], 0);
        data.push_change(2, vec![9, 8, 7]);
        data.push_change(0, vec![9, 8]);
        data.push_change(4, vec![9]);
        assert_eq!(data.bytes(), &[9, 8, 9, 8, 9]);
        data.clear_history();
        data.undo();
        assert_eq!(data.bytes(), &[9, 8, 9, 8, 9]);
    }
}
