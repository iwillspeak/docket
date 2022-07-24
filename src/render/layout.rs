//! Layout
//!
//! This module contains the traits needed to implement a layout. Layouts are
//! used to abstract over the exact HTML that is written.

mod html;

use super::{PageKind, RenderState};
use crate::{doctree, error::Result};
use html::HtmlLayout;
use std::io::Write;

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

// Get the dfault layout
pub(crate) fn get_default_layout<'a>() -> &'a dyn Layout {
    const DEFAULT: HtmlLayout = HtmlLayout;
    &DEFAULT
}
