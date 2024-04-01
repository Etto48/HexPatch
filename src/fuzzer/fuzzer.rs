use super::fuzzer_entry::FuzzerEntry;

pub struct Fuzzer
{
    entries: Vec<FuzzerEntry>,
}

impl Fuzzer
{
    pub fn new(entries: &[&str]) -> Fuzzer
    {
        Fuzzer
        {
            entries: entries.iter().map(|entry| FuzzerEntry::new(entry)).collect(),
        }
    }

    pub fn fuzzy_search_sorted(&self, key: &str) -> Vec<String>
    {
        let mut ret = self.entries.clone();
        ret.sort_by_key(|entry| -entry.score(key));
        ret.into_iter().map(|entry| entry.key).collect()
    }
}