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
    Heading(Heading)
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
                let mut rendered = String::new();
                html::push_html(&mut rendered, current.into_iter());
                current = Vec::new();
                toc.push(TocElement::Html(rendered));
            }
            Event::End(Tag::Header(i)) => {
                let mut rendered = String::new();
                html::push_html(&mut rendered, current.into_iter());
                current = Vec::new();
                toc.push(TocElement::Heading(Heading{
                    level: i,
                    contents: rendered,
                }));

                current.clear();
            }
            Event::Text(ref t) if t == "[TOC]" => {
                let mut rendered = String::new();
                html::push_html(&mut rendered, current.into_iter());
                current = Vec::new();
                toc.push(TocElement::Html(rendered));
                toc.push(TocElement::TocReference);
            }
            e => current.push(e),
        }
    }

    if current.len() > 0 {
        let mut rendered = String::new();
        html::push_html(&mut rendered, current.into_iter());
        toc.push(TocElement::Html(rendered));
    }

    toc
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
