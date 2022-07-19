use std::path::{Path, PathBuf};

/// Convert a string of arbitrary charactes to a form suitable for use
/// as an HTML identifier or file name.
pub(crate) fn slugify<S: AsRef<str>>(input: S) -> String {
    let input = input.as_ref();
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

/// Slufigy a Path
pub(crate) fn slugify_path<P: AsRef<Path>>(input: P) -> String {
    slugify(match input.as_ref().file_stem() {
        Some(stem) => stem.to_string_lossy(),
        _ => input.as_ref().to_string_lossy(),
    })
}

/// Get a Normalised File Extension, if available
///
/// Returns the downcased representaiton of the file extension if one is
/// available. If the path has no extension then `None` is returned.
pub(crate) fn normalised_path_ext<P: AsRef<Path>>(path: P) -> Option<String> {
    if let Some(ext) = path.as_ref().extension() {
        Some(ext.to_string_lossy().to_ascii_lowercase())
    } else {
        None
    }
}

/// Get the Normalised File Stem from the Path
///
/// Returns the file's stem, that is the part before the extension, downcased
/// and converted into a standard stirng.
pub(crate) fn normalised_stem<P: AsRef<Path>>(path: P) -> Option<String> {
    if let Some(stem) = path.as_ref().file_stem() {
        Some(stem.to_string_lossy().to_ascii_lowercase())
    } else {
        None
    }
}

/// Path or Default
///
/// Try to convert a command line argument to a path. Falling back to the
/// default if none is provided.
pub(crate) fn path_or_default(maybe_path: Option<String>, default: &str) -> PathBuf {
    maybe_path
        .map(PathBuf::from)
        .unwrap_or_else(|| default.to_owned().into())
}

/// Prettify a Directory Name
///
/// Trims the leading and trailing parts of the given directory name. Drops
/// leading numeric characters.
pub(crate) fn prettify_dir<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref().file_name().map(|name| {
        name.to_string_lossy()
            .trim_start_matches(char::is_numeric)
            .trim_start_matches(['-', '_'])
            .trim()
            .to_owned()
    })
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

    #[test]
    fn slugify_path_removes_extension() {
        assert_eq!("about", slugify_path(Path::new("01-about.md")));
        assert_eq!("foo", slugify_path(Path::new("foo.html")));
        assert_eq!("bar-baz", slugify_path(Path::new("10-bar-baz")));
        assert_eq!("human2.0", slugify_path(Path::new("human2.0.html")));
    }

    #[test]
    fn slugify_path_removes_leading_path() {
        assert_eq!("world", slugify_path(Path::new("hello/world.gif")));
        assert_eq!("e", slugify_path(Path::new("a/b/c/cd/11-e.10")));
    }

    #[test]
    fn normalised_path_ext_no_ext() {
        assert_eq!(None, normalised_path_ext("/hello/world"));
        assert_eq!(None, normalised_path_ext("bar::"));
        assert_eq!(None, normalised_path_ext("../some_path/"));
    }

    #[test]
    fn normalised_path_ext_returns_downcased_extension() {
        assert_eq!(Some(String::from("md")), normalised_path_ext("README.md"));
        assert_eq!(Some(String::from("rs")), normalised_path_ext("TEST.RS"));
    }

    #[test]
    fn normalised_stem_no_stem() {
        assert_eq!(None, normalised_stem("../"));
    }

    #[test]
    fn nromalised_stem_downcases_stem() {
        assert_eq!(Some(String::from("readme")), normalised_stem("README.md"));
        assert_eq!(Some(String::from("test")), normalised_stem("TEST.RS"));
        assert_eq!(Some(String::from("index")), normalised_stem("index.html"));
        assert_eq!(
            Some(String::from("world")),
            normalised_stem("../hello/../world/")
        );
        assert_eq!(
            Some(String::from(".config")),
            normalised_stem("/hello/.config.world/")
        );
    }

    #[test]
    fn prettified_dirnames() {
        assert_eq!(Some("Hello World"), prettify_dir("Hello World").as_deref());
        assert_eq!(Some("Test"), prettify_dir("/this/is/a/Test").as_deref());
        assert_eq!(Some("Test"), prettify_dir("../another/Test/.").as_deref());
        assert_eq!(
            Some("Testing2"),
            prettify_dir("./futher/10-Testing2/").as_deref()
        );
    }

    #[test]
    fn path_or_default_with_valid_argument() {
        let source = Some("/Users/foo/".to_owned());
        assert_eq!(Path::new("/Users/foo/"), path_or_default(source, "."));
    }

    #[test]
    fn path_or_default_without_argument() {
        let source = None;
        assert_eq!(Path::new("baz/"), path_or_default(source, "baz/"));
    }
}
