use std::path::Path;

/// Docket
///
/// Represents a documentation set. Repsonsible for finding files on
/// disk which are part of a given set of documentation and rendering
/// them out to HTML files.
pub struct Docket;

impl Docket {

    /// Create a Docket Instance
    ///
    /// ## Arguments
    ///
    /// The path to search for documentation. Markdown files in this
    /// directory will be used for documentation.
    pub fn new(doc_path: &Path) -> Self {
        println!("Searching for docs in {:?}", doc_path);
        Docket
    }

    /// Render to Html
    ///
    /// Creates a tree of HTML files in the given `output` directory.
    pub fn render(self, output: &Path) {
        println!("Rendering docs to {:?}", output);
    }
}
