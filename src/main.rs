//! Markdown to HTML Documentation Generator

#![deny(missing_docs)]

mod args;
mod asset;
mod baler;
mod docket;
mod error;
mod legacy;
mod page;
mod render;
mod search;
mod utils;

use std::{error::Error, path::PathBuf, process::exit};

use docket::Docket;
use env_logger::Env;
use error::Result;
use log::{info, warn};

#[cfg(feature = "syntect-hl")]
#[macro_use]
extern crate lazy_static;

/// Run a single pass of documentation generation
fn run(source: PathBuf, target: PathBuf) -> Result<()> {
    info!("Rendering documenation from {:?} => {:?}", &source, &target);
    Docket::open(source)?.render(target)?;
    Ok(())
}

fn main() {
    init_logging();

    let args = args::from_command_line();
    let source = utils::path_or_default(args.flag_source, ".");
    let target = utils::path_or_default(args.flag_target, "build/");

    if let Err(err) = run(source, target) {
        eprintln!("docket: error: {}", err);
        if let Some(source) = err.source() {
            warn!("Error caused by {}", source);
        }
        exit(-1);
    }
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
