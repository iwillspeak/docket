use std::fs::{File, create_dir_all};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use pulldown_cmark::*;
use util;
use toc::*;

static STYLE: &'static str = include_str!("../style.css");

/// Page to be Rendered
pub struct Page<'a> {
    path: &'a Path,
}

/// Result of page rendering
pub struct PageInfo {
    /// Title of the page.
    pub title: String,

    /// The slug of the page.
    pub slug: String,

    /// The path this page was rendered to.
    pub path: PathBuf,
}

impl<'a> Page<'a> {
    pub fn new(path: &'a Path) -> Self {
        Page { path: path }
    }

    /// Render this page to a given path
    ///
    /// Reads the markdown from the file path for this page, renders
    /// the HTML and returns information about the rendered page.
    pub fn render_with_footer(&self, footer: &str, output_dir: &Path) -> PageInfo {

        let source = ::util::read_file_to_string(self.path);
        let events = Parser::new(&source).collect::<Vec<_>>();
        let toc = parse_toc(events.clone().into_iter());
        info!("TOC: {:?}", toc);

        let title = toc[0].plain_header().to_owned();

        // Create a slug for this page. Render to `index.html` in that
        // directory, ensuring that it exists.
        let slug = util::slugify(&title);
        let output_dir = output_dir.join(&slug);
        let output_path = output_dir.join("index.html");
        create_dir_all(&output_dir).unwrap();

        debug!("Rendering to: {:?}", output_path);

        let mut file = File::create(&output_path).unwrap();

        // HTML header, containing hardcoded CSS
        write!(
            file,
            "<html>
  <head>
    <title>{}</title>
    <style>{}</style>
  </head>
<body>",
            title,
            &STYLE
        ).unwrap();

        // Render the main part of the page
        let mut rendered = String::new();
        html::push_html(&mut rendered, events.into_iter());
        write!(file, "{}", rendered).unwrap();

        // footer finishes body.
        write!(file, "<footer>{}</footer></body>", footer).unwrap();

        PageInfo {
            title: title,
            slug: slug,
            path: output_path,
        }
    }
}
