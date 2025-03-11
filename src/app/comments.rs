use super::App;

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
