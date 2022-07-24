use std::{fmt, io::Write};

use pulldown_cmark::HeadingLevel;

use crate::{
    doctree::Page,
    error::Result,
    render::{NavInfo, PageKind, RenderState},
    toc::{Nodes, Toc, TocElement},
};

use super::Layout;

struct Content<'a>(&'a Toc);

impl<'a> Content<'a> {
    fn new(toc: &'a Toc) -> Self {
        Content(toc)
    }
}

impl<'a> fmt::Display for Content<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for element in self.0.walk_elements() {
            match element {
                TocElement::Html(htm) => htm.fmt(f)?,
                TocElement::TocReference => render_toc_to(f, self.0.nodes(), HeadingLevel::H3)?,
                // We only need to write the heading here, the contents will be
                // handled by the recurse from the walker.
                TocElement::Node(nested) => write!(
                    f,
                    "<a href='#{slug}'><span id='{slug}'>{heading}</a>",
                    slug = &nested.heading.slug,
                    heading = &nested.heading.contents,
                )?,
            }
        }
        Ok(())
    }
}

/// Render the TOC
///
/// This walks the list of `TocElement`s and renders them as a list of links
/// using their clean text for the link body.
fn render_toc_to(f: &mut fmt::Formatter<'_>, nodes: Nodes, limit: HeadingLevel) -> fmt::Result {
    write!(f, "<ul class='toc'>")?;
    for node in nodes {
        if node.heading.level <= limit {
            write!(
                f,
                "<li><a href='#{0}'>{1}</a>",
                node.heading.slug, node.heading.clean_text
            )?;

            render_toc_to(f, node.nodes(), limit)?;
        }
    }
    write!(f, "</ul>")
}

/// Renderable struct
struct Navs<'a>(&'a [NavInfo], &'a str);

impl<'a> fmt::Display for Navs<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.len() > 0 {
            write!(f, "<ul>")?;
            for nav in self.0.iter() {
                write!(
                    f,
                    "<li><a href='{prefix}{slug}'>{title}</a>",
                    title = nav.title,
                    prefix = self.1,
                    slug = nav.slug
                )?;
            }
            write!(f, "</ul>")?;
        }
        Ok(())
    }
}

fn get_footer<'a>(state: &'a RenderState) -> &'a str {
    state
        .current_bale()
        .footer()
        .unwrap_or("<p>Rendered by Docket</p>")
}

/// The HTML Layout
///
/// This struct implements the `Layout` trait to allow rendering pages.
pub(super) struct HtmlLayout;

impl Layout for HtmlLayout {
    fn render(
        &self,
        writer: &mut dyn Write,
        state: &RenderState,
        kind: PageKind,
        page: &Page,
    ) -> Result<()> {
        let nav_prefix = kind.path_to_bale();
        write!(
            writer,
            "<!DOCTYPE html>
<html>
    <body>
        <heading><h1>{site_name}</h1></heading>
        <nav>
            <ul>
                <li><a href='{nav_prefix}'>{bale_title}</a>
                    {navs}
            </ul>
        </nav>
        <main>
            {content}
        </main>
        <footer>{footer}</footer>
    </body>
</html>
        ",
            site_name = state.ctx().site_name,
            bale_title = state.current_bale().title(),
            nav_prefix = nav_prefix,
            navs = Navs(&state.navs, nav_prefix),
            content = Content::new(&page.content()),
            footer = get_footer(state)
        )?;
        Ok(())
    }
}
