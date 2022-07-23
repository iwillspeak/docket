//! Layout
//!
//! This module contains the traits needed to implement a layout. Layouts are
//! used to abstract over the exact HTML that is written.

use std::io::Write;

use crate::{doctree, error::Result};

use super::{PageKind, RenderState};

/// Layout Trait
///
/// Layouts are responsible for writing out the contents of pages to files. A
/// layout recieves a reference to the current render state, and information
/// about the current page.
pub(crate) trait Layout {
    /// Render a Page
    ///
    /// Layout rendering should write a representaiton of the `page` to the
    /// given `writer`. Context is provided in the `state` and `kind` make
    /// redenring of navigation information possible.
    fn render(
        &self,
        writer: &mut dyn Write,
        state: &RenderState,
        kind: PageKind,
        page: &doctree::Page,
    ) -> Result<()>;
}

struct DefaultLayout;

impl Layout for DefaultLayout {
    fn render(
        &self,
        writer: &mut dyn Write,
        state: &RenderState,
        kind: PageKind,
        page: &doctree::Page,
    ) -> Result<()> {
        let nav_prefix = kind.path_to_bale();

        writeln!(writer, "<!DOCTYPE html><html><body>")?;
        writeln!(
            writer,
            "<heading><h1>{}</h1></heading>",
            state.ctx().site_name
        )?;
        writeln!(writer, "<nav><ul>")?;
        writeln!(
            writer,
            "<li><a href='{1}'>{0}</a>",
            state.current_bale().title(),
            nav_prefix
        )?;
        writeln!(writer, "<ul>")?;
        for nav in state.navs.iter() {
            writeln!(
                writer,
                "<li><a href='{1}{2}'>{0}</a>",
                nav.title, nav_prefix, nav.slug
            )?;
        }
        writeln!(writer, "</ul>")?;
        writeln!(writer, "</ul></nav>")?;
        writeln!(writer, "<main>")?;
        write!(writer, "{}", page.content())?;
        writeln!(writer, "</main>")?;
        writeln!(writer, "<footer>")?;
        if let Some(footer) = state.current_bale().footer() {
            write!(writer, "{}", footer)?;
        } else {
            write!(writer, "Rendered by Docket")?;
        }
        Ok(())
    }
}

// Get the dfault layout
pub(crate) fn get_default_layout<'a>() -> &'a dyn Layout {
    const DEFAULT: DefaultLayout = DefaultLayout;
    &DEFAULT
}
