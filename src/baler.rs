//! The Baler
//!
//! The baler is responsible for waking a given node in the directory tree and
//! producing bales. A bale is collection of one or more pages and zero or more
//! assets.
//!
//! A bale is a lazy item. Opening a bale in a directory scans _only_ the
//! directory that is opened. The idnex for hte bale is calculated, and the
//! remaining items in the bale are lazily found when the bale is later walked.
//! This allows bales to be used in a depth-first walk of the documentation tree
//! without paying the cost of loading all items within all bales up front.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::{page::Page, utils};

/// Information about the bale's index. This is the eagerly opened portion of
/// the bale.
#[derive(Debug)]
struct IndexInfo {
    index: Option<Page>,
    footer: Option<String>,
}

/// An Unopened Bale
///
/// Represents an interior node in the tree of documentation. Bales contain an
/// index page, any number of child pages, and any number of assets. Assets are
/// carried with the bale to ensure that relative paths to images and other
/// items are preserved.
///
/// An unopened bale can be opened to access the pages and child bales contained
/// within it. Until opened an `UnopenedBale` only performs a shallow traversal
/// of its directory. This restricts the information avaiable to just what is
/// needed to build the main navigation tree.
#[derive(Debug)]
pub struct UnopenedBale {
    index_info: IndexInfo,
    assets: Vec<PathBuf>,
    pages: Vec<PathBuf>,
    nested: Vec<PathBuf>,
}

impl UnopenedBale {
    /// Gets the Index Page, if any, for this Bale
    pub fn index(&self) -> Option<&Page> {
        self.index_info.index.as_ref()
    }

    /// Get the footer for this bale, if any
    pub fn footer(&self) -> Option<&String> {
        // TODO: Do we want to render the footer as markdown?
        self.index_info.footer.as_ref()
    }

    pub fn open(self) -> Result<Bale, io::Error> {
        let assets = self.assets;
        let mut items = Vec::with_capacity(self.pages.len() + self.nested.len());

        for page in self.pages {
            items.push(BaleItem::Page(Page::open(&page)?));
        }

        for nested in self.nested {
            // FIXME: what happens if there isn't anything in the directory?
            items.push(BaleItem::Bale(bale_dir(&nested)?));
        }

        // TODO: Sorting of the items before we construct the bale. Items should
        // be ordered by the filename on disk, asciibetically. After this point
        // that information will be lost. Only the slug remains. Slugs no longer
        // contain the decimial prefix, if any, so won't sort correctly.

        Ok(Bale {
            index_info: self.index_info,
            assets,
            items,
        })
    }
}

/// Child Item within a Bale
///
/// Bales can contain both pages, and nested bales. This enum abstracts over
/// that.
#[derive(Debug)]
pub enum BaleItem {
    /// A documentation page
    Page(Page),
    /// A nested Bale
    Bale(UnopenedBale),
}

/// An Opened Bale
///
/// An opened bale exposes the full inforamtion for the bale. Opened bales have
/// inner bales resolved, and pages loaded.
#[derive(Debug)]
pub struct Bale {
    index_info: IndexInfo,
    assets: Vec<PathBuf>,
    items: Vec<BaleItem>,
}

impl Bale {
    /// Get the index page, if any, for this bale
    pub fn index(&self) -> Option<&Page> {
        self.index_info.index.as_ref()
    }

    /// Get the footer for this bale, if any
    pub fn footer(&self) -> Option<&String> {
        self.index_info.footer.as_ref()
    }

    /// Iterate over the assets within this bale
    pub fn assets(&self) -> impl Iterator<Item = &PathBuf> {
        self.assets.iter()
    }

    /// Iterate over the items within this bale
    pub fn items(&self) -> impl Iterator<Item = &BaleItem> {
        self.items.iter()
    }

    /// Unwrap the bale and return the inner items
    pub fn into_items(self) -> Vec<BaleItem> {
        self.items
    }
}

/// Bale a Directory
///
/// Waks a directories contents and tries to build a bale from them. Markdown
/// files are added as pages. Index files are added
pub fn bale_dir<P: AsRef<Path>>(path: P) -> Result<UnopenedBale, io::Error> {
    let mut index = None;
    let mut footer = None;
    let mut pages = Vec::new();
    let mut assets = Vec::new();
    let mut nested = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path().clone();

        if path.is_file() {
            match utils::normalised_path_ext(&path).as_deref() {
                Some("md" | "markdown" | "mdown") => match utils::normalised_stem(&path).as_deref()
                {
                    Some("index" | "readme") => index = Some(path),
                    Some("footer") => footer = Some(fs::read_to_string(path)?),
                    _ => pages.push(path),
                },
                _ => assets.push(path),
            }
        } else {
            nested.push(path);
        }
    }

    let index = match index {
        Some(path) => Some(Page::open(path)?),
        None => None,
    };

    Ok(UnopenedBale {
        index_info: IndexInfo { index, footer },
        assets,
        pages,
        nested,
    })
}
