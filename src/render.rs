use log::trace;

mod layout;

use crate::{
    asset::Asset,
    doctree::{self, DoctreeItem, Frontispiece},
    error::Result,
};
use std::{
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
};

use self::layout::Layout;

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
    /// The layout for this render
    layout: Option<Box<dyn Layout>>,
}

impl RenderContext {
    /// Create a new Root Render Context
    ///
    /// Root render contexts hold global information about the render, and are
    /// used as parents for derived cotnexts.
    pub fn new(path: PathBuf, site_name: String) -> Self {
        RenderContext {
            path,
            site_name,
            layout: None,
        }
    }

    fn layout(&self) -> &dyn Layout {
        self.layout
            .as_deref()
            .unwrap_or_else(layout::get_default_layout)
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

impl<'s, 'b> RenderStateKind<'s, 'b> {
    /// Create a new root render state
    ///
    /// This render state represents the root node in the documentaiton tree. It
    /// renders to the path in the given render context directly.
    fn new_root(context: &'s RenderContext) -> Self {
        RenderStateKind::Root(context)
    }

    /// Create a new render context based on a parent state
    ///
    /// The nested render state gets a new path based on the paren'ts location
    /// and the given `slug`.
    fn with_parent(state: &'s RenderState<'s, 'b>, slug: &str) -> Self {
        let mut new_path = PathBuf::from(state.output_path());
        new_path.push(slug);
        RenderStateKind::Nested(state, new_path)
    }
}

/// Render State
///
/// The render state is used by layout to render pages.
pub struct RenderState<'s, 'b> {
    /// The kind link
    kind: RenderStateKind<'s, 'b>,
    /// The bale that is being rendered
    bale: &'b Frontispiece,
    /// The navigation items at this level
    navs: Vec<NavInfo>,
}

impl<'s, 'b> RenderState<'s, 'b> {
    /// Create a new render state
    ///
    /// This render state represents the root node in the documentaiton tree. It
    /// renders to the path in the given render context directly.
    fn new(kind: RenderStateKind<'s, 'b>, bale: &'b Frontispiece, navs: Vec<NavInfo>) -> Self {
        RenderState { kind, bale, navs }
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
    fn current_bale(&self) -> &Frontispiece {
        &self.bale
    }

    /// Get the render context
    fn ctx(&self) -> &RenderContext {
        match self.kind {
            RenderStateKind::Root(ctx) => ctx,
            RenderStateKind::Nested(parent, _) => parent.ctx(),
        }
    }

    /// Get this state's parent, if any
    fn parent(&self) -> Option<&RenderState> {
        if let RenderStateKind::Nested(parent, _) = &self.kind {
            Some(parent)
        } else {
            None
        }
    }
}

/// An entry in the navigation tree.
struct NavInfo {
    /// The title of the item.
    pub title: String,
    /// The slug to use when constructing a URI.
    pub slug: String,
}

impl NavInfo {
    fn new(slug: &str, title: &str) -> Self {
        NavInfo {
            title: title.to_owned(),
            slug: slug.to_owned(),
        }
    }
}

/// Kind of page we are rendering. For index pages we don't need to do anything
/// to get to the bale root. For nested pages we keep the page's slug.
pub(crate) enum PageKind {
    /// An index page
    Index,
    /// A neseted page with a given slug
    Nested(String),
}

impl PageKind {
    /// Get the path to the current bale given this page's kind.
    fn path_to_bale(&self) -> &'static str {
        match self {
            PageKind::Index => "./",
            PageKind::Nested(_) => "../",
        }
    }
}

/// Render a bale to the given directory
///
/// This walks the tree of documentaiton referred to by the given `bale` and
/// writes the rendered result to the given `ctx`.
fn render_bale_contents(
    state: &RenderState,
    assets: Vec<Asset>,
    items: Vec<DoctreeItem>,
) -> Result<()> {
    trace!(
        "rendering bale contents {:?} to {:?}",
        state.bale,
        state.output_path()
    );
    fs::create_dir_all(&state.output_path())?;

    // If we have an index page then redner that
    if let Some(page) = state.current_bale().index_page() {
        trace!("Bale has an index. Rendering.");
        render_page(&state, PageKind::Index, page)?;
    }

    // Walk our assets and copy them
    for asset in assets {
        asset.copy_to(&state.output_path())?;
    }

    // Walk the inner items in the bale and render them, in nested contexts if
    // required.
    for item in items {
        match item {
            DoctreeItem::Bale(bale) => {
                let (bale, assets, items) = bale.break_open()?;
                let navs = navs_for_items(&items);
                let state = RenderState::new(
                    RenderStateKind::with_parent(&state, bale.slug()),
                    &bale,
                    navs,
                );
                render_bale_contents(&state, assets, items)?;
            }
            DoctreeItem::Page(page) => {
                render_page(&state, PageKind::Nested(page.slug().to_owned()), &page)?
            }
        }
    }

    Ok(())
}

fn navs_for_items(items: &[DoctreeItem]) -> Vec<NavInfo> {
    items
        .iter()
        .map(|item| match item {
            DoctreeItem::Page(page) => NavInfo::new(page.slug(), page.title()),
            DoctreeItem::Bale(bale) => {
                NavInfo::new(bale.frontispiece().slug(), bale.frontispiece().title())
            }
        })
        .collect()
}

/// Render a Single Page
///
/// Writes the rendred contents of a given page to a given path.
fn render_page(state: &RenderState, kind: PageKind, page: &doctree::Page) -> Result<()> {
    let mut path = PathBuf::from(state.output_path());
    if let PageKind::Nested(slug) = &kind {
        path.push(slug);
        fs::create_dir_all(&path)?;
    };

    trace!("rendering page {} at {:?}", page.title(), path);

    let output_path = path.join("index.html");
    let file = File::create(&output_path)?;
    let mut writer = BufWriter::new(file);

    let layout = state.ctx().layout();
    layout.render(&mut writer, state, kind, page)?;

    Ok(())
}

/// Copy any assets used by the layout
fn copy_global_assets(ctx: &RenderContext) -> Result<()> {
    fs::create_dir_all(&ctx.path)?;
    for asset in ctx.layout().assets() {
        asset.copy_to(&ctx.path)?;
    }

    Ok(())
}

/// Render a Doctree
///
/// Wwrite  the given doctree out to the `target` path using the default render
/// contex.
pub(crate) fn render<P: AsRef<Path>>(
    target: P,
    title: String,
    doctree_root: doctree::Bale,
) -> Result<()> {
    let ctx = RenderContext::new(target.as_ref().to_owned(), title);
    let (frontispiece, assets, items) = doctree_root.break_open()?;
    let navs = navs_for_items(&items);
    let state = RenderState::new(RenderStateKind::new_root(&ctx), &frontispiece, navs);
    copy_global_assets(&ctx)?;
    render_bale_contents(&state, assets, items)
}
