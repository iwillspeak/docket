use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

/// Read a file's contents to a string
pub fn read_file_to_string(path: &Path) -> String {
    let mut contents = String::new();
    if let Ok(mut file) = File::open(path) {
        while let Ok(read) = file.read_to_string(&mut contents) {
            trace!("read {} from {:?}", read, path);
            if read == 0 {
                break;
            }
        }
    }
    contents
}

/// Convert a string of arbitrary charactes to a form suitable for use
/// as an HTML identifier or file name.
pub fn slugify(input: &str) -> String {

    let to_map = match input.find('-') {
        Some(offset) => {
            let (prefix, suffix) = input.split_at(offset + 1);
            if prefix[..offset].chars().all(|c| c.is_numeric()) {
                suffix
            } else {
                input
            }
        }
        None => input,
    };

    to_map
        .chars()
        .map(|c| match c {
            c if c.is_whitespace() => '-',
            '?' | '&' | '=' | '\\' | '/' | '"' => '-',
            _ => c,
        })
        .collect()
}


#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn slugify_simple_strings() {
        assert_eq!("a", slugify("a"));
        assert_eq!("hello", slugify("hello"));
        assert_eq!("1000", slugify("1000"));
    }

    #[test]
    fn slugify_replaces_space_characters() {
        assert_eq!("hello-world", slugify("hello world"));
        assert_eq!("foo-bar-baz", slugify("foo\tbar baz"));
    }

    #[test]
    fn slufigy_replaces_url_punctuation() {
        assert_eq!("foo-bar", slugify("foo&bar"));
        assert_eq!("foo-bar", slugify("foo\"bar"));
        assert_eq!("foo-bar", slugify("foo?bar"));
        assert_eq!("foo-bar", slugify("foo=bar"));
    }

    #[test]
    fn slugify_replaces_path_components() {
        assert_eq!("yop-yop-yop", slugify("yop/yop\\yop"));
    }

    #[test]
    fn slugify_allows_unicode() {
        assert_eq!("€$§±*", slugify("€$§±*"));
        assert_eq!("ü¬é∆∂", slugify("ü¬é∆∂"));
    }

    #[test]
    fn slufigy_removes_leading_numbers() {
        assert_eq!("hello", slugify("01-hello"));
        assert_eq!("01a-foo", slugify("01a-foo"));
        assert_eq!("01a-foo", slugify("01a-foo"));
    }
}
