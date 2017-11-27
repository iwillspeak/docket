use std::io::{self, Write};
use std::borrow::Cow;

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
    fn get_title<'a>(&'a self) -> Cow<'a, str>;

    /// Write the body to the output file.
    fn write_body<T: Write>(&self, &mut T) -> io::Result<()>;
}
