use std::path::Path;
use std::io::prelude::*;
use std::fs::{File, create_dir_all};
use renderable::Renderable;
use page::PageInfo;

use failure::Error;

static STYLE: &'static str = include_str!("../style.css");

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
        Renderer {
            title,
            footer
        }
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

        let mut file = File::create(&output_path)?;

        // HTML header, containing hardcoded CSS
        write!(
            file,
            r#"<html>
  <head>
    <title>{}</title>
    <link href="https://fonts.googleapis.com/css?family=Merriweather|Open+Sans" rel="stylesheet">
    <style>{}</style>
  </head>
<body>"#,
            title,
            &STYLE
        )?;

        renderable.write_header(&mut file, &self.title)?;
        renderable.write_body(&mut file)?;

        // footer finishes body.
        write!(file, "<footer>{}</footer></body>", self.footer)?;

        Ok(PageInfo {
            title: title.into(),
            slug: slug,
            path: output_path,
        })
    }
}
