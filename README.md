# Docket

[![Build Status][build_badge_image]][build_info]

Simple markdown to HTML documentation rendering. Docket aims to be a Rust clone of [`d`](https://github.com/sjl/d).

## Key Features

* Binary which can be installed with `cargo install`
* Command line argument parsing with [Docopt](https://docs.rs/docopt/0.8.1/docopt/)
* Markdown rendering with `pulldown-cmark`.
* Zero-configuration.

## Installation

Docket can be installed with cargo via `cargo install docket`. Once installed you should be able to run it form the command line as `docket`.

Docket has two Cargo features which are enabled by default. You can disable them with `--no-default-features` when installing if you don't need them to save some time.

 * `watch` - Support for watching files and re-generating the output folder when changes are made.
 * `par_render` - Support for rendering pages in parallel using the Rayon crate.

## Getting Started

To begin creating your documentation create a new `docs/` folder at the root of your repository. Add a file called `index.md` with a short markdown description of the project. Add pages by creating new markdown files in the `docs/` folder. Each page should have a level-1 heading at the beginning which is treated as the title of the page. 

To render the HTML output change to the `docs/` folder and run `docket`. This should create a new `docs/build/` folder containing the rendered site; ready to be published to a web-server or served with GitHub Pages. For more information about setup and configuration [check out the docs](https://iwillspeak.github.io/docket/).

 [build_badge_image]: https://dev.azure.com/iwillspeak/GitHub/_apis/build/status/iwillspeak.docket?branchName=master
 [build_info]: https://dev.azure.com/iwillspeak/GitHub/_build/latest?definitionId=1&branchName=master
