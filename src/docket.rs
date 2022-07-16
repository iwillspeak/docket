use std::{
    fs, io,
    path::{self, Path},
};

use crate::{
    baler::{self, BaleItem, UnopenedBale},
    error::Error,
};

use crate::error::Result as DocketResult;

/// Docket
///
/// Represents a documentation set. Responsible for opening the bales of
/// documentation and traversing them to render out to HTML.
#[derive(Debug)]
pub struct Docket {
    title: String,
    root_bale: UnopenedBale,
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
        let title = title_from_path(path.as_ref())?;
        let root_bale = baler::bale_dir(path)?;
        Ok(Docket { title, root_bale })
    }

    /// Render the documentation set to the given location
    pub fn render<P: AsRef<Path>>(self, target: P) -> Result<(), io::Error> {
        println!("RENDERING {0}", self.title);
        walk_bale(self.root_bale)?;
        Ok(())
    }
}

/// Calculate the title of the documentation set from the given path.
fn title_from_path(path: &Path) -> Result<String, io::Error> {
    let title_file = path.join("title");
    Ok(if title_file.is_file() {
        fs::read_to_string(title_file)?
    } else {
        Path::canonicalize(path)
            .unwrap()
            .components()
            .filter_map(|c| match c {
                path::Component::Normal(path) => path.to_owned().into_string().ok(),
                _ => None,
            })
            .filter(|s| s != "docs")
            .last()
            .unwrap()
    })
}

// JUNK FUNCTIONS

fn walk_bale(bale: UnopenedBale) -> Result<(), io::Error> {
    println!("got bale with index {:?}", bale.index());
    let opend = bale.open()?;
    for item in opend.into_items() {
        match item {
            BaleItem::Page(page) => println!("Page {:?}", page),
            BaleItem::Bale(bale) => walk_bale(bale)?,
        }
    }
    Ok(())
}
