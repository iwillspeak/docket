//! Markdown to HTML Documentation Generator

#![deny(missing_docs)]

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
    if std::env::var("DOCKET_LEGACY").is_ok() {
        legacy::main();
    } else {
        for dir in std::env::args().skip(1) {
            match baler::bale_dir(&Path::new(&dir)) {
                Ok(bale) => {
                    walk_bale(bale).unwrap();
                }
                Err(err) => {
                    eprintln!("Error opening bale: {0}", err);
                }
            };
        }
    }
}
