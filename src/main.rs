//! Markdown to HTML Documentation Generator

extern crate docopt;
#[macro_use]
extern crate serde_derive;
extern crate pulldown_cmark;

// mod docket;
// mod page;

use docopt::*;

/// Usage Information
///
/// This is a [Docopt] compliant usage description of this program.
///
///  [Docopt]: http://docopt.org/
const USAGE: &'static str = "
Docket Documentation Generator

Usage: docket [options]

Options:
  -h --help           Show this screen.
  -s, --source=<in>   Documentation directory, default is current directory.
  -t, --target=<out>  Write the output to <out>, default is `./build/`.
";

/// Program Arguments
///
/// Structure to capture the command line arguments for the
/// program. This is filled in for us by Docopt.
#[derive(Debug, Deserialize)]
struct Args {
    flag_source: Option<String>,
    flag_target: Option<String>,
}

fn main() {

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    println!("Args: {:?}", args);
}
