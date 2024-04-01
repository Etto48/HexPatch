#[derive(Debug, Clone)]
pub struct FuzzerEntry
{
    pub key: String,
}

impl FuzzerEntry
{
    pub fn new(key: &str) -> FuzzerEntry
    {
        FuzzerEntry
        {
            key: key.to_string(),
        }
    }

    pub fn score(&self, key: &str) -> isize
    {
        let mut score = 0;
        let mut key_chars = key.chars();
        let self_chars = self.key.chars();
        let mut current_key_char = key_chars.next();
        for self_char in self_chars
        {
            if let Some(key_char) = current_key_char
            {
                if self_char == key_char
                {
                    score += 1;
                    current_key_char = key_chars.next();
                }
            }
            else 
            {
                break;
            }
        }
        for _ in key_chars
        {
            score -= 1;
        }
        score
    }
}