use std::io::prelude::*;
use std::io;
use std::path::{Path, PathBuf};
use std::borrow::Cow;
use pulldown_cmark::*;
use renderable::Renderable;
use util;
use toc::*;

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

impl<'a> Renderable for Page<'a> {
    fn get_slug(&self) -> String {
        util::slugify_path(self.path)
    }

    fn get_title<'t>(&'t self) -> Cow<'t, str> {
        self.get_slug().into()
    }

    fn write_body<T: Write>(&self, file: &mut T) -> io::Result<()> {

        let source = ::util::read_file_to_string(self.path);
        let events = Parser::new(&source).collect::<Vec<_>>();
        let toc = parse_toc(events.clone().into_iter());
        info!("TOC: {:?}", toc);

        // TODO: get this exposed in `get_title`
        let _title = toc[0].plain_header().to_owned();

        // Render the main part of the page
        let mut rendered = String::new();
        html::push_html(&mut rendered, events.into_iter());
        write!(file, "{}", rendered).unwrap();

        Ok(())
    }
}

impl<'a> Page<'a> {
    pub fn new(path: &'a Path) -> Self {
        Page { path: path }
    }
}
