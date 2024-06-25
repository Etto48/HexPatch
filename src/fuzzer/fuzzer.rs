pub fn fuzzy_search_cloned<T>(key: &str, entries: &[T]) -> Vec<T>
where T: AsRef<str> + Clone
{
    let mut ret = entries.to_vec();
    fuzzy_search_in_place(key, &mut ret);
    ret
}

pub fn fuzzy_search_in_place<T>(key: &str, entries: &mut [T])
where T: AsRef<str> + Clone
{
    entries.sort_by_cached_key(|source|score(source.as_ref(), key));
    entries.reverse();
}

fn score(source: &str, key: &str) -> isize
{
    let mut score = 0;

    let char_found_bonus = 10;
    let key_char_not_found_penalty = -5;
    let self_char_not_found_penalty = -1;

    let mut key_chars = key.chars();
    let self_chars = source.chars();
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