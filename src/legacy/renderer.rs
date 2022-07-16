use super::renderable::Renderable;
use super::Result;
use super::{highlight, page::PageInfo};
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::Path;

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
    pub fn render<T: Renderable>(&self, renderable: &T, path: &Path) -> Result<PageInfo> {
        let slug = renderable.get_slug();
        let title = renderable.get_title();

        // Create the directory to render to
        let output_dir = path.join(&slug);
        let output_path = output_dir.join("index.html");
        create_dir_all(&output_dir)?;

        let style_path = format!("{}/style.css", renderable.path_to_root());
        let script_path = format!("{}/search.js", renderable.path_to_root());

        let mut file = File::create(&output_path)?;

        // HTML header, containing hardcoded CSS
        write!(
            file,
            r##"<!DOCTYPE html>
                <html>
                <head>
                    <title>{}</title>
                    <meta name="viewport" content="width=device-wdith,initial-scale=1">
                    <meta charset="UTF-8">
                    <link rel="preconnect" href="https://fonts.googleapis.com">
                    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
                    <link href="https://fonts.googleapis.com/css2?family=Montserrat&display=swap" rel="stylesheet"> 
                    <link rel="stylesheet" href="{}">
                    <script src="{}"></script>
                    <script>initialiseSearch("{}", "#docket-search");</script>"##,
            title,
            style_path,
            script_path,
            renderable.path_to_root()
        )?;
        highlight::get_hilighter().write_header(&mut file)?;
        write!(
            file,
            r#"
              </head>
            <body>"#
        )?;

        write!(file, r#"<nav class="side-nav">"#)?;
        renderable.write_nav(&mut file)?;
        write!(file, "</nav>")?;

        // Main content wrapped in a section
        write!(
            file,
            r#"
        <main>
          <section class="content">"#
        )?;

        renderable.write_header(&mut file, &self.title)?;
        renderable.write_body(&mut file)?;

        // footer finishes content section.
        write!(file, "<footer>{}</footer>", self.footer)?;
        write!(file, r#"</section></main>"#)?;

        // Finish tthe
        write!(file, "</body></html>")?;

        Ok(PageInfo {
            title: title.into(),
            slug,
            path: output_path,
            search_index: renderable.get_search_index(),
        })
    }
}
