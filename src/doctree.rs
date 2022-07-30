//! The Documentation Tree
//!
//! This module defines the types that model out the tree of documentation to be
//! rendered. The tree is defined in such a way that it can be lazily traversed
//! to avoid pulling the whole thing into memory at the beginning of rendering.
//!
//! The documentation tree is made up of two things: bales and pages. Bales form
//! the interior nodes of the tree, and pages the leaves.

use std::{
    borrow::Borrow,
    fs,
    path::{Path, PathBuf},
    result,
};

use log::info;

use crate::{
    asset::Asset,
    error::Result,
    search,
    toc::Toc,
    utils::{self, slugify_path},
};

/// A Doctree Item
///
/// Represents the kinds of item that can appear within the doctree.
pub(crate) enum DoctreeItem {
    /// A leaf page
    Page(Page),

    /// An unopened bale
    Bale(Bale),
}

/// A Documentation Page
///
/// Each documentaiton page is mad eup of two items: a simple `slug` which
/// should be used to refer to the page in the navigation tree, and a TOC. The
/// TOC is a heirachical represenattion of the contents of the page. A TOC can
/// be traversed to inspect the structure of the page; and rendered to HTML.
#[derive(Debug)]
pub(crate) struct Page {
    slug: String,
    title: String,
    tree: Toc,
}

impl search::SearchableDocument for Page {
    /// Get the title for this page
    fn title(&self) -> &str {
        &self.title
    }

    /// Get the slug for this page
    fn slug(&self) -> &str {
        &self.slug
    }

    /// Get the search index for the given page
    fn search_index(&self) -> Option<&search::TermFrequenciesIndex> {
        Some(&self.content().search_index())
    }
}

impl Page {
    /// Open a Page
    ///
    /// Loads the contents of the given file and parses it as markdown.
    pub fn open<P: AsRef<Path>>(path: P) -> result::Result<Self, std::io::Error> {
        let markdown = fs::read_to_string(&path)?;
        Ok(Self::from_parts(path, markdown))
    }

    /// Construct a Page from Constituent Parts
    ///
    /// Builds the TOC tree for the given page, and returns the opened and
    /// parsed page.
    fn from_parts<P: AsRef<Path>, M: Borrow<str>>(path: P, markdown: M) -> Self {
        let slug = utils::slugify_path(&path);
        let tree = Toc::new(markdown.borrow());
        let title = tree.primary_heading().cloned().unwrap_or_else(|| {
            path.as_ref()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        });
        Page { slug, title, tree }
    }

    /// Get the title for this page
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get the slug for this page
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// Get the content
    pub fn content(&self) -> &Toc {
        &self.tree
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
    frontispiece: Frontispiece,
    /// The paths we suspect to be page items
    pages: Vec<PathBuf>,
    /// The paths we know to be assets
    assets: Vec<PathBuf>,
    /// The paths we susepct to be child bales
    nested: Vec<PathBuf>,
}

impl Bale {
    /// Create a new Bale
    ///
    /// Wraps the given `path` as a bale. This performs a shallow traversal of
    /// the directory to find the index to produce the `Frontispiece`. The full
    /// contents of the bale can be retrieved by `Bale::break_open`.
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
            frontispiece: Frontispiece::new(path, index, footer),
            pages,
            assets,
            nested,
        })
    }

    /// Break Open the Bale
    ///
    /// This reifies the contents of the bale. Inner items are converted into
    /// real pages and bales.
    pub fn break_open(self) -> Result<(Frontispiece, Vec<Asset>, Vec<DoctreeItem>)> {
        info!(
            "Breaking open bale {} ({})",
            self.frontispiece.title,
            self.frontispiece.slug(),
        );

        let mut assets: Vec<_> = self.assets.into_iter().map(Asset::path).collect();
        let mut items = Vec::with_capacity(self.pages.len() + self.nested.len());

        for page in self.pages {
            items.push((
                utils::normalised_stem(&page),
                DoctreeItem::Page(Page::open(page)?),
            ));
        }

        for nested in self.nested {
            let bale = Bale::new(&nested)?;
            if bale.frontispiece.index.is_none() && bale.pages.is_empty() {
                info!(
                    "Inner item {:?} does not appear to be able. Adding as an asset",
                    &nested
                );
                assets.push(Asset::path(nested));
            } else {
                items.push((utils::normalised_stem(&nested), DoctreeItem::Bale(bale)));
            }
        }

        // Sort the items by their origional path. This allows files on disk to
        // be given a prefix that is stripped off in slugification but still
        // affects the item's order within the documentation tree.
        items.sort_by_cached_key(|(k, _)| k.clone());

        Ok((
            self.frontispiece,
            assets,
            items.into_iter().map(|(_, i)| i).collect(),
        ))
    }

    /// Get the Frontispiece for this bale
    pub(crate) fn frontispiece(&self) -> &Frontispiece {
        &self.frontispiece
    }
}

