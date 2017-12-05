use pulldown_cmark::*;

/// # A single ement in the TOC
///
/// Represents either a captured event from Pulldown or a header
/// element.
pub enum TocElement {
    /// Pre-Rendered HTML block
    Html(String),

    /// TOC references
    TocReference,

    /// A heading
    Heading(Heading, Vec<TocElement>),
}

/// # A heading
pub struct Heading {
    /// The header level. H1 .. H6
    pub level: i32,

    /// The raw contents for this heading.
    pub contents: String,
}

/// Parse a TOC tree from the headers in the markdown document
pub fn parse_toc<'a, P: Iterator<Item = Event<'a>>>(parser: P) -> Vec<TocElement> {
    let mut toc = Vec::new();
    let mut current = Vec::new();

    for e in parser {
        match e {
            Event::Start(Tag::Header(_)) => {
                toc.push(TocElement::Html(render_to_string(current.drain(..))));
            }
            Event::End(Tag::Header(i)) => {
                toc.push(TocElement::Heading(
                    Heading {
                        level: i,
                        contents: render_to_string(current.drain(..)),
                    },
                    Vec::new(),
                ));
            }
            Event::Text(ref t) if t == "[TOC]" => {
                toc.push(TocElement::Html(render_to_string(current.drain(..))));
                toc.push(TocElement::TocReference);
            }
            e => current.push(e),
        }
    }

    if current.len() > 0 {
        let rendered = render_to_string(current.into_iter());
        toc.push(TocElement::Html(rendered));
    }

    toc
}

/// # Render Pulldown Events to a String
///
/// Takes a set of captured events and renders them to a string.
fn render_to_string<'a, I>(events: I) -> String
where
    I: Iterator<Item = Event<'a>>,
{
    let mut rendered = String::new();
    html::push_html(&mut rendered, events.into_iter());
    rendered
}

impl Heading {
    /// Get the Plain Text Representation of the heading
    pub fn plain_header(&self) -> String {
        self.contents.clone()
    }
}

#[cfg(test)]
mod test {

    // use super::*;

    // #[test]
    // fn parse_wit_no_headings() {
    //     let doc = "hello world";
    //     let mut parser = Parser::new(doc);

    //     let toc = parse_toc(&mut parser);

    //     assert_eq!(0, toc.len());
    // }

    // #[test]
    // fn parse_with_single_heading() {
    //     let doc = "# I am an H1";
    //     let mut parser = Parser::new(doc);

    //     let toc = parse_toc(&mut parser);

    //     assert_eq!(
    //         Vec::<TocElement>::new(),
    //         toc
    //     );
    // }
}
