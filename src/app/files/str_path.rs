pub fn path_is_absolute(path: &str) -> bool
{
    path.starts_with('/') || path.starts_with('\\')
}

pub fn path_is_root(path: &str) -> bool
{
    path == "/" || 
    (path.strip_prefix("\\\\?\\").map(
        |path| 
        {
            let bs_count = path.chars().filter(|c| *c=='\\').count();
            (bs_count == 1 && path.ends_with('\\')) ||
            (bs_count == 0)
        }
    ).unwrap_or(false))
}

pub fn path_parent(path: &str) -> Option<&str>
{
    if path_is_root(path)
    {
        return None;
    }
    let path = path.trim_end_matches(|c| c == '/' || c == '\\');
    let last_delimiter_index = path.rfind(|c| c == '/' || c == '\\');
    Some(path.split_at(last_delimiter_index? + 1).0)
}

pub fn path_join(start: &str, end: &str, separator: char) -> String
{
    let start = start.trim_end_matches(|c| c == separator);
    let end = end.trim_start_matches(|c| c == separator);
    if end == ".."
    {
        path_parent(start).unwrap_or(start).to_string()
    }
    else
    {
        format!("{}{}{}", start, separator, end)
    }
}

pub fn path_filename(path: &str) -> Option<&str>
{
    if path_is_root(path)
    {
        None
    }
    else
    {
        let path = path.trim_end_matches(|c| c == '/' || c == '\\');
        let path = path.rsplit(|c| c == '/' || c == '\\').collect::<Vec<_>>();
        Some(path[0])
    }
}

pub fn path_diff<'a>(full_path: &'a str, prefix_path: &str) -> &'a str
{
    if let Some(unprefixed_path) = full_path.strip_prefix(prefix_path)
    {
        unprefixed_path.trim_start_matches(|c| c == '/' || c == '\\')
    }
    else if let Some(unprefixed_path_reverse_logic) = prefix_path.strip_prefix(full_path)
    {
        if unprefixed_path_reverse_logic.split(|c| c == '/' || c == '\\').filter(|s|!s.is_empty()).count() == 1
        {
            return "..";
        }
        else
        {
            full_path
        }
    }
    else
    {
        full_path
    }
}