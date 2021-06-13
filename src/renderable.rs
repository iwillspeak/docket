use std::borrow::Cow;
use std::collections::HashMap;
use std::io::{self, Write};

/// Defines a Renderable component
///
/// Renderables can be consumed by the `Renderer` and used to write
/// HTML out to disk. This abstracts over standard `Page` instances
/// and the index page.
pub trait Renderable {
    /// Get the slug of the page
    ///
    /// This should be suitable for use in a URI.
    fn get_slug(&self) -> String;

    /// Get the tilte of the page, so that rendering can place it in
    /// the <head>
    fn get_title(&self) -> Cow<'_, str>;

    /// Write the heading to the output file.
    fn write_header<T: Write>(&self, f: &mut T, title: &str) -> io::Result<()>;

    /// Write the body to the output file.
    fn write_body<T: Write>(&self, _: &mut T) -> io::Result<()>;

    /// Path from this renderbabel to the Root
    fn path_to_root(&self) -> Cow<'_, str>;

    // Get the search index for this renderable, if any.
    fn get_search_index(&self) -> Option<HashMap<String, i32>>;
}
