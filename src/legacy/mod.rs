mod docket;
mod highlight;
mod index;
mod page;
mod renderable;
mod renderer;
mod toc;

use crate::utils::path_or_default;
use docket::Docket;
use std::path::Path;

pub type DynError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, DynError>;

/// On Error Behaviour
///
/// Chooses what should happen if an error happens when running the build.
#[derive(PartialEq, Copy, Clone)]
#[cfg_attr(not(feature = "watch"), allow(dead_code))]
enum OnError {
    Skip,
    Exit,
}

pub(crate) fn main() {
    let args = crate::args::from_command_line();

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
fn build(source: &Path, target: &Path) -> Result<()> {
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
