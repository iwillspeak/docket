use crate::{
    asset::Asset,
    doctree::Page,
    error::Result,
    highlight,
    render::{CardSummary, NavInfo, PageKind, RenderState},
    toc::{Nodes, Toc, TocElement, TocNode},
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

struct InlineBreadcrumbs<'a>(&'a RenderState<'a, 'a>, &'a str, &'a str);

impl<'a> fmt::Display for InlineBreadcrumbs<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current = Some(self.0);
        let mut stack = Vec::new();
        let mut path = String::from(self.1);
        let root = self.2;

        while let Some(state) = current {
            stack.push((path.clone(), state));
            path.push_str("../");
            current = state.parent();
        }
        write!(f, "<ol class='breadcrumbs'>")?;
        if let Some((path, _)) = stack.pop() {
            write!(
                f,
                "<li class='primary'><a href='{}'>\
                    <svg class='bc-home-icon' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' aria-hidden='true'><use href='{}feather-sprite.svg#home'/></svg>\
                    <span class='bc-site-name'>{}</span>\
                </a></li>",
                path,
                root,
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
                TocElement::TocReference => {
                    write!(f, "<nav class='inline-toc'><h2 class='toc-section-label'>In this section</h2>")?;
                    render_toc_to(f, self.0.nodes(), HeadingLevel::H2)?;
                    write!(f, "</nav>")?;
                }
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
        render_toc_to(f, self.0.nodes(), self.1)
    }
}

/// Renders the card grid for a bale's children in the article body.
struct Cards<'a>(&'a [CardSummary]);

impl<'a> fmt::Display for Cards<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }
        write!(f, "<div class='index-cards'>")?;
        for card in self.0 {
            write!(
                f,
                "<div class='index-card'>\
                    <h3 class='index-card-title'>\
                        <a class='index-card-link' href='{href}'>{title}</a>\
                    </h3>\
                    <div class='index-card-blurb'>{blurb}</div>\
                </div>",
                href = card.href,
                title = card.title,
                blurb = card.blurb,
            )?;
        }
        write!(f, "</div>")
    }
}

/// Renders a compact link list of child pages below the sidebar TOC.
struct CardLinks<'a>(&'a [CardSummary]);

impl<'a> fmt::Display for CardLinks<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }
        write!(f, "<nav class='child-links'><h2>Pages</h2><ul class='toc'>")? ;
        for card in self.0 {
            write!(
                f,
                "<li><a href='{href}'>{title}</a></li>",
                href = card.href,
                title = card.title,
            )?;
        }
        write!(f, "</ul></nav>")
    }
}

/// Render the TOC
///
/// This walks the list of `TocElement`s and renders them as a list of links
/// using their clean text for the link body.
///
/// When there is exactly one root node (the page's H1), its children (H2+) are
/// rendered directly so the page title is not repeated in the TOC.
fn render_toc_to(f: &mut fmt::Formatter<'_>, nodes: Nodes, limit: HeadingLevel) -> fmt::Result {
    if nodes.len() == 1 {
        let inner_nodes = nodes.into_iter().next().unwrap().nodes();
        if !inner_nodes.len() > 0 {
            return render_toc_node_to(f, inner_nodes, limit);
        }

        return Ok(());
    }

    render_toc_node_to(f, nodes.into_iter(), limit)
}

/// Render a Single TOC Node and its children
///
/// This renders a single TOC node as a link, and then recurses to render any
/// children it may have as a nested list.
fn render_toc_node_to<'a>(
    f: &mut fmt::Formatter<'_>,
    nodes: impl Iterator<Item = &'a TocNode>,
    limit: HeadingLevel,
) -> fmt::Result {
    write!(f, "<ul class='toc'>")?;
    for node in nodes {
        if node.heading.level <= limit {
            write!(
                f,
                "<li><a href='#{0}'>{1}</a>",
                node.heading.slug, node.heading.contents
            )?;

            render_toc_node_to(f, node.nodes(), limit)?;
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

/// Convert a `SystemTime` to a `YYYY-MM-DD` date string.
///
/// Uses Howard Hinnant's `civil_from_days` algorithm to avoid any external
/// date/time dependency.
fn format_date(t: std::time::SystemTime) -> String {
    use std::time::UNIX_EPOCH;
    let secs = match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(_) => return String::from("unknown"),
    };
    let z = secs / 86400 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Renders the last-updated mini-footer inside the article, or nothing.
struct LastUpdated(Option<std::time::SystemTime>);

impl fmt::Display for LastUpdated {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(t) = self.0 {
            write!(
                f,
                "<p class='page-updated'>Last updated: {}</p>",
                format_date(t)
            )?;
        }
        Ok(())
    }
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
        let cards = match &kind {
            PageKind::Index(summaries) => summaries.as_slice(),
            PageKind::Nested(_) => &[],
        };
        let hl_header = {
            let mut buf = Vec::new();
            highlight::get_hilighter().write_header(&mut buf)?;
            match String::from_utf8(buf) {
                Ok(s) => s,
                Err(e) => String::from_utf8_lossy(&e.into_bytes()).into_owned(),
            }
        };
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
    {hl_header}
    <script src="{root}dark.js" type=module></script>
    <script src="{root}search.js" type=module></script>
    <script src="{root}nav.js" type=module></script>
    <script src="{root}permalink.js" type=module></script>
</head>
<body>
    <header class="site-head">
        <div class="content">
            <button class="nav-toggle header-icon-btn" id="nav-toggle" aria-label="Toggle navigation" aria-expanded="false" aria-controls="sidebar">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><use href='{root}feather-sprite.svg#menu'/></svg>
            </button>
            <nav class="breadcrumbs">{breadcrumbs}</nav>
            <div id="dark-mode-placeholder"></div>
            <button class="toc-toggle header-icon-btn" id="toc-toggle" aria-label="Toggle table of contents" aria-expanded="false" aria-controls="toc-panel">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><use href='{root}feather-sprite.svg#more-vertical'/></svg>
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
            {card_links}
        </nav>
        <main>
            <nav class="breadcrumbs breadcrumbs-inline">{inline_breadcrumbs}</nav>
            <article id="document-content">
                {content}
                {child_cards}
                {last_updated}
            </article>
        </main>
    </section>
    <footer><div class="content">{footer}</div></footer>
</body>
</html>"##,
            site_name = state.ctx().site_name,
            root = root,
            hl_header = hl_header,
            breadcrumbs = Breadcrumbs(state, nav_prefix),
            inline_breadcrumbs = InlineBreadcrumbs(state, nav_prefix, &root),
            page_title = page.title(),
            navs = Navs(&state.navs, nav_prefix, &root),
            toc = RenderedToc(page.content(), HeadingLevel::H4),
            card_links = CardLinks(cards),
            content = Content(page.content(), &root),
            child_cards = Cards(cards),
            last_updated = LastUpdated(page.modified()),
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
