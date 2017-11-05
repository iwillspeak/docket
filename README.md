# Docket

Simple markdown to HTML documentation rendering.

* Binary which can be installed with `cargo install`
* Command line argument parsing with [Docopt](https://docs.rs/docopt/0.8.1/docopt/)
* Markdown rendering with `pulldown-cmark`.
* Zero-configuration.

## Implementation

The main program will just instantiate a `Docket` instance with parameters parsed from the command line. The `Docket` will scan the tree and find pages to be rendered.

When rendering a page:

 * Write out the header, which will contain styling information for the page.
 * Write out the body, transforming the H1 to be a page link, and inserting the TOC into the body.
 * Write out the global footer, if there is one.

Once all pages are rendered will will then know the 'pretty' names for each page (extracted from the H1). We can then write out the index page.