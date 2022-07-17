use std::{
    fs, io,
    path::{self, Path},
};

use log::trace;

use crate::{
    doctree::{self, Bale},
    error::{Error, Result as DocketResult},
    render,
};

/// Docket
///
/// Represents a documentation set. Responsible for opening the bales of
/// documentation and traversing them to render out to HTML.
#[derive(Debug)]
pub struct Docket {
    title: String,
    doctree_root: Bale,
}

impl Docket {
    /// Open the given `path` as a documentaiton collection
    ///
    /// Once opned a documentaiton collection has a title, and can be rendreed
    /// to a target path.
    pub fn open<P: AsRef<Path>>(path: P) -> DocketResult<Self> {
        if !path.as_ref().is_dir() {
            Err(Error::SourcePathNotADirectory(path.as_ref().into()))?;
        }
        Ok(Docket {
            title: title_from_path(path.as_ref())?,
            doctree_root: doctree::open(&path)?,
        })
    }

    /// Render the documentation set to the given location
    pub fn render<P: AsRef<Path>>(self, target: P) -> DocketResult<()> {
        trace!(
            "Rendering documentation for {} to {:?}",
            self.title,
            target.as_ref()
        );
        render::render(target, self.doctree_root)?;
        Ok(())
    }
}

/// Calculate the title of the documentation set from the given path.
fn title_from_path(path: &Path) -> std::result::Result<String, io::Error> {
    let title_file = path.join("title");
    Ok(if title_file.is_file() {
        fs::read_to_string(title_file)?
    } else {
        Path::canonicalize(path)?
            .components()
            .filter_map(|c| match c {
                path::Component::Normal(path) => path.to_owned().into_string().ok(),
                _ => None,
            })
            .filter(|s| s != "docs")
            .last()
            .unwrap_or_else(|| String::from("Documentation"))
    })
}
