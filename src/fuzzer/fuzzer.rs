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

    /// Returns a sorted list of keys, from most relevant to least relevant.
    /// ```
    /// use hex_patch::fuzzer::fuzzer::Fuzzer;
    /// 
    /// let fuzzer = Fuzzer::new(&["cherry", "banana", "apple"]);
    /// let results = fuzzer.fuzzy_search_sorted("a");
    /// assert_eq!(results, vec!["apple", "banana", "cherry"]);
    /// 
    /// let results = fuzzer.fuzzy_search_sorted("an");
    /// assert_eq!(results, vec!["banana", "apple", "cherry"]);
    /// ```
    pub fn fuzzy_search_sorted(&self, key: &str) -> Vec<String>
    {
        let mut ret = self.entries.clone();
        ret.sort_by_key(|entry| -entry.score(key));
        ret.into_iter().map(|entry| entry.key).collect()
    }
}