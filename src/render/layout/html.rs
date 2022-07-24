use std::{fmt, io::Write};

use crate::{
    doctree::Page,
    error::Result,
    render::{NavInfo, PageKind, RenderState},
};

use super::Layout;

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
            content = page.content(),
            footer = get_footer(state)
        )?;
        Ok(())
    }
}
