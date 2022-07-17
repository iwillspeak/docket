use log::trace;

use crate::{
    baler::{self, BaleItem, NavInfo},
    error::Result,
};
use std::{
    fs::{create_dir_all, File},
    io::{BufWriter, Write},
    path::PathBuf,
};

/// Render Contex
///
/// A render context represents a point into which pages can be rendered. It
/// stores global information about the current rendering process, such as the
/// overall documentation site title, as well as context specific information.
///
/// Each document bale is rendered in a single render context. New contexts are
/// created for derived bales.
pub struct RenderContext {
    path: PathBuf,
    title: String,
    slug_path: Vec<String>,
}

impl RenderContext {
    /// Create a new Root Render Context
    ///
    /// Root render contexts hold global information about the render, and are
    /// used as parents for derived cotnexts.
    pub fn create_root(path: PathBuf, title: String) -> Self {
        RenderContext {
            path,
            title,
            slug_path: Vec::new(),
        }
    }

    /// Create a render context based on a parent one
    ///
    /// TODO: This currently just copies information from the parent context.
    /// Instead it might be nice to form a proper tree, allowing contexts to
    /// return references to their parent contexts. This would hopefully move
    /// some of the infromation about navigation into the render context rather
    /// than having it 'tag along' as it currently does.
    ///
    /// Anotehr advantage mgith be that asset paths could be reified in the
    /// render context too, allowing assets to be queried for during rendering.
    /// This would allow us to 'fixup' asset paths that we have re-written the
    /// slugs for.
    pub fn create_with_parent(parent: &Self, path: PathBuf, slug: String) -> Self {
        let mut slug_path = parent.slug_path.clone();
        slug_path.push(slug);
        RenderContext {
            path,
            title: parent.title.to_owned(),
            slug_path,
        }
    }
}

/// Render a bale to the given directory
///
/// This walks the tree of documentaiton referred to by the given `bale` and
/// writes the rendered result to the given `ctx`.
///
/// TODO: Should this just take a bale, and an output path. We cna then
/// fabricate a context _as required_?
pub(crate) fn render_bale(ctx: &RenderContext, bale: baler::UnopenedBale) -> Result<()> {
    trace!("rendering bale {:?} to {:?}", bale, ctx.path);
    create_dir_all(&ctx.path)?;

    // Break open the bale and build navigiation information for it
    let bale = bale.break_open()?;
    let navs: Vec<_> = bale
        .items()
        .map(|item| match &item {
            BaleItem::Page(page) => NavInfo {
                slug: page.slug().to_owned(),
                title: page.title().to_owned(),
            },
            BaleItem::Bale(bale) => bale.nav_info().clone(),
        })
        .collect();

    // If we have an index page then redner that
    if let Some(page) = bale.index() {
        render_page(&ctx, &ctx.path, "", &navs, page)?;
    }

    // Walk our assets and copy them
    for asset in bale.assets() {
        asset.copy_to(&ctx.path)?;
    }

    // Walk the inner items in the bale and render them, in nested contexts if
    // required.
    for item in bale.into_items() {
        match item {
            BaleItem::Bale(bale) => {
                // FIXME: need to create a child context here, appending the
                // bale's slug and generating the correct path.
                let path = (&ctx.path).clone();
                let path = path.join(&bale.nav_info().slug);
                let ctx =
                    RenderContext::create_with_parent(ctx, path, bale.nav_info().slug.clone());
                render_bale(&ctx, bale)?;
            }
            BaleItem::Page(page) => {
                let path = (&ctx.path).clone();
                let path = path.join(page.slug());
                render_page(&ctx, &path, "../", &navs, &page)?;
            }
        }
    }

    Ok(())
}

/// Render a Single Page
///
/// Writes the rendred contents of a given page to a given path.
///
/// TOOD: This method has too many paramters. We should move as much of this as
/// possible into either the render contex, or some page specific item.
///
/// One of the akward things we're dealing with here, with the `nav_prefix`, is
/// the page's locaiton within the bale. We could abstact over this with some
/// structure that is either `Index`, or `Nested(NavItem)`. That would allow a
/// page to know both the correct nav prefix to access other items in the bale
/// (and therefore outer bales) as well as identify itself in the navigation
/// tree. This might be useful for setting a `class=current` on the navigation
/// items.
fn render_page(
    _ctx: &RenderContext,
    path: &PathBuf,
    nav_prefix: &str,
    navs: &[NavInfo],
    page: &crate::page::Page,
) -> Result<()> {
    trace!("rendering page {} at {:?}", page.title(), path);
    create_dir_all(&path)?;

    let output_path = path.join("index.html");
    let file = File::create(&output_path)?;
    let mut writer = BufWriter::new(file);

    for nav in navs {
        writeln!(writer, " * [{}]({}{})", nav.title, nav_prefix, nav.slug)?;
    }

    write!(writer, "\n***\n\n{}", page.content())?;

    Ok(())
}
