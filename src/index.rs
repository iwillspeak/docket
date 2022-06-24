use crate::page::PageInfo;
use crate::renderable::Renderable;
use crate::search::TermFrequenciesIndex;
use crate::util::read_file_to_string;
use log::debug;
use pulldown_cmark::{html, Parser};
use std::borrow::Cow;
use std::io::{self, Write};
use std::path::PathBuf;

pub struct Index {
    /// The project title
    title: String,

    /// The path to the markdown source
    path: Option<PathBuf>,

    /// Pages to include in this index.
    pages: Vec<PageInfo>,
}

impl Renderable for Index {
    fn get_slug(&self) -> String {
        "".into()
    }

    fn get_title(&self) -> Cow<'_, str> {
        (&self.title[..]).into()
    }

    fn write_header<T: Write>(&self, file: &mut T, title: &str) -> io::Result<()> {
        write!(
            file,
            r#"<header class="index-heading"><h1><a href="">{}</a></h1></header>"#,
            title
        )
    }

    fn write_body<T: Write>(&self, file: &mut T) -> io::Result<()> {
        if let Some(ref index_md) = self.path {
            debug!("found index file, rendering");
            let contents = read_file_to_string(&index_md);
            let parsed = Parser::new(&contents);
            let mut rendered = String::new();
            html::push_html(&mut rendered, parsed);
            write!(file, "{}", rendered)?;
        }

        // Placehoder for the JS search
        write!(file, r#"<div id="docket-search"></div>"#)?;

        // List of pages in the index
        debug!("listing pages in index");
        write!(file, "<h2>Table of Contents</h2>")?;
        write!(file, r#"<ol class="index-toc">"#)?;
        for page in self.pages.iter() {
            write!(
                file,
                r#"<li><a href="{}/">{}</a></li>"#,
                page.slug, page.title
            )?;
        }
        write!(file, "</ol>")?;

        Ok(())
    }

    fn path_to_root(&self) -> Cow<'_, str> {
        ".".into()
    }

    fn get_search_index(&self) -> Option<TermFrequenciesIndex> {
        None
    }
}

impl Index {
    /// Create a New Index Page
    ///
    /// # Parameters
    ///  * `title` - The title of the index page
    ///  * `path` - The path to ths Markdown source
    ///  * `pages` - The pages to link to from this index
    pub fn new(title: String, path: Option<PathBuf>, pages: Vec<PageInfo>) -> Self {
        Index { title, path, pages }
    }
}
