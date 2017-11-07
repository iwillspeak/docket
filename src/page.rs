use std::path::Path;
use pulldown_cmark::*;
use toc::*;

/// Page to be Rendered
pub struct Page<'a> {
    path: &'a Path
}

/// Result of page rendering
pub struct PageInfo;

impl <'a> Page<'a> {
    
    pub fn new(path: &'a Path) -> Self {
        Page { path: path }
    }
    
    /// Render this page to a given path
    ///
    /// Reads the markdown from the file path for this page, renders
    /// the HTML and returns information about the rendered page.
    pub fn render_with_footer(&self, _footer: &str, _path: &Path) -> PageInfo {
        debug!("Rendering page: {:?}", self.path);
        let source = ::util::read_file_to_string(self.path);
        let mut parser = Parser::new(&source);

        let _toc = parse_toc(&mut parser);

        for event in parser {
            println!("ev: {:?}", event);
        }

        PageInfo
    }
}
