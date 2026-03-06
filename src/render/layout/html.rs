use crate::{
    asset::Asset,
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
                "<li class='primary'><a href='{}'>{}</a></li>",
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

struct Content<'a>(&'a Toc, &'a str);

impl<'a> fmt::Display for Content<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let root = self.1;
        for element in self.0.walk_elements() {
            match element {
                TocElement::Html(htm) => htm.fmt(f)?,
                TocElement::TocReference => render_toc_to(f, self.0.nodes(), HeadingLevel::H3)?,
                // We only need to write the heading here, the contents will be
                // handled by the recurse from the walker.
                TocElement::Node(nested) => write!(
                    f,
                    "<{level} id='{slug}'>{heading}<a class='heading-anchor' href='#{slug}' aria-label='Permalink to this heading'><svg class='anchor-icon' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'><use href='{root}feather-sprite.svg#copy'/></svg></a></{level}>",
                    level = &nested.heading.level,
                    slug = &nested.heading.slug,
                    heading = &nested.heading.contents,
                    root = root,
                )?,
            }
        }
        Ok(())
    }
}

struct RenderedToc<'a>(&'a Toc, HeadingLevel);

impl<'a> fmt::Display for RenderedToc<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.nodes().count() == 1 {
            let inner_node = self.0.nodes().next().unwrap();
            if inner_node.nodes().count() > 0 {
                render_toc_to(f, inner_node.nodes(), self.1)?;

                return Ok(());
            }
        }
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
struct Navs<'a>(&'a [NavInfo], &'a str, &'a str);

impl<'a> fmt::Display for Navs<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        render_nav_list(f, self.0, self.1, self.2)
    }
}

/// Recursively render a navigation list.
///
/// Items without children render as plain links; items with children render
/// as `<details>/<summary>` disclosure widgets with a nested list.
fn render_nav_list(
    f: &mut fmt::Formatter<'_>,
    navs: &[NavInfo],
    prefix: &str,
    root: &str,
) -> fmt::Result {
    if navs.is_empty() {
        return Ok(());
    }
    write!(f, "<ul class='site-nav-list'>")?;
    for nav in navs {
        write!(f, "<li>")?;
        if nav.children.is_empty() {
            write!(
                f,
                "<a href='{prefix}{slug}'>{title}</a>",
                prefix = prefix,
                slug = nav.slug,
                title = nav.title,
            )?;
        } else {
            let child_prefix = format!("{}{}/", prefix, nav.slug);
            write!(
                f,
                "<details><summary><svg class='nav-chevron' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'><use href='{root}feather-sprite.svg#chevron-right'/></svg><a href='{prefix}{slug}'>{title}</a></summary>",
                root = root,
                prefix = prefix,
                slug = nav.slug,
                title = nav.title,
            )?;
            render_nav_list(f, &nav.children, &child_prefix, root)?;
            write!(f, "</details>")?;
        }
        write!(f, "</li>")?;
    }
    write!(f, "</ul>")
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
        let root = state.path_to_root(&kind);
        write!(
            writer,
            r##"<!DOCTYPE html>
<html>
<head>
    <title>{site_name} | {page_title}</title>
    <meta name="viewport" content="width=device-width,initial-scale=1">
    <meta charset="UTF-8">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Montserrat&family=JetBrains+Mono&display=swap" rel="stylesheet">
    <link rel="stylesheet" href="{root}style.css">
    <script src="{root}dark.js" type=module></script>
    <script src="{root}search.js" type=module></script>
    <script src="{root}nav.js" type=module></script>
    <script src="{root}permalink.js" type=module></script>
</head>
<body>
    <header class="site-head">
        <div class="content">
            <button class="nav-toggle" id="nav-toggle" aria-label="Toggle navigation" aria-expanded="false" aria-controls="sidebar">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><use href='{root}feather-sprite.svg#menu'/></svg>
            </button>
            <nav class="breadcrumbs">{breadcrumbs}</nav>
            <div id="dark-mode-placeholder"></div>
            <button class="toc-toggle" id="toc-toggle" aria-label="Toggle table of contents" aria-expanded="false" aria-controls="toc-panel">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><use href='{root}feather-sprite.svg#list'/></svg>
            </button>
        </div>
    </header>
    <section class="content doc-grid">
        <aside class="sidebar" id="sidebar">
            <button class="drawer-close" aria-label="Close navigation">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><use href='{root}feather-sprite.svg#x'/></svg>
            </button>
            <div id="docket-search"></div>
            <nav class="site-nav">
                <h2>In this section</h2>
                {navs}
            </nav>
        </aside>
        <nav class="toc-tree" id="toc-panel">
            <button class="drawer-close" aria-label="Close table of contents">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><use href='{root}feather-sprite.svg#x'/></svg>
            </button>
            <h2>On this Page</h2>
            {toc}
        </nav>
        <main>
            <article id="document-content">
                {content}
            </article>
        </main>
    </section>
    <footer><div class="content">{footer}</div></footer>
</body>
</html>"##,
            site_name = state.ctx().site_name,
            root = root,
            breadcrumbs = Breadcrumbs(state, nav_prefix),
            page_title = page.title(),
            navs = Navs(&state.navs, nav_prefix, &root),
            toc = RenderedToc(page.content(), HeadingLevel::H3),
            content = Content(page.content(), &root),
            footer = get_footer(state)
        )?;
        Ok(())
    }

    fn assets(&self) -> &[Asset] {
        static ASSETS: [Asset; 6] = [
            Asset::internal("style.css", include_str!("../../../assets/style.css")),
            Asset::internal("search.js", include_str!("../../../assets/search.js")),
            Asset::internal("dark.js", include_str!("../../../assets/dark.js")),
            Asset::internal("nav.js", include_str!("../../../assets/nav.js")),
            Asset::internal(
                "permalink.js",
                include_str!("../../../assets/permalink.js"),
            ),
            Asset::internal(
                "feather-sprite.svg",
                include_str!("../../../assets/feather-sprite.svg"),
            ),
        ];
        &ASSETS[..]
    }
}
