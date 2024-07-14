#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    offset: usize,
    old: Vec<u8>,
    new: Vec<u8>,
}

impl Change {
    pub fn new(offset: usize, old: &[u8], new: &[u8]) -> Self {
        if old.len() != new.len() {
            panic!("Old and new data must be the same length");
        }
        Self {
            offset,
            old: old.to_vec(),
            new: new.to_vec(),
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn len(&self) -> usize {
        self.old.len()
    }

    pub fn is_empty(&self) -> bool {
        self.old.is_empty()
    }

    pub fn apply(&self, data: &mut Vec<u8>) {
        data.splice(
            self.offset..self.offset + self.old.len(),
            self.new.iter().cloned(),
        );
    }

    pub fn revert(&self, data: &mut Vec<u8>) {
        data.splice(
            self.offset..self.offset + self.new.len(),
            self.old.iter().cloned(),
        );
    }
}
