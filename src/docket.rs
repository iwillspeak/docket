use std::path::{Path, PathBuf};
use glob::glob;

/// Docket
///
/// Represents a documentation set. Repsonsible for finding files on
/// disk which are part of a given set of documentation and rendering
/// them out to HTML files.
#[derive(Debug)]
pub struct Docket {
    /// The index, if there is one
    index: Option<PathBuf>,

    /// The page footer, if there is one
    footer: Option<PathBuf>,

    /// The pages, in order
    pages: Vec<PathBuf>,
}

/// Checks if this is the expected file
fn is_file(path: &Path, name: &str) -> bool {
    path.file_stem()
        .map(|p| p == name)
        .unwrap_or(false)
}

impl Docket {

    /// Create a Docket Instance
    ///
    /// ## Arguments
    ///
    /// The path to search for documentation. Markdown files in this
    /// directory will be used for documentation.
    pub fn new(doc_path: &Path) -> Self {
        println!("Searching for docs in {:?}", doc_path);
        if !doc_path.is_dir() {
            // TODO: Return Result<> instead?
            panic!("Not a directory");
        }

        let pattern = doc_path.join("*.md");
        // This seems a bit unwrappy..
        let matches = glob(pattern.to_str().unwrap())
            .unwrap()
            .filter_map(|p| p.ok());

        let mut index = None;
        let mut footer = None;
        let mut pages = Vec::new();

        for path in matches.into_iter() {
            match path {
                ref p if is_file(&p, "footer") => footer = Some(p.to_owned()),
                ref p if is_file(&p, "index") => index = Some(p.to_owned()),
                _ => pages.push(path),
            }
        }
        Docket {
            index: index,
            footer: footer,
            pages: pages,
        }
    }

    /// Render to Html
    ///
    /// Creates a tree of HTML files in the given `output` directory.
    pub fn render(self, output: &Path) {
        println!("Rendering docs to {:?} ({:?})", output, self);
    }
}
