//! The Baler
//!
//! The baler is responsible for walking a given node in the directory tree and
//! producing bales. A bale is a collection of one or more pages, and zero or
//! more assets. Bales are the main unit of navigation for Docket documentation
//! sets.
//!
//! A bale is a lazy item. Baling a directory scans _only_ the directory that is
//! opened. The index file for the bale is calcualted. Remaining items in the
//! bale are lazily loaded when the bale is later broken open. This allows bales
//! to be used in a depth-first walk of the documentation tree without paying
//! the cost of loading all items within all bales up front.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use log::info;

use crate::{asset::Asset, doctree::Page, utils};

/// Information about the bale's index. This is the eagerly opned portion of the
/// bale.
#[derive(Debug)]
struct IndexInfo {
    /// The index page, if the bale has one
    index: Option<Page>,

    /// The footer text, as markdown, if the bale has one specified.
    footer: Option<String>,
}

/// Navigation information for a given bale item
///
/// FIXME: This feels clunky. Do we need to break nav out into a separate
/// module? How do render contexts play with bale items?
#[derive(Debug, Clone)]
pub(crate) struct NavInfo {
    /// The slug for the navigation item. This is the path component that should
    /// be used to link to the item when rendering.
    pub slug: String,

    /// The user-facing part of the navigation item. This should be presented as
    /// the text of any links to this item.
    pub title: String,
}

/// An Unopened Bale
///
/// Represents an interior node in the tree of documentaiton. Bales contain an
/// index page, a number of child pages, and a number of assets. Assets are
/// carried with the bale to ensure that relative paths to images and other
/// items are preserved.
///
///
/// An unopened bale can be broken open to access the inner items within the
/// bale. Until opened an `UnopenedBale` only performs a shallow traversal of
/// its directory. This restricts the information available to just that needed
/// to build the bale's own navigation item within the navigation tree.
#[derive(Debug)]
pub struct UnopenedBale {
    index_info: IndexInfo,
    nav_info: NavInfo,
    assets: Vec<PathBuf>,
    pages: Vec<PathBuf>,
    nested: Vec<PathBuf>,
}

impl UnopenedBale {
    /// Gets the Index Page, if any, for this bale
    pub fn index(&self) -> Option<&Page> {
        self.index_info.index.as_ref()
    }

    /// Get the footer, if any, for this bale
    pub fn footer(&self) -> Option<&String> {
        // TODO: Do we want to render the footer as markdown?
        self.index_info.footer.as_ref()
    }

    /// Get the navigation entry for this bale
    ///
    /// FIXME: Do nav items need to live _insite_ bales and pages, or should
    /// they move one level higher in the tree?
    pub fn nav_info(&self) -> &NavInfo {
        &self.nav_info
    }

    /// Break open a bale
    ///
    /// Walks the contents of the bale and builds pages for them. Assets are
    /// identified and grouped. Child bales are identified and an unopend bale
    /// is created for each.
    pub fn break_open(self) -> Result<Bale, io::Error> {
        info!(
            "Breaking open bale {} ({})",
            self.nav_info.title, self.nav_info.slug
        );

        let mut assets: Vec<_> = self.assets.into_iter().map(Asset::path).collect();
        let mut items = Vec::with_capacity(self.pages.len() + self.nested.len());

        for page in self.pages {
            items.push(BaleItem::Page(Page::open(&page)?));
        }

        for nested in self.nested {
            let bale = bale_dir(&nested)?;
            if bale.index_info.index.is_none() && bale.pages.is_empty() {
                info!("Inner item {:?} is not a bale, adding as an asset", nested);
                assets.push(Asset::path(nested));
            } else {
                items.push(BaleItem::Bale(bale));
            }
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
    assets: Vec<Asset>,
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
    pub fn assets(&self) -> impl Iterator<Item = &Asset> {
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

    // Walk the items in the directory and collect them into the initial
    // unsorted bale contents. We're just using raw paths at this point to refer
    // to all the bale's contents.
    for entry in fs::read_dir(&path)? {
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

    // If we have an index item in this bale then open the page. We need this to
    // know the bale's intended title for navigation purposes.
    let index = match index {
        Some(path) => Some(Page::open(path)?),
        None => None,
    };

    let nav_info = NavInfo {
        slug: utils::slugify_path(&path),
        // FIXME: need to generate a title for folders where we have documents
        // but no index.
        title: match &index {
            Some(page) => page.title().to_owned(),
            None => String::from("FIXME fallback title"),
        },
    };

    Ok(UnopenedBale {
        nav_info,
        index_info: IndexInfo { index, footer },
        assets,
        pages,
        nested,
    })
}
