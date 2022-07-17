use log::trace;

use crate::{
    doctree::{self, BaleFrontispice, DoctreeItem},
    error::Result, asset::Asset,
};
use std::{
    path::{Path, PathBuf}, fs::{self, File}, io::{Write, BufWriter},
};

/// Render Contex
///
/// A render context represents a point into which pages can be rendered. It
/// stores global information about the current rendering process. The render
/// context is immutable for the duration of the render.
/// 
/// Nested information about the current output directory, the bale being
/// rendred, and the current point in the navigation heirachy is stored in the
/// `RenderState`.
pub struct RenderContext {
    /// The output directory to write the state to
    path: PathBuf,
    /// The overall site name. This is used as the root point in the navigation.
    site_name: String,
}

impl RenderContext {
    /// Create a new Root Render Context
    ///
    /// Root render contexts hold global information about the render, and are
    /// used as parents for derived cotnexts.
    pub fn new(path: PathBuf, site_name: String) -> Self {
        RenderContext {
            path,
            site_name
        }
    }
}

enum RenderStateKind<'s, 'b> {
    /// A root render state. This state has a direct reference to the render
    /// context.
    Root(&'s RenderContext),
    /// A nested render state. This is made up of a reference to the parent
    /// render state, along with the new directory root this context should
    /// write into.
    Nested(&'s RenderState<'s, 'b>, PathBuf),
}

/// Render State
/// 
/// The render state is used by layout to render pages.
pub struct RenderState<'s, 'b> {
    /// The kind link
    kind: RenderStateKind<'s, 'b>,
    /// The bale that is being rendered
    bale: &'b BaleFrontispice,
}

impl<'s, 'b> RenderState<'s, 'b> {
    /// Create a new root render state
    /// 
    /// This render state represents the root node in the documentaiton tree. It
    /// renders to the path in the given render context directly.
    fn new_root(context: &'s RenderContext, bale: &'b BaleFrontispice) -> Self {
        RenderState { kind: RenderStateKind::Root(context), bale }
    }

    /// Create a new render context based on a parent
    fn with_parent(state: &'s RenderState<'s, 'b>, bale: &'b BaleFrontispice) -> Self {
        let mut new_path = PathBuf::from(state.output_path());
        new_path.push(&bale.slug());
        RenderState { kind: RenderStateKind::Nested(state, new_path), bale }
    }

    /// Get the output path for this render state
    /// 
    /// The path is the folder where items should be created when rendering.
    fn output_path(&self) -> &Path {
        match &self.kind {
            RenderStateKind::Root(ctx) => &ctx.path,
            RenderStateKind::Nested(_, path) => path,
        }
    }

    /// The current bale
    fn current_bale(&self) -> &BaleFrontispice {
        &self.bale
    }

    /// Get the render context
    fn ctx(&self) -> &RenderContext {
        match self.kind {
            RenderStateKind::Root(ctx) => ctx,
            RenderStateKind::Nested(parent, _) => parent.ctx(),
        }
    }
}

/// Kind of page we are rendering. For index pages we don't need to do anything
/// to get to the bale root. For nested pages we keep the page's slug.
enum PageKind {
    /// An index page
    Index,
    /// A neseted page with a given slug
    Nested(String)
}

/// Render a bale to the given directory
///
/// This walks the tree of documentaiton referred to by the given `bale` and
/// writes the rendered result to the given `ctx`.
fn render_bale_contents(state: &RenderState, assets: Vec<PathBuf>, items: Vec<DoctreeItem>) -> Result<()> {
    trace!("rendering bale contents {:?} to {:?}", state.bale, state.output_path());
    fs::create_dir_all(&state.output_path())?;

    // TODO: Navs should probably be moved into the render state, rather than
    // being fabricated here in the render function.
    let navs: Vec<_> = items
        .iter()
        .map(|item| match item {
            DoctreeItem::Page(page) => (page.slug().to_owned(), page.title().to_owned()),
            DoctreeItem::Bale(bale) => (bale.frontispiece().slug().to_owned(), bale.frontispiece().title().to_owned()),
        })
        .collect();

    // If we have an index page then redner that
    if let Some(page) = state.current_bale().index_page() {
        trace!("Bale has an index. Rendering.");
        render_page(&state, &navs,PageKind::Index,  page)?;
    }

    // Walk our assets and copy them
    for asset in assets {
        Asset::path(asset).copy_to(&state.output_path())?;
    }

    // Walk the inner items in the bale and render them, in nested contexts if
    // required.
    for item in items {
        match item {
            DoctreeItem::Bale(bale) => {
                let (bale, assets, items) = bale.break_open()?;
                let state = RenderState::with_parent(&state, &bale);
                render_bale_contents(&state, assets, items)?;
            }
            DoctreeItem::Page(page) =>
                render_page(&state, &navs, PageKind::Nested(page.slug().to_owned()), &page)?
        }
    }

    Ok(())
}

/// Render a Single Page
///
/// Writes the rendred contents of a given page to a given path.
fn render_page(
    state: &RenderState,
    // FIXME: Navs need mving into the render state
    navs: &[(String, String)],
    kind: PageKind,
    page: &crate::doctree::Page,
) -> Result<()> {

    let (path, nav_prefix) = match kind {
        PageKind::Index => (state.output_path().into(), "./"),
        PageKind::Nested(slug) => {
            let mut path = PathBuf::from(state.output_path());
            path.push(slug);
            (path, "../")
        },
    };

    trace!("rendering page {} at {:?}", page.title(), path);
    fs::create_dir_all(&path)?;

    let output_path = path.join("index.html");
    let file = File::create(&output_path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "{} -- {}", state.ctx().site_name, page.title())?;
    writeln!(writer, "-----------------------------------")?;

    writeln!(writer, " * # [{}]({})", state.current_bale().title(), nav_prefix)?;
    for nav in navs {
        writeln!(writer, "   - ## [{}]({}{})", nav.1, nav_prefix, nav.0)?;
    }

    write!(writer, "\n***\n\n{}", page.content())?;

    Ok(())
}

/// Render a Doctree
///
/// Wwrite  the given doctree out to the `target` path using the default render
/// contex.
pub(crate) fn render<P: AsRef<Path>>(target: P, title: String, doctree_root: doctree::Bale) -> Result<()> {
    let ctx = RenderContext::new(target.as_ref().to_owned(), title);
    let (frontispiece, assets, items) = doctree_root.break_open()?;
    let state = RenderState::new_root(&ctx, &frontispiece);
    render_bale_contents(&state, assets, items)
}
