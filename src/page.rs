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
    toc: Vec<TocElement>,
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
        self.toc.iter()
            .filter_map(|e| {
                match e {
                    &TocElement::Heading(ref h) => Some(h.plain_header()),
                    _ => None,
                }
            })
            .nth(0)
            .unwrap_or("".into()).into()
    }

    fn write_body<T: Write>(&self, file: &mut T) -> io::Result<()> {

        // TODO: render the TOC into an HTML `ol`.
        let rendered_toc = String::new();

        let parts = self.toc
            .iter()
            .map(|e| {
                match e {
                    &TocElement::Html(ref s) => s.to_owned(),
                    &TocElement::Heading(ref h) => {
                        format!("<h{0}>{1}</h{0}>", h.level, h.contents)
                    },
                    &TocElement::TocReference => rendered_toc.clone()
                }
            })
            .collect::<Vec<_>>();

        let rendered = &parts[..]
            .concat();
            
        write!(file, "{}", rendered).unwrap();

        Ok(())
    }
}

impl<'a> Page<'a> {
    pub fn new(path: &'a Path) -> Self {

        let source = ::util::read_file_to_string(path);
        let events = Parser::new(&source).collect::<Vec<_>>();
        let toc = parse_toc(events.into_iter());

        Page {
            path: path,
            toc: toc,
        }
    }
}
