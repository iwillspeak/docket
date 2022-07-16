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

/// Baler Error Type
///
/// Repersents the different kinds of errors that can be encountered when
/// walking a directory to produce a  bale.
#[derive(Debug)]
pub(crate) enum BalerError {
    /// An IO Error
    Io(io::Error),
    /// Am unimplemented item
    Uninmpl,
}

impl std::fmt::Display for BalerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BalerError::Io(io) => io.fmt(f),
            BalerError::Uninmpl => write!(f, "Unimplemented item"),
        }
    }
}

impl From<io::Error> for BalerError {
    fn from(err: io::Error) -> Self {
        BalerError::Io(err)
    }
}

impl std::error::Error for BalerError {}

/// A Bale
///
/// Represents an interior node in the tree of documentation. Bales contain an
/// index page, any number of child pages, and any number of assets. Assets are
/// carried with the bale to ensure that relative paths to images and other
/// items are preserved.
#[derive(Debug)]
pub(crate) struct Bale {
    index: Option<PathBuf>,
    footer: Option<PathBuf>,
    assets: Vec<PathBuf>,
    pages: Vec<PathBuf>,
    nested: Vec<PathBuf>,
}

impl Bale {
    /// Get the Assets for a Bale
    /// 
    /// Returns an iterator over the asset paths this bale contains.
    pub(crate) fn assets(&self) -> impl Iterator<Item=&PathBuf> {
        self.assets.iter()
    }

    pub(crate) fn pages(&self) -> impl Iterator<Item=&PathBuf> {
        self.pages.iter()
    }
}

/// Open a Bale
///
/// Waks a directories contents and tries to build a bale from them. Markdown
/// files are added as pages. Index files are added
pub(crate) fn open_bale<P: AsRef<Path>>(path: P) -> Result<Bale, BalerError> {
    let mut index = None;
    let mut footer = None;
    let mut pages = Vec::new();
    let mut assets = Vec::new();
    let mut nested = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path().clone();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                match ext.to_string_lossy().to_ascii_lowercase().as_ref() {
                    "md" | "markdown" | "mdown" => {
                        if let Some(stem) = path.file_stem() {
                            let stem = stem.to_string_lossy().to_ascii_lowercase();
                            match stem.as_ref() {
                                "index" | "readme" => index = Some(path),
                                "footer" => footer = Some(path),
                                _ => pages.push(path),
                            }
                        } else {
                            pages.push(path)
                        }
                    }
                    _ => assets.push(path),
                }
            } else {
                assets.push(path);
            }
        } else {
            nested.push(path);
        }
    }

    Ok(Bale {
        index,
        footer,
        assets,
        pages,
        nested,
    })
}