/// Bale Frontispiece
///
/// The frontispiece represents the eagerly loaded portion of the bale. Bales
/// are broken open into three parts: frontispiece, assets, and inner items.
/// This type is used to group together the index.
#[derive(Debug)]
pub(crate) struct Frontispiece {
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

impl Frontispiece {
    /// Create a new Frontispiece
    ///
    /// This picks a title and slug for the bale based on the bale's path.
    fn new<P: AsRef<Path>>(path: P, index: Option<Page>, footer: Option<String>) -> Frontispiece {
        let title = match &index {
            Some(page) => page.title.clone(),
            None => utils::prettify_dir(&path).expect("Could not create a title"),
        };
        let footer = footer.map(|text| {
            let mut output = String::new();
            pulldown_cmark::html::push_html(&mut output, pulldown_cmark::Parser::new(&text));
            output
        });
        Frontispiece {
            title,
            slug: slugify_path(path),
            index,
            footer,
        }
    }

    /// Get the bale's slug
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// Get the bale's title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get a reference to the index page of this bale, if any
    pub fn index_page(&self) -> Option<&Page> {
        self.index.as_ref()
    }

    /// Get the page's footer
    pub fn footer(&self) -> Option<&str> {
        self.footer.as_deref()
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
mod test {

    use super::*;
    use std::path::PathBuf;

    #[test]
    fn page_has_search_terms() {
        let path = PathBuf::from("foo/bar.md");
        let page = Page::from_parts(&path, "Some sample text in some text");

        let index = page.content().search_index().as_raw();
        assert_ne!(0, index.len());
        let some_fq = index.get("some").cloned().unwrap_or_default();
        let sample_fq = index.get("sample").cloned().unwrap_or_default();
        let text_fq = index.get("text").cloned().unwrap_or_default();
        assert_eq!(some_fq, text_fq);
        assert!(some_fq > sample_fq);
    }

    #[test]
    fn index_of_example_markdown() {
        let path = PathBuf::from("foo/bar.md");
        let page = Page::from_parts(
            &path,
            r###"

# Down the Rabbit Hole

Either the well was very deep, or she fell very slowly, for she had
plenty of time as she went down to look about her, and to wonder what
was going to happen next. First, she tried to look down and make out
what she was coming to, but it was too dark to see anything; then she
looked at the sides of the well and noticed that they were filled with
cupboards and book-shelves: here and there she saw maps and pictures
hung upon pegs. She took down a jar from one of the shelves as she
passed; it was labelled "ORANGE MARMALADE," but to her disappointment it
was empty; she did not like to drop the jar for fear of killing
somebody underneath, so managed to put it into one of the cupboards as
she fell past it.

"Well!" thought Alice to herself. "After such a fall as this, I shall
think nothing of tumbling down stairs! How brave they'll all think me at
home! Why, I wouldn't say anything about it, even if I fell off the top
of the house!" (Which was very likely true.)

### Going Down?

Down, down, down. Would the fall _never_ come to an end? "I wonder how
many miles I've fallen by this time?" she said aloud. "I must be getting
somewhere near the centre of the earth. Let me see: that would be four
thousand miles down. I think--" (for, you see, Alice had learnt several
things of this sort in her lessons in the schoolroom, and though this
was not a _very_ good opportunity for showing off her knowledge, as
there was no one to listen to her, still it was good practice to say it
over) "--yes, that's about the right distance--but then I wonder what
Latitude or Longitude I've got to?" (Alice had no idea what Latitude
was, or Longitude either, but thought they were nice grand words to
say.)

        "###,
        );

        assert_eq!("Down the Rabbit Hole", page.title);

        // Check some of the relative frequencies of terms
        let index = page.content().search_index().as_raw();
        assert_ne!(0, index.len());
        let rabbit_fq = index.get("rabbit").cloned().unwrap_or_default();
        assert!(rabbit_fq > 0.0);
        let well_fq = index.get("well").cloned().unwrap_or_default();
        assert!(well_fq > rabbit_fq);
        assert_eq!(
            index.get("distance").cloned().unwrap_or_default(),
            rabbit_fq
        );
        assert!(index.get("down").cloned().unwrap_or_default() > well_fq);

        // Check terms are downcased
        assert_ne!(None, index.get("orange"));
        assert_eq!(None, index.get("MARMALADE"));

        // check we don't have any whitespace or other junk symbols in the index
        assert_eq!(None, index.get(""));
        assert_eq!(None, index.get("!"));
        assert_eq!(None, index.get("-"));
        assert_eq!(None, index.get(" "));
        assert_eq!(None, index.get("\t"));
        assert_eq!(None, index.get("("));
    }
}
