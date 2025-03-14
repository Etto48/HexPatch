use std::collections::{hash_map::Iter, HashMap};

use super::App;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Comments {
    comments: HashMap<u64, String>,
}

impl Comments {
    pub fn new() -> Self {
        Self {
            comments: HashMap::new(),
        }
    }

    pub fn remove(&mut self, address: &u64) {
        self.comments.remove(address);
    }

    pub fn insert(&mut self, address: u64, comment: String) {
        self.comments.insert(address, comment);
    }

    pub fn iter(&self) -> Iter<'_, u64, String> {
        self.comments.iter()
    }

    pub fn get(&self, address: &u64) -> Option<&String> {
        self.comments.get(address)
    }

    pub fn get_mut(&mut self, address: &u64) -> Option<&mut String> {
        self.comments.get_mut(address)
    }

    pub fn len(&self) -> usize {
        self.comments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.comments.is_empty()
    }

    pub fn clear(&mut self) {
        self.comments.clear();
    }
}

impl App {
    pub(super) fn edit_comment(&mut self, comment: &str) {
        let address = self.get_cursor_position().global_byte_index as u64;
        if comment.is_empty() {
            self.comments.remove(&address);
        } else {
            self.comments.insert(address, comment.to_string());
        }
    }

    pub(super) fn find_comments(&self, filter: &str) -> Vec<(u64, String)> {
        if filter.is_empty() {
            return Vec::new();
        }
        let mut comments: Vec<(u64, String)> = self
            .comments
            .iter()
            .filter(|(_, symbol)| symbol.contains(filter))
            .map(|(address, symbol)| (*address, symbol.clone()))
            .collect();
        comments.sort_by_key(|(_, symbol)| symbol.len());
        comments
    }
}
