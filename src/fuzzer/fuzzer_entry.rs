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

        let char_found_bonus = 10;
        let key_char_not_found_penalty = -5;
        let self_char_not_found_penalty = -1;

        let mut key_chars = key.chars();
        let self_chars = self.key.chars();
        let mut current_key_char = key_chars.next();
        for self_char in self_chars
        {
            if let Some(key_char) = current_key_char
            {
                if self_char == key_char
                {
                    score += char_found_bonus;
                    current_key_char = key_chars.next();
                }
                else
                {
                    score += self_char_not_found_penalty;
                }
            }
            else 
            {
                break;
            }
        }
        for _ in key_chars
        {
            score += key_char_not_found_penalty;
        }
        score
    }
}