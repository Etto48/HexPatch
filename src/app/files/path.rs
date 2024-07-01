use regex::Regex;

pub fn is_absolute(path: &str) -> bool {
    path.starts_with('/')
        || Regex::new(r"(^(\\\\\?\\)?[a-zA-Z]:)|(^\\\\\?\\[a-zA-Z]{1,})")
            .expect("Invalid regex.")
            .is_match(path)
}

pub fn is_root(path: &str) -> bool {
    path == "/"
        || Regex::new(r"(^(\\\\\?\\)?[a-zA-Z]:\\?$)|(^\\\\\?\\[a-zA-Z]{1,}\\?$)")
            .expect("Invalid regex.")
            .is_match(path)
}

pub fn parent(path: &str) -> Option<&str> {
    if is_root(path) {
        return None;
    }
    let path = path.trim_end_matches(['/', '\\']);
    let last_delimiter_index = path.rfind(['/', '\\']);
    Some(path.split_at(last_delimiter_index? + 1).0)
}

pub fn join(start: &str, end: &str, separator: char) -> String {
    let start = start.trim_end_matches(separator);
    let end = end.trim_start_matches(separator);
    if end == ".." {
        parent(start).unwrap_or(start).to_string()
    } else if end == "." {
        start.to_string()
    } else {
        format!("{}{}{}", start, separator, end)
    }
}

pub fn filename(path: &str) -> Option<&str> {
    if is_root(path) {
        None
    } else {
        let path = path.trim_end_matches(['/', '\\']);
        path.rsplit(['/', '\\']).next()
    }
}

pub fn diff<'a>(full_path: &'a str, prefix_path: &str) -> &'a str {
    if full_path == prefix_path {
        "."
    } else if let Some(unprefixed_path) = full_path.strip_prefix(prefix_path) {
        unprefixed_path.trim_start_matches(['/', '\\'])
    } else if let Some(unprefixed_path_reverse_logic) = prefix_path.strip_prefix(full_path) {
        if unprefixed_path_reverse_logic
            .split(['/', '\\'])
            .filter(|s| !s.is_empty())
            .count()
            == 1
        {
            ".."
        } else {
            full_path
        }
    } else {
        full_path
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_absolute() {
        assert!(is_absolute("/"));
        assert!(!is_absolute("test"));
        assert!(is_absolute("/home/user"));
        assert!(!is_absolute("home/user"));

        assert!(!is_absolute("\\\\?\\"));
        assert!(is_absolute("\\\\?\\C:\\Users\\"));
        assert!(is_absolute("\\\\?\\d:\\Users"));
        assert!(is_absolute("C:\\Users"));
        assert!(is_absolute("d:\\Users"));
        assert!(!is_absolute("ASDF\\Users"));
        assert!(is_absolute("\\\\?\\ASDF\\"));
        assert!(!is_absolute(".\\Users"));
    }

    #[test]
    fn test_is_root() {
        assert!(is_root("/"));
        assert!(!is_root("test"));
        assert!(!is_root("/home/user"));
        assert!(!is_root("home/user"));

        assert!(!is_root("\\\\?\\"));
        assert!(is_root("\\\\?\\C:\\"));
        assert!(is_root("\\\\?\\d:"));
        assert!(is_root("C:\\"));
        assert!(is_root("d:"));
        assert!(!is_root("ASDF\\"));
        assert!(is_root("\\\\?\\ASDF\\"));
        assert!(!is_root(".\\Users"));
    }

    #[test]
    fn test_parent() {
        assert_eq!(parent("/home/user"), Some("/home/"));
        assert_eq!(parent("/home"), Some("/"));
        assert_eq!(parent("/"), None);
        assert_eq!(parent("C:\\Users\\user"), Some("C:\\Users\\"));
        assert_eq!(parent("C:\\Users"), Some("C:\\"));
        assert_eq!(parent("C:\\"), None);
        assert_eq!(parent("\\\\?\\C:\\Users\\user"), Some("\\\\?\\C:\\Users\\"));
        assert_eq!(parent("\\\\?\\C:\\Users"), Some("\\\\?\\C:\\"));
        assert_eq!(parent("\\\\?\\C:\\"), None);
        assert_eq!(
            parent("\\\\?\\ASDF\\Users\\user"),
            Some("\\\\?\\ASDF\\Users\\")
        );
        assert_eq!(parent("\\\\?\\ASDF\\Users"), Some("\\\\?\\ASDF\\"));
        assert_eq!(parent("\\\\?\\ASDF"), None);
    }

    #[test]
    fn test_join() {
        assert_eq!(join("/home", "user", '/'), "/home/user");
        assert_eq!(join("/home", "..", '/'), "/");
        assert_eq!(join("/home", ".", '/'), "/home");

        assert_eq!(join("C:\\Users", "user", '\\'), "C:\\Users\\user");
        assert_eq!(join("C:\\Users", "..", '\\'), "C:\\");
        assert_eq!(join("C:\\Users", ".", '\\'), "C:\\Users");
    }

    #[test]
    fn test_filename() {
        assert_eq!(filename("/home/user"), Some("user"));
        assert_eq!(filename("/home"), Some("home"));
        assert_eq!(filename("/"), None);
        assert_eq!(filename("C:\\Users\\user"), Some("user"));
        assert_eq!(filename("C:\\Users"), Some("Users"));
        assert_eq!(filename("C:\\"), None);
        assert_eq!(filename("\\\\?\\C:\\Users\\user"), Some("user"));
        assert_eq!(filename("\\\\?\\C:\\Users"), Some("Users"));
        assert_eq!(filename("\\\\?\\C:\\"), None);
        assert_eq!(filename("\\\\?\\ASDF\\Users\\user"), Some("user"));
        assert_eq!(filename("\\\\?\\ASDF\\Users"), Some("Users"));
        assert_eq!(filename("\\\\?\\ASDF"), None);
    }

    #[test]
    fn test_diff() {
        assert_eq!(diff("/home/user", "/home"), "user");
        assert_eq!(diff("/home", "/home/user"), "..");
        assert_eq!(diff("/home", "/home"), ".");
        assert_eq!(diff("/home", "/"), "home");
        assert_eq!(diff("/", "/"), ".");
        assert_eq!(diff("C:\\Users\\user", "C:\\Users"), "user");
        assert_eq!(diff("C:\\Users", "C:\\Users\\user"), "..");
        assert_eq!(diff("C:\\Users", "C:\\Users"), ".");
        assert_eq!(diff("C:\\Users", "C:\\"), "Users");
        assert_eq!(diff("C:\\", "C:\\"), ".");
        assert_eq!(diff("\\\\?\\C:\\Users\\user", "\\\\?\\C:\\Users"), "user");
        assert_eq!(diff("\\\\?\\C:\\Users", "\\\\?\\C:\\Users"), ".");
        assert_eq!(diff("\\\\?\\C:\\Users", "\\\\?\\C:\\"), "Users");
        assert_eq!(diff("\\\\?\\C:\\", "\\\\?\\C:\\"), ".");
        assert_eq!(
            diff("\\\\?\\ASDF\\Users\\user", "\\\\?\\ASDF\\Users"),
            "user"
        );
        assert_eq!(diff("\\\\?\\ASDF\\Users", "\\\\?\\ASDF\\Users"), ".");
        assert_eq!(diff("\\\\?\\ASDF\\Users", "\\\\?\\ASDF"), "Users");
        assert_eq!(diff("\\\\?\\ASDF", "\\\\?\\ASDF"), ".");
    }
}
