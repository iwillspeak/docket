//! Markdown to HTML Documentation Generator

#![deny(missing_docs)]

mod baler;
mod legacy;

use std::path::Path;

#[cfg(feature = "syntect-hl")]
#[macro_use]
extern crate lazy_static;

fn main() {
    if std::env::var("DOCKET_LEGACY").is_ok() {
        legacy::main();
    } else {
        for dir in std::env::args().skip(1) {
            match baler::open_bale(&Path::new(&dir)) {
                Ok(bale) => {
                    println!("got bale {0:?}", bale);
                }
                Err(err) => {
                    eprintln!("Error opening bale: {0}", err);
                }
            };
        }
    }
}
