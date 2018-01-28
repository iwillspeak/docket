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
    title: String,
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

fn render_tree<'a, W: Write, I: Iterator<Item = &'a TocElement>>(
    tree: I,
    toc: &str,
    w: &mut W,
) -> io::Result<()> {
    for e in tree {
        match e {
            &TocElement::Heading(ref h, ref children) => {
                write!(
                    w,
                    "<h{0} id=\"{1}\"><a href=\"#{1}\">{2}</a></h{0}>",
                    h.level, h.slug, h.contents
                )?;
                render_tree(children.iter(), toc, w)?;
            }
            &TocElement::Html(ref html) => {
                write!(w, "{}", html)?;
            }
            &TocElement::TocReference => {
                write!(w, "{}", toc)?;
            }
        }
    }

    Ok(())
}

fn toc_to_html<'a, I>(tree: I, depth: i32) -> String
where
    I: Iterator<Item = &'a TocElement>,
{
    let mut rendered = String::new();

    rendered.push_str("<ol>");
    for e in tree {
        if let &TocElement::Heading(ref h, ref children) = e {
            if h.level > depth {
                continue;
            }
            rendered.push_str(&format!("<li><a href=\"#{0}\">{1}</a>", h.slug, h.contents));
            if h.level < depth {
                let child_toc = toc_to_html(children.iter(), depth);
                rendered.push_str(&child_toc);
            }
            rendered.push_str("</li>")
        }
    }
    rendered.push_str("</ol>");

    rendered
}

impl<'a> Renderable for Page<'a> {
    fn get_slug(&self) -> String {
        util::slugify_path(self.path)
    }

    fn get_title<'t>(&'t self) -> Cow<'t, str> {
        Cow::Borrowed(&self.title)
    }

    fn write_header<T: Write>(&self, file: &mut T, title: &str) -> io::Result<()> {
        write!(
            file,
            "<header class=\"page-heading\"><h1><a href=\"#\">{}</h1><a class=\"home-link\" href=\"../\">{}</a></header>",
            self.get_title(),
            title
        )
    }

    fn write_body<T: Write>(&self, file: &mut T) -> io::Result<()> {
        let toc = toc_to_html(self.toc.iter(), 3);
        render_tree(self.toc.iter(), &toc, file)
    }

    fn path_to_root(&self) -> Cow<str> {
        "..".into()
    }
}

impl<'a> Page<'a> {
    pub fn new(path: &'a Path) -> Self {
        let source = ::util::read_file_to_string(path);
        let events = Parser::new(&source).collect::<Vec<_>>();
        let mut toc = parse_toc(events.into_iter());
        let mut title = path.file_stem().unwrap().to_string_lossy().to_string();

        if toc.len() == 1 {
            if let TocElement::Heading(main_header, children) = toc.remove(0) {
                title = main_header.contents;
                toc = children;
            }
        }

        debug!("TOC: {:?}", toc);

        Page {
            path: path,
            title: title,
            toc: toc,
        }
    }
}
