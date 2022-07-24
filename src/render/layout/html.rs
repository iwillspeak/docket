use crate::{
    doctree::Page,
    error::Result,
    render::{NavInfo, PageKind, RenderState},
    toc::{Nodes, Toc, TocElement},
};
use pulldown_cmark::HeadingLevel;
use std::{fmt, io::Write};

use super::Layout;

struct Breadcrumbs<'a>(&'a RenderState<'a, 'a>, &'a str);

impl<'a> fmt::Display for Breadcrumbs<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current = Some(self.0);
        let mut stack = Vec::new();
        let mut path = String::from(self.1);

        while let Some(state) = current {
            stack.push((path.clone(), state));
            path.push_str("../");
            current = state.parent();
        }
        write!(f, "<ol class='breadcrumbs'>")?;
        if let Some((path, _)) = stack.pop() {
            write!(
                f,
                "<li class='primary'><a href='{}'><strong>{}</strong></a></li>",
                path,
                self.0.ctx().site_name
            )?;
        }
        while let Some((path, state)) = stack.pop() {
            write!(
                f,
                "<li><a href='{}'>{}</a></li>",
                path,
                state.current_bale().title()
            )?;
        }
        write!(f, "</ol>")
    }
}

struct Content<'a>(&'a Toc);

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
                    "<{level} id='{slug}'><a href='#{slug}'>{heading}</a></{level}>",
                    level = &nested.heading.level,
                    slug = &nested.heading.slug,
                    heading = &nested.heading.contents,
                )?,
            }
        }
        Ok(())
    }
}

struct RenderedToc<'a>(&'a Toc, HeadingLevel);

impl<'a> fmt::Display for RenderedToc<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        render_toc_to(f, self.0.nodes(), self.1)
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
                node.heading.slug, node.heading.contents
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
        .unwrap_or("<p>Rendered by <a href='https://github.com/iwillspeak/docket/'>Docket</a></p>")
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
            r##"<!DOCTYPE html>
<html>
<head>
    <title>{site_name} | {page_title}</title>
    <meta name="viewport" content="width=device-wdith,initial-scale=1">
    <meta charset="UTF-8">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Montserrat&display=swap" rel="stylesheet">
</head>
<body>
    <heading>{breadcrumbs}</heading>
    <section id="main-contnet">
        <nav id="site-nav">
            <ul>
                <li><a href='{nav_prefix}'>{bale_title}</a>
                    {navs}
            </ul>
        </nav>
        <nav id="toc-tree">
            <h2>What's on this Page</h2>
            {toc}
        </nav>
        <main>
            <article id="documetnation">
                {content}
            </article>
        </main>
    </section>
    <footer>{footer}</footer>
</body>
</html>"##,
            site_name = state.ctx().site_name,
            breadcrumbs = Breadcrumbs(state, nav_prefix),
            page_title = page.title(),
            bale_title = state.current_bale().title(),
            nav_prefix = nav_prefix,
            navs = Navs(&state.navs, nav_prefix),
            toc = RenderedToc(page.content(), HeadingLevel::H2),
            content = Content(page.content()),
            footer = get_footer(state)
        )?;
        Ok(())
    }
}
