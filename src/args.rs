//! Command Line Argument Parsing
//!
//! This module defines the command line usage text and provides a method to
//! load the arguments from the command line.

use docopt::Docopt;
use serde::Deserialize;

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
/// Structure to capture the command line arguments for the program. This is
/// filled in for us by Docopt.
#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    pub flag_watch: bool,
    pub flag_source: Option<String>,
    pub flag_target: Option<String>,
}

/// Parse the arguments from a given command line
pub(crate) fn from_argv<I: IntoIterator<Item = S>, S: AsRef<str>>(
    argv: I,
) -> Result<Args, docopt::Error> {
    Docopt::new(USAGE).and_then(|d| {
        d.argv(argv)
            .help(true)
            .version(Some(format!("Docket {}", env!("CARGO_PKG_VERSION"))))
            .deserialize()
    })
}

/// Get the command line arguments
pub(crate) fn from_command_line() -> Args {
    let argv = std::env::args();
    from_argv(argv).unwrap_or_else(|e| e.exit())
}

#[cfg(test)]
mod test {
    use super::from_argv;

    #[test]
    fn parse_empty_argv() {
        let args = from_argv::<&[String], &String>(&[]).unwrap();
        assert_eq!(None, args.flag_source);
        assert_eq!(None, args.flag_target);
        assert_eq!(false, args.flag_watch);

        let args = from_argv(&["docket"]).unwrap();
        assert_eq!(None, args.flag_source);
        assert_eq!(None, args.flag_target);
        assert_eq!(false, args.flag_watch);
    }

    #[test]
    fn parse_commands_argv() {
        let args = ["docket", "-s", "/some//.path", "-t", "../another", "-w"];
        let args = from_argv(args);
        let args = args.unwrap();
        assert_eq!(Some("/some//.path"), args.flag_source.as_deref());
        assert_eq!(Some("../another"), args.flag_target.as_deref());
        assert_eq!(true, args.flag_watch);
    }

    #[test]
    fn parse_long_form_options() {
        let args = [
            "docket",
            "--target=../another",
            "--watch",
            "--source",
            "/some/Path with Spaces/",
        ];
        let args = from_argv(args);
        let args = args.unwrap();
        assert_eq!(Some("/some/Path with Spaces/"), args.flag_source.as_deref());
        assert_eq!(Some("../another"), args.flag_target.as_deref());
        assert_eq!(true, args.flag_watch);
    }
}
