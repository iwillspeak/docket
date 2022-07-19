//! Markdown to HTML Documentation Generator

#![deny(missing_docs)]

mod args;
mod asset;
mod docket;
mod doctree;
mod error;
mod highlight;
mod render;
mod search;
mod utils;

use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use docket::Docket;
use error::Result;
use log::{info, warn};

use crate::error::ResultExt;

#[cfg(all(feature = "syntect-hl"))]
#[macro_use]
extern crate lazy_static;

/// On Error Behaviour
///
/// Chooses what should happen if an error happens when running the build.
#[derive(PartialEq, Copy, Clone)]
#[cfg_attr(not(feature = "watch"), allow(dead_code))]
enum OnError {
    Ignore,
    Exit,
}

fn main() {
    init_logging();

    let args = args::from_command_line();
    let source = utils::path_or_default(args.flag_source, ".");
    let target = utils::path_or_default(args.flag_target, "build/");

    handle_err(
        if args.flag_watch {
            watch_and_build(&target, &source)
        } else {
            build(&source, &target)
        },
        OnError::Exit,
    )
}

/// Watch and Rebuild
///
/// This opens a file watcher listening for changes in the source directory.
/// When a file is changed we re-build the documentaiton tree.
fn watch_and_build(target: &PathBuf, source: &PathBuf) -> Result<()> {
    #[cfg(feature = "watch")]
    {
        use notify::{watcher, RecursiveMode, Watcher};
        use std::sync::mpsc::channel;
        use std::time::Duration;

        // Create the target directory first, to ensure we can unwatch it if
        // needed.
        //
        // TODO: properly handle the error here.
        fs::create_dir_all(target).annotate_err("Error creating target directory")?;

        let (tx, rx) = channel();

        let mut watcher =
            watcher(tx, Duration::from_secs(2)).annotate_err("could not create file watcher")?;

        watcher
            .watch(source, RecursiveMode::Recursive)
            .annotate_err("Error watching source directory")?;
        // Ignore the erorr here. It most likely means the source and target dir
        // don't overlap.
        let _ = watcher.unwatch(target);

        handle_err(build(source, target), OnError::Ignore);
        println!("Build complete. Watching for changes.");
        loop {
            match rx.recv() {
                Ok(s) => match s {
                    // Ignore these. They're not debounced.
                    notify::DebouncedEvent::NoticeWrite(_)
                    | notify::DebouncedEvent::NoticeRemove(_) => (),
                    // Write something out if we notice erorrs.
                    notify::DebouncedEvent::Error(e, path) => {
                        warn!("Watcher error at path {path:?}: {e}")
                    }
                    // Anything else means a rebuild
                    _ => {
                        println!("Rebuilding...");
                        handle_err(build(source, target), OnError::Ignore);
                        println!("Rebuild complete. Watching for changes.");
                    }
                },
                // println!("Rebuilding... {s:?}")
                Err(e) => eprintln!("Watcher error: {}", e),
            }
        }
    }
    #[cfg(not(feature = "watch"))]
    {
        eprintln!("Watch not supported. Performing on off build.");
        build(source, target)
    }
}

/// Display the contents of the error, if any, and maybe exit
fn handle_err(err: Result<()>, on_err: OnError) {
    if let Err(err) = err {
        eprintln!("docket: error: {}", err);
        if let Some(source) = err.source() {
            warn!("Error caused by {}", source);
        }
        if on_err == OnError::Exit {
            std::process::exit(-1);
        }
    }
}

/// Run a single pass of documentation generation
///
/// This does the main job of rendering the documentaiton. Seprated into a
/// different function so we can use the `?` operator, and repeatedly call if
/// we're watching files and re-rendering on change.
fn build(source: &Path, target: &Path) -> Result<()> {
    info!("Rendering documenation from {:?} => {:?}", &source, &target);
    Docket::open(source)?.render(target)
}

/// Initialise logging
///
/// Sets up the env logger using our custom environment varaibles.
fn init_logging() {
    use env_logger::*;
    builder()
        .target(Target::Stdout)
        .parse_env(
            Env::new()
                .filter("DOCKET_LOG")
                .write_style("DOCKET_LOG_STYLE"),
        )
        .init();
}
