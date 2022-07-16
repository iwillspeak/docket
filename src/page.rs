//! Markdonw Pages with TOC
//!
//! Provides the `Page` type, a&long with functions for fabricating and working
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
    title: String,
    content: String,
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

    /// Make a page from constituent parts
    fn from_parts<M: AsRef<str>>(slug: String, markdown: M) -> Self {
        // FIXME: use the body of the page...
        let title = format!("DUMMY PAGE TITLE {}", slug);
        Page {
            slug,
            title,
            content: markdown.as_ref().to_owned(),
        }
    }

    /// Get the title for this page
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get the slug for this page    
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// FIXME: Shim to allow copying
    pub fn content(&self) -> &str {
        &self.content
    }
}
