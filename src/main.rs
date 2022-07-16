//! Markdown to HTML Documentation Generator

#![deny(missing_docs)]

mod args;
mod baler;
mod docket;
mod error;
mod legacy;
mod page;
mod search;
mod utils;

use std::{path::PathBuf, process::exit};

use docket::Docket;
use error::Result;

#[cfg(feature = "syntect-hl")]
#[macro_use]
extern crate lazy_static;

/// Run a single pass of documentation generation
fn run(source: PathBuf, target: PathBuf) -> Result<()> {
    Docket::open(source)?.render(target)?;
    Ok(())
}

fn main() {
    env_logger::init();

    if std::env::var("DOCKET_LEGACY").is_ok() {
        legacy::main();
    } else {
        let args = args::from_command_line();
        let source = utils::path_or_default(args.flag_source, ".");
        let target = utils::path_or_default(args.flag_target, "build/");

        if let Err(err) = run(source, target) {
            eprint!("Error {}", err);
            exit(-1);
        }
    }
}
