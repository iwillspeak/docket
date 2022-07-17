//! The Documentation Tree
//!
//! This module defines the types that model out the tree of documentation to be
//! rendered. The tree is defined in such a way that it can be lazily traversed
//! to avoid pulling the whole thing into memory at the beginning of rendering.
//!
//! The documentation tree is made up of two things: bales and pages. Bales form
//! the interior nodes of the tree, and pages the leaves.

// FIXME: Remove once this is done prototyping
#![allow(dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
    result,
};

use crate::{
    error::Result,
    utils::{self, slugify_path},
};

/// A Doctree Item
///
/// Represents the kinds of item that can appear within the doctree.
enum DoctreeItem {
    /// A leaf page
    Page(Page),

    /// An unopened bale
    Bale(Bale),
}

impl DoctreeItem {}

/// A Documentation Page
///
/// Each documentaiton page is mad eup of two items: a simple `slug` which
/// should be used to refer to the page in the navigation tree, and a TOC. The
/// TOC is a heirachical represenattion of the contents of the page. A TOC can
/// be traversed to inspect the structure of the page; and rendered to HTML.
#[derive(Debug)]
pub(crate) struct Page {
    // TODO: This is widly different from what the documentaion, or design, suggest.
    slug: String,
    title: String,
    content: String,
}

impl Page {
    /// Open a Page
    ///
    /// Loads the contents of the given file and parses it as markdown.
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> result::Result<Self, std::io::Error> {
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

/// An Unopened Bale
///
/// Represents an interior node in the tree of documentation. Bales contain an
/// index page, a number of child pages, and a number of assets. Assets are
/// carried with the bale to ensure that relative paths to images and other
/// items are preserved.
///
/// An unopend bale can be broken open to access the inner `DoctreeItem`s within
/// the bale. Until opened a `Bale` only performs a shallow traversal of its
/// directory. This restrict sthe information avilable to just that needed to
/// build the bale's own navigation item within the navigation tree.
#[derive(Debug)]
pub(crate) struct Bale {
    /// The frontispiece for this bale. This contains the bale's resolved slug
    /// and title to be used in navigation.
    frontispice: BaleFrontispice,
    /// The paths we suspect to be page items
    pages: Vec<PathBuf>,
    /// The paths we know to be assets
    assets: Vec<PathBuf>,
    /// The paths we susepct to be child bales
    nested: Vec<PathBuf>,
}

impl Bale {
    /// Createa New Bale
    ///
    /// Wraps the given `path` as a bale. This performs a shallow traversal of
    /// the directory to find the index.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut index = None;
        let mut footer = None;
        let mut pages = Vec::new();
        let mut assets = Vec::new();
        let mut nested = Vec::new();

        // Walk the items in the directory and collect them into the initial
        // unsorted bale contents. We're just using raw paths at this point to refer
        // to all the bale's contents.
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path().clone();

            if path.is_file() {
                match utils::normalised_path_ext(&path).as_deref() {
                    Some("md" | "markdown" | "mdown") => {
                        match utils::normalised_stem(&path).as_deref() {
                            Some("index" | "readme") => index = Some(path),
                            Some("footer") => footer = Some(fs::read_to_string(path)?),
                            _ => pages.push(path),
                        }
                    }
                    _ => assets.push(path),
                }
            } else {
                nested.push(path);
            }
        }

        // If we have an index item in this bale then open the page. We need this to
        // know the bale's intended title for navigation purposes.
        let index = match index {
            Some(path) => Some(Page::open(path)?),
            None => None,
        };

        Ok(Bale {
            frontispice: BaleFrontispice::new(path, index, footer),
            pages,
            assets,
            nested,
        })
    }
}

/// Bale Frontispiece
///
/// The frontispiece represents the eagerly loaded portion of the bale. Bales
/// are broken open into three parts: frontispiece, assets, and inner items.
/// This type is used to group together the index.
#[derive(Debug)]
struct BaleFrontispice {
    /// The title for this bale. This is from the index page, if there is one,
    /// or falls back to the directory name otherwise.
    title: String,
    /// The slug for this bale
    ///
    /// TODO: Do we want a special `Slug` type to wrap these?
    slug: String,

    /// Index page for the bale, if one exists
    index: Option<Page>,

    /// The footer information for this bale. Rendering of any nested pages
    /// should use this as the markdown for the page's footer.
    footer: Option<String>,
}

impl BaleFrontispice {
    /// Create a new Frontispiece
    ///
    /// This picks a title and slug for the bale based on the bale's path.
    fn new<P: AsRef<Path>>(
        path: P,
        index: Option<Page>,
        footer: Option<String>,
    ) -> BaleFrontispice {
        let title = match &index {
            Some(page) => page.title.clone(),
            None => utils::prettify_dir(&path).expect("Could not create a title"),
        };
        BaleFrontispice {
            title,
            slug: slugify_path(path),
            index,
            footer,
        }
    }
}

/// Open a Doctree
///
/// This tries to create a new doctree rooted at the given `path`. If the path
/// can be opened and loaded as a valid `Bale` then that `Bale` is returned. If
/// there was an error initialising the doctree that failure is propagated.
pub(crate) fn open<P: AsRef<Path>>(path: P) -> Result<Bale> {
    Bale::new(path)
}

#[cfg(test)]
mod test {}
