use std::{fmt, io::Write};

use crate::{
    doctree::Page,
    error::Result,
    render::{NavInfo, PageKind, RenderState},
    toc::TocElement,
};

use super::Layout;

struct Content<'a>(&'a [TocElement], &'a [TocElement]);

impl<'a> fmt::Display for Content<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tree = self.0;
        let current = self.1;
        for element in current {
            match element {
                TocElement::Html(htm) => write!(f, "{}", htm)?,
                TocElement::TocReference => render_toc_to(f, &tree)?,
                TocElement::Node(nested) => write!(
                    f,
                    "<a id='{slug}' href='#{slug}'>{heading}</a>{content}",
                    slug = &nested.heading.slug,
                    heading = &nested.heading.contents,
                    content = Content(tree, &nested.contents)
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
fn render_toc_to(f: &mut fmt::Formatter<'_>, tree: &[TocElement]) -> fmt::Result {
    write!(f, "<ul class='toc'>")?;
    for element in tree {
        if let TocElement::Node(node) = element {
            write!(
                f,
                "<li><a href='#{0}'>{1}</a>",
                node.heading.slug, node.heading.clean_text
            )?;

            // TODO: We should recurse here and render out any nested headings.
            // This might be best if the TOC rendering was wrapped in a
            // renderable struct.
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
            // TODO: some more ergonomic constructor here.
            content = Content(&page.content().0, &page.content().0),
            footer = get_footer(state)
        )?;
        Ok(())
    }
}
