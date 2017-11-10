use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use pulldown_cmark::*;
use toc::*;

static STYLE: &'static str = include_str!("../style.css");

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
    pub fn render_with_footer(&self, footer: &str, path: &Path) -> PageInfo {
        debug!("Rendering page: {:?}", self.path);
        let source = ::util::read_file_to_string(self.path);
        let events = Parser::new(&source).collect::<Vec<_>>();

        // let toc = parse_toc(events.iter());

        // println!("TOC: {:?}", toc);

        for event in events.iter() {
            println!("ev: {:?}", event);
        }

        let mut rendered = String::new();
        rendered.push_str(&format!("<head><style>{}</style></head><body>", &STYLE));
        html::push_html(&mut rendered, events.into_iter());
        rendered.push_str(&format!("<footer>{}</footer></body>", footer));
        let mut file = File::create(path).unwrap();
        write!(file, "{}", rendered).unwrap();
        
        PageInfo
    }
}
