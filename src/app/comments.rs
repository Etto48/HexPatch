use std::collections::{hash_map::Iter, HashMap};

use serde::{Deserialize, Serialize};

use super::{log::NotificationLevel, App};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comments {
    comments: HashMap<u64, String>,
    #[serde(skip)]
    dirty: bool,
}

impl Comments {
    pub fn new() -> Self {
        Self {
            comments: HashMap::new(),
            dirty: false,
        }
    }

    pub fn remove(&mut self, address: &u64) {
        self.comments.remove(address);
        self.dirty = true;
    }

    pub fn insert(&mut self, address: u64, comment: String) {
        self.comments.insert(address, comment);
        self.dirty = true;
    }

    pub fn iter(&self) -> Iter<'_, u64, String> {
        self.comments.iter()
    }

    pub fn get(&self, address: &u64) -> Option<&String> {
        self.comments.get(address)
    }

    pub fn len(&self) -> usize {
        self.comments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.comments.is_empty()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn reset_dirty(&mut self) {
        self.dirty = false;
    }

    pub fn to_vec(&self) -> Vec<(u64, String)> {
        self.comments.iter().map(|(a, s)| (*a, s.clone())).collect()
    }

    pub fn check_max_address(&mut self, max_address: u64) {
        let mut comments_removed = false;
        self.comments.retain(|address, _| {
            if *address > max_address {
                comments_removed = true;
                false
            } else {
                true
            }
        });
        if comments_removed {
            self.dirty = true;
        }
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

    pub(super) fn get_comments_path(&self) -> String {
        let path = self.filesystem.pwd();
        path.to_string() + ".hp-data.json"
    }

    /// If comments_path is None, it will use the default path calculated by get_comments_path.
    pub(super) fn save_comments(&mut self, comments_path: Option<String>) {
        if self.comments.is_dirty() {
            let comments_str = serde_json::to_string_pretty(&self.comments).unwrap();
            let comments_path = comments_path.unwrap_or(self.get_comments_path());
            if let Err(e) = self.filesystem.create(&comments_path) {
                self.log(
                    NotificationLevel::Error,
                    t!("errors.create_comments", e = e),
                );
                return;
            }
            if let Err(e) = self
                .filesystem
                .write(&comments_path, comments_str.as_bytes())
            {
                self.log(NotificationLevel::Error, t!("errors.write_comments", e = e));
                return;
            }
            self.log(NotificationLevel::Info, t!("app.messages.comments_saved"));
            self.comments.reset_dirty();
        }
    }

    /// If comments_path is None, it will use the default path calculated by get_comments_path.
    pub(super) fn load_comments(&mut self, comments_path: Option<String>) {
        let comments_path = comments_path.unwrap_or(self.get_comments_path());
        match self.filesystem.read(&comments_path) {
            Ok(comments_data) => match serde_json::from_slice::<Comments>(&comments_data) {
                Ok(comments) => {
                    self.comments = comments;
                    self.comments
                        .check_max_address(self.data.bytes().len() as u64);
                    self.log(NotificationLevel::Info, t!("app.messages.comments_loaded"));
                }
                Err(e) => {
                    self.log(NotificationLevel::Error, t!("errors.parse_comments", e = e));
                }
            },
            Err(e) => {
                // This is in debug because the file may not exist.
                self.log(NotificationLevel::Debug, t!("errors.read_comments", e = e));
                self.comments = Comments::new();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{app::plugins::plugin::Plugin, get_app_context};

    use super::*;

    #[test]
    fn insert_and_remove() {
        let mut comments = Comments::new();
        comments.insert(0x1000, "comment_1".to_string());
        assert!(comments.is_dirty());
        comments.insert(0x2000, "comment_2".to_string());
        comments.insert(0x3000, "comment_3".to_string());
        comments.reset_dirty();
        assert_eq!(comments.len(), 3);
        assert_eq!(comments.get(&0x1000), Some(&"comment_1".to_string()));
        assert_eq!(comments.get(&0x2000), Some(&"comment_2".to_string()));
        assert_eq!(comments.get(&0x3000), Some(&"comment_3".to_string()));
        assert!(!comments.is_dirty());
        comments.remove(&0x2000);
        assert!(comments.is_dirty());
        assert_eq!(comments.len(), 2);
        assert_eq!(comments.get(&0x1000), Some(&"comment_1".to_string()));
        assert_eq!(comments.get(&0x2000), None);
        assert_eq!(comments.get(&0x3000), Some(&"comment_3".to_string()));
        comments.check_max_address(0x2000);
        assert_eq!(comments.len(), 1);
        assert_eq!(comments.get(&0x1000), Some(&"comment_1".to_string()));
        assert_eq!(comments.get(&0x2000), None);
        assert_eq!(comments.get(&0x3000), None);
    }

    #[test]
    fn save_and_load() {
        let mut app = App::mockup(vec![0x90; 0x100]);
        let tmp_comments = tempfile::NamedTempFile::new().unwrap();
        let comments_path = tmp_comments.path().to_str().unwrap().to_string();
        app.comments.insert(0x10, "comment_1".to_string());
        app.comments.insert(0x20, "comment_2".to_string());
        app.comments.insert(0x30, "comment_3".to_string());
        app.save_comments(Some(comments_path.clone()));
        assert!(!app.comments.is_dirty());
        assert!(app.logger.get_notification_level() < NotificationLevel::Warning);
        app.comments = Comments::new();
        app.load_comments(Some(comments_path));
        assert_eq!(app.comments.len(), 3);
        assert_eq!(app.comments.get(&0x10), Some(&"comment_1".to_string()));
        assert_eq!(app.comments.get(&0x20), Some(&"comment_2".to_string()));
        assert_eq!(app.comments.get(&0x30), Some(&"comment_3".to_string()));
    }

    #[test]
    fn test_plugin() {
        let source = "
        function init(context)
            context.set_comment(0x10, 'comment_1')
            context.set_comment(0x20, 'comment_2')
            context.set_comment(0x30, 'comment_3')
            c1 = context.get_comment(0x10)
            c2 = context.get_comment(0x20)
            c3 = context.get_comment(0x30)
            should_be_nil = context.get_comment(0x40)
            assert(c1 == 'comment_1', 'c1')
            assert(c2 == 'comment_2', 'c2')
            assert(c3 == 'comment_3', 'c3')
            assert(should_be_nil == nil, 'should_be_nil')
            comments = context.get_comments()
            assert(comments[0x10] == 'comment_1', 'table c1')
            assert(comments[0x20] == 'comment_2', 'table c2')
            assert(comments[0x30] == 'comment_3', 'table c3')
            context.set_comment(0x10, '')
            assert(context.get_comment(0x10) == nil, 'remove c1')
            context.set_comment(0x20, nil)
            assert(context.get_comment(0x20) == nil, 'remove c2')
        end";

        let mut app = App::mockup(vec![0; 0x100]);
        let mut app_context = get_app_context!(app);
        Plugin::new_from_source(source, &mut app_context).unwrap();
    }
}
