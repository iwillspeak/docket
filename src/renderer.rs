use crate::page::PageInfo;
use crate::renderable::Renderable;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::Path;

use failure::Error;

/// Render Context
///
/// Holds the state of the current render session (basically just the
/// footer). Responsible for knowing how to take the HTML from a
/// `Renderable` and write it out to a file.
pub struct Renderer {
    /// The title of the project
    title: String,

    /// The rendered page footer
    footer: String,
}

impl Renderer {
    /// Create a new `Renderer`
    ///
    /// # Arguments
    ///  - `footer` The footer to render at the bottom of each page.
    pub fn new(title: String, footer: String) -> Self {
        Renderer { title, footer }
    }

    /// Render the page
    ///
    /// Takes the path to the output directory where the page should
    /// be rendered.
    pub fn render<T: Renderable>(&self, renderable: &T, path: &Path) -> Result<PageInfo, Error> {
        let slug = renderable.get_slug();
        let title = renderable.get_title();

        // Create the directory to render to
        let output_dir = path.join(&slug);
        let output_path = output_dir.join("index.html");
        create_dir_all(&output_dir)?;

        let style_path = format!("{}/style.css", renderable.path_to_root());

        let mut file = File::create(&output_path)?;

        // HTML header, containing hardcoded CSS
        write!(
            file,
            r#"<html>
  <head>
    <title>{}</title>
    <meta name="viewport" content="width=700">
    <meta charset="UTF-8"> 
    <link href="https://fonts.googleapis.com/css?family=Merriweather|Open+Sans" rel="stylesheet">
    <link rel="stylesheet"
      href="//cdnjs.cloudflare.com/ajax/libs/highlight.js/9.12.0/styles/default.min.css">
    <script src="//cdnjs.cloudflare.com/ajax/libs/highlight.js/9.12.0/highlight.min.js"></script>
    <link rel="stylesheet" href="{}">
    <script>hljs.initHighlightingOnLoad();</script>
  </head>
  <body>"#,
            title, style_path,
        )?;

        renderable.write_header(&mut file, &self.title)?;
        renderable.write_body(&mut file)?;

        // footer finishes body.
        write!(file, "<footer>{}</footer></body></html>", self.footer)?;

        Ok(PageInfo {
            title: title.into(),
            slug,
            path: output_path,
        })
    }
}
