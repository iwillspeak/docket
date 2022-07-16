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

/// Get the command line arguments
pub(crate) fn from_command_line() -> Args {
    Docopt::new(USAGE)
        .map(|d| {
            d.help(true)
                .version(Some(format!("Docket {}", env!("CARGO_PKG_VERSION"))))
        })
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit())
}
