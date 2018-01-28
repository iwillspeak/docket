//! Markdown to HTML Documentation Generator

extern crate docopt;
extern crate env_logger;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate pulldown_cmark;
#[macro_use]
extern crate serde_derive;

mod docket;
mod page;
mod util;
mod toc;
mod index;
mod renderable;
mod renderer;

use docopt::*;
use std::path::PathBuf;
use docket::Docket;
use failure::Error;

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
  --version           Show the version.
  -s, --source=<in>   Documentation directory, default is current directory.
  -t, --target=<out>  Write the output to <out>, default is `./build/`.
";

/// Program Arguments
///
/// Structure to capture the command line arguments for the
/// program. This is filled in for us by Docopt.
#[derive(Debug, Deserialize)]
struct Args {
    flag_version: bool,
    flag_source: Option<String>,
    flag_target: Option<String>,
}

/// Path or Default
///
/// Try to convert a command line argument to a path. Falling back to
/// the default if none is provided.
fn path_or_default(maybe_path: Option<String>, default: &str) -> PathBuf {
    maybe_path
        .map({ |p| PathBuf::from(p) })
        .unwrap_or(default.to_owned().into())
}

fn main() {
    let _ = env_logger::init();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("Docket version {}", env!("CARGO_PKG_VERSION"));
    }

    let source = path_or_default(args.flag_source, ".");
    let target = path_or_default(args.flag_target, "build/");

    if let Err(e) = run(source, target) {
        eprintln!("Error: {}", e);
        std::process::exit(-1);
    }
}

/// Run Method
///
/// The main work of rendering the documentation. Separated into a
/// different function so we can use the `?` operator.
fn run(source: PathBuf, target: PathBuf) -> Result<(), Error> {
    Docket::new(&source)?.render(&target)
}

#[cfg(test)]
mod test {

    use std::path::Path;
    use super::*;

    #[test]
    fn path_or_default_with_valid_argument() {
        let source = Some("/Users/foo/".to_owned());
        assert_eq!(Path::new("/Users/foo/"), path_or_default(source, "."));
    }

    #[test]
    fn path_or_default_without_argument() {
        let source = None;
        assert_eq!(Path::new("baz/"), path_or_default(source, "baz/"));
    }
}
