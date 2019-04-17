#![deny(missing_docs)]

use crate::asset::Asset;
use crate::index::Index;
use crate::page::Page;
use crate::renderer::Renderer;
use crate::util::read_file_to_string;
use failure::Error;
use failure::Fail;
use log::trace;
use pulldown_cmark::{html, Parser};
use rayon::prelude::*;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Fail)]
enum DocketError {
    /// Path used to create the docket instance was bad
    #[fail(display = "Not a directory {:?}", path)]
    PathNotDirectory { path: PathBuf },
}

/// Docket
///
/// Represents a documentation set. Repsonsible for finding files on
/// disk which are part of a given set of documentation and rendering
/// them out to HTML files.
#[derive(Debug)]
pub struct Docket {
    /// The documentation title, based on the directory name
    title: String,

    /// The index, if there is one
    index: Option<PathBuf>,

    /// The page footer, if there is one
    footer: Option<PathBuf>,

    /// The pages, in order
    pages: Vec<PathBuf>,

    /// The Assets Files
    assets: Vec<Asset>,
}

/// The raw stylesheet contents
///
/// This string is written to the assets file `style.css` and
/// referenced from each rendered page.
static STYLE: &'static str = include_str!("../style.css");

impl Docket {
    /// Create a Docket Instance
    ///
    /// ## Arguments
    ///
    /// The path to search for documentation. Markdown files in this
    /// directory will be used for documentation.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(doc_path: &Path) -> Result<Self, Error> {
        trace!("Searching for docs in {:?}", doc_path);
        if !doc_path.is_dir() {
            return Err(DocketError::PathNotDirectory {
                path: doc_path.to_owned(),
            }
            .into());
        }

        let title_file = doc_path.join("title");
        let title = if title_file.is_file() {
            read_file_to_string(&title_file)
        } else {
            Path::canonicalize(doc_path)
                .unwrap()
                .components()
                .filter_map(|c| match c {
                    Component::Normal(path) => path.to_owned().into_string().ok(),
                    _ => None,
                })
                .filter(|s| s != "docs")
                .last()
                .unwrap()
        };

        let mut index = None;
        let mut footer = None;
        let mut pages = Vec::new();
        let mut assets = vec![Asset::internal("style.css", &STYLE)];

        for entry in doc_path.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                // We only support file assets at the moment
                continue;
            }

            let name = path.file_name().unwrap().to_string_lossy();
            if name == "titlle" {
                continue;
            }

            if let Some(ext) = path.extension() {
                let extension = ext.to_string_lossy();
                match extension.as_ref() {
                    "md" | "markdown" | "mdown" => match path {
                        ref p if has_stem(&p, "footer") => footer = Some(p.to_owned()),
                        ref p if has_stem(&p, "index") => index = Some(p.to_owned()),
                        _ => pages.push(path.clone()),
                    },
                    _ => assets.push(Asset::path(path.clone())),
                }
            } else {
                assets.push(Asset::path(path.clone()));
            }
        }

        pages.sort();

        Ok(Docket {
            title: title.to_string(),
            index,
            footer,
            pages,
            assets,
        })
    }

    /// Render to Html
    ///
    /// Creates a tree of HTML files in the given `output` directory.
    pub fn render(self, output: &Path) -> Result<(), Error> {
        trace!("Rendering docs to {:?} ({:?})", output, self);
        let footer = self.rendered_footer();

        let renderer = Renderer::new(self.title.clone(), footer);

        let rendered_pages: Vec<_> = self
            .pages
            .par_iter()
            .map(|path| Page::new(&path))
            .map(|p| renderer.render(&p, &output))
            .collect::<Result<Vec<_>, _>>()?;

        let index = Index::new(self.title, self.index, rendered_pages);

        renderer.render(&index, &output)?;

        for asset in self.assets {
            asset.copy_to(&output)?;
        }

        Ok(())
    }

    /// Render the Footer to a String
    fn rendered_footer(&self) -> String {
        let mut footer = String::new();
        if let Some(ref footer_path) = self.footer {
            let contents = read_file_to_string(&footer_path);
            let parsed = Parser::new(&contents);
            html::push_html(&mut footer, parsed);
        }
        footer
    }
}

/// Checks if this is the expected file
fn has_stem(path: &Path, name: &str) -> bool {
    path.file_stem().map(|p| p == name).unwrap_or(false)
}
