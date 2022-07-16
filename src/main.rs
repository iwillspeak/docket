//! Markdown to HTML Documentation Generator

#![deny(missing_docs)]

mod args;
mod baler;
mod legacy;
mod page;
mod utils;

use std::{io, path::Path};

use baler::UnopenedBale;

use crate::baler::BaleItem;

#[cfg(feature = "syntect-hl")]
#[macro_use]
extern crate lazy_static;

fn walk_bale(bale: UnopenedBale) -> Result<(), io::Error> {
    println!("got bale with index {:?}", bale.index());
    let opend = bale.open()?;
    for item in opend.into_items() {
        match item {
            BaleItem::Page(page) => println!("Page {:?}", page),
            BaleItem::Bale(bale) => walk_bale(bale)?,
        }
    }
    Ok(())
}

fn main() {
    env_logger::init();

    if std::env::var("DOCKET_LEGACY").is_ok() {
        legacy::main();
    } else {
        let args = args::from_command_line();
        let source = utils::path_or_default(args.flag_source, ".");
        let _target = utils::path_or_default(args.flag_target, "build/");

        match baler::bale_dir(source) {
            Ok(bale) => {
                walk_bale(bale).unwrap();
            }
            Err(err) => {
                eprintln!("Error opening bale: {0}", err);
            }
        };
    }
}
