//! Markdonw Pages with TOC
//!
//! Provides the `Page` type, along with functions for fabricating and working
//! with pages.

use std::fs;

use crate::utils;

/// A Documentation Page
///
/// Each documentation page is made of two items: a simple `slug` which should
/// be used to refer to the page in navigation contexts, and a TOC. The TOC is a
/// hierachical representation of the contents of the page and can be traversed
/// to inspect the structure as well as rendred to HTML.
#[derive(Debug)]
pub struct Page {
    slug: String,
}

impl Page {
    /// Open a Page
    ///
    /// Loads the contents of the given file and parses it as markdown.
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, std::io::Error> {
        let slug = utils::slugify_path(&path);
        let contents = fs::read_to_string(path)?;
        Ok(Page::from_parts(slug, contents))
    }

    fn from_parts<M: AsRef<str>>(slug: String, _markdown: M) -> Self {
        // FIXME: use the body of the page...
        Page { slug }
    }
}
