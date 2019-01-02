//! Markdown to HTML Documentation Generator


use env_logger;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;


mod asset;
mod docket;
mod page;
mod util;
mod toc;
mod index;
mod renderable;
mod renderer;


use crate::docket::Docket;
use docopt::*;
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::path::{Path, PathBuf};
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
  -w, --watch         Watch for changes and re-generate.
  -s, --source=<in>   Documentation directory, default is current directory.
  -t, --target=<out>  Write the output to <out>, default is `./build/`.
";

/// Program Arguments
///
/// Structure to capture the command line arguments for the
/// program. This is filled in for us by Docopt.
#[derive(Debug, Deserialize)]
struct Args {
    flag_watch: bool,
    flag_source: Option<String>,
    flag_target: Option<String>,
}

/// On Erorr Behaviour
///
/// Chooses what should happen if an error happens when running the build.
#[derive(PartialEq)]
enum OnError { Skip, Exit }

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
        .map(|d| d.version(Some(format!("Docket {}", env!("CARGO_PKG_VERSION")))))
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let source = path_or_default(args.flag_source, ".");
    let target = path_or_default(args.flag_target, "build/");

    if args.flag_watch {
        let (tx, rx) = channel();

        let mut watcher = watcher(tx, Duration::from_secs(2))
            .expect("could not create file watcher");

        watcher
            .watch(&source, RecursiveMode::NonRecursive)
            .unwrap();

        loop {
            run(&source, &target, OnError::Skip);
            println!("Watching for changes.");
            match rx.recv() {
                Ok(_) => println!("Rebuilding..."),
                Err(e) => eprintln!("Watcher error: {}", e)
            }
        }
    } else {
        run(&source, &target, OnError::Exit);
    }
}

/// Run Method
///
/// The main work of rendering the documentation. Separated into a
/// different function so we can use the `?` operator.
fn run(source: &Path, target: &Path, on_err: OnError) {
    if let Err(e) = build(source, target) {
        eprintln!("Error: {}", e);
        if on_err == OnError::Exit {
            std::process::exit(-1);
        }
    }
}

/// Do the Build
fn build(source: &Path, target: &Path) -> Result<(), Error> {
    Docket::new(source)?.render(target)
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
