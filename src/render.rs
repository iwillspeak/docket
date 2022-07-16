use log::trace;

use crate::{
    baler::{self, BaleItem, NavInfo},
    error::Result,
};
use std::{
    fs::{create_dir, create_dir_all, File},
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
}

impl RenderContext {
    /// Create a new Root Render Context
    ///
    /// Root render contexts hold global information about the render, and are
    /// used as parents for derived cotnexts.
    pub fn create_root(path: PathBuf, title: String) -> Self {
        RenderContext { path, title }
    }
}

pub(crate) fn render_bale(ctx: &RenderContext, bale: baler::UnopenedBale) -> Result<()> {
    trace!("rendering bale {:?} to {:?}", bale, ctx.path);
    create_dir_all(&ctx.path)?;

    let bale = bale.open()?;
    let navs: Vec<_> = bale
        .items()
        .map(|item| match &item {
            BaleItem::Page(page) => NavInfo {
                // FIXME: Slug path for pages
                slug_path: Vec::new(),
                title: page.title().to_owned(),
            },
            BaleItem::Bale(bale) => bale.nav_info().clone(),
        })
        .collect();

    if let Some(page) = bale.index() {
        render_page(&ctx, &ctx.path, &navs, page)?;
    }

    for item in bale.into_items() {
        match item {
            BaleItem::Bale(bale) => {
                // FIXME: need to create a child context here, appending the
                // bale's slug and generating the correct path.
                let path = (&ctx.path).clone();
                let path = path.join(bale.slug());
                let ctx = RenderContext::create_root(path, ctx.title.clone());
                render_bale(&ctx, bale)?;
            }
            BaleItem::Page(page) => {
                let path = (&ctx.path).clone();
                let path = path.join(page.slug());
                render_page(&ctx, &path, &navs, &page)?;
            }
        }
    }

    Ok(())
}

fn render_page(
    _ctx: &RenderContext,
    path: &PathBuf,
    navs: &[NavInfo],
    page: &crate::page::Page,
) -> Result<()> {
    trace!("rendering page {} at {:?}", page.title(), path);
    create_dir_all(&path)?;

    let output_path = path.join("index.html");
    let file = File::create(&output_path)?;
    let mut writer = BufWriter::new(file);

    for nav in navs {
        writeln!(writer, " * [{}]({:?})", nav.title, nav.slug_path)?;
    }

    write!(writer, "\n***\n\n{}", page.content())?;

    Ok(())
}
