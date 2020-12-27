//! Markdown to HTML Documentation Generator

mod asset;
mod docket;
mod highlight;
mod index;
mod page;
mod renderable;
mod renderer;
mod toc;
mod util;

#[macro_use]
extern crate lazy_static;
use crate::docket::Docket;
use docopt::*;
use env_logger;
use failure::Error;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Usage Information
///
/// This is a [Docopt] compliant usage description of this program.
///
///  [Docopt]: http://docopt.org/
const USAGE: &str = "
Docket Documentation Generator

Usage: docket [options]

Options:
  --version           Show the version.
  -h --help           Show this screen.
  -s, --source=<in>   Documentation directory, default is current directory.
  -t, --target=<out>  Write the output to <out>, default is `./build/`.
  -w, --watch         Watch for changes and re-generate.
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

/// On Error Behaviour
///
/// Chooses what should happen if an error happens when running the build.
#[derive(PartialEq, Copy, Clone)]
#[cfg_attr(not(feature = "watch"), allow(dead_code))]
enum OnError {
    Skip,
    Exit,
}

/// Path or Default
///
/// Try to convert a command line argument to a path. Falling back to
/// the default if none is provided.
fn path_or_default(maybe_path: Option<String>, default: &str) -> PathBuf {
    maybe_path
        .map(PathBuf::from)
        .unwrap_or_else(|| default.to_owned().into())
}

fn main() {
    env_logger::init();

    let args: Args = Docopt::new(USAGE)
        .map(|d| d.version(Some(format!("Docket {}", env!("CARGO_PKG_VERSION")))))
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let source = path_or_default(args.flag_source, ".");
    let target = path_or_default(args.flag_target, "build/");

    if args.flag_watch {
        #[cfg(feature = "watch")]
        {
            use notify::{watcher, RecursiveMode, Watcher};
            use std::sync::mpsc::channel;
            use std::time::Duration;

            let (tx, rx) = channel();

            let mut watcher =
                watcher(tx, Duration::from_secs(2)).expect("could not create file watcher");

            watcher.watch(&source, RecursiveMode::NonRecursive).unwrap();

            loop {
                run(&source, &target, OnError::Skip);
                println!("Watching for changes.");
                match rx.recv() {
                    Ok(_) => println!("Rebuilding..."),
                    Err(e) => eprintln!("Watcher error: {}", e),
                }
            }
        }
        #[cfg(not(feature = "watch"))]
        eprintln!("Watch not supported.");
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

    use super::*;
    use std::path::Path;

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
