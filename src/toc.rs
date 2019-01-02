use crate::util;
use pulldown_cmark::*;
use std::borrow::Cow;
use std::iter::Peekable;

/// # A single ement in the TOC
///
/// Represents either a captured event from Pulldown or a header
/// element.
#[derive(Debug, PartialEq)]
pub enum TocElement {
    /// Pre-Rendered HTML block
    Html(String),

    /// TOC references
    TocReference,

    /// A heading
    Heading(Heading, Vec<TocElement>),
}

/// # A heading
#[derive(Debug, PartialEq)]
pub struct Heading {
    /// The header level. H1 .. H6
    pub level: i32,

    /// The raw contents for this heading.
    pub contents: String,

    /// The sanitized slug for this heading.
    pub slug: String,
}

/// Parse a TOC tree from the headers in the markdown document
pub fn parse_toc<'a, P: Iterator<Item = Event<'a>>>(parser: P) -> Vec<TocElement> {
    parse_toc_at(&mut parser.peekable(), 0)
}

fn parse_toc_at<'a, P>(parser: &mut Peekable<P>, header_level: i32) -> Vec<TocElement>
where
    P: Iterator<Item = Event<'a>>,
{
    let mut toc = Vec::new();
    let mut current = Vec::new();

    loop {
        if let Some(&Event::Start(Tag::Header(i))) = parser.peek() {
            if i <= header_level {
                break;
            }
        }
        if let Some(e) = parser.next() {
            match e {
                Event::Start(Tag::Header(_)) => {
                    if !current.is_empty() {
                        toc.push(TocElement::Html(render_to_string(current.drain(..))));
                    }
                }
                Event::End(Tag::Header(i)) => {
                    toc.push(TocElement::Heading(
                        Heading::from_events(i, current.drain(..)),
                        parse_toc_at(parser, i),
                    ));
                }
                // Replace code blocks which start with `:::`-lines
                // with ones containing the appropriate type.
                Event::Text(ref t) if t.starts_with(":::") => {
                    let last = current.pop();
                    if let Some(event) = last {
                        if let Event::Start(Tag::CodeBlock(Cow::Borrowed(""))) = event {
                            current
                                .push(Event::Start(Tag::CodeBlock(String::from(&t[3..]).into())));
                        } else {
                            current.push(event);
                            current.push(Event::Text(t.clone()));
                        }
                    }
                }
                // Replace references to the TOC, but only if they are
                // at the start of a new paragraph. This allows the
                // escaping of the prhase with code blocks and within
                // longer streams of text
                Event::Text(ref t) if t == "[TOC]" => {
                    if let Some(&Event::Start(Tag::Paragraph)) = current.last() {
                        if !current.is_empty() {
                            toc.push(TocElement::Html(render_to_string(current.drain(..))));
                        }
                        toc.push(TocElement::TocReference);
                    } else {
                        current.push(Event::Text("[TOC]".into()))
                    }
                }
                e => current.push(e),
            }
        } else {
            break;
        }
    }

    if !current.is_empty() {
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
    html::push_html(&mut rendered, events);
    rendered
}

impl Heading {
    pub fn from_events<'a, I>(level: i32, events: I) -> Self
    where
        I: Iterator<Item = Event<'a>>,
    {
        let mut slug = String::new();
        let contents = render_to_string(events.inspect(|e| {
            if let Event::Text(ref t) = *e {
                slug.push_str(&t)
            }
        }));
        let slug = util::slugify(&slug);
        Heading {
            level,
            contents,
            slug,
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn h(level: i32, contents: &str) -> Heading {
        let slug = util::slugify(contents);
        Heading {
            level,
            contents: contents.into(),
            slug,
        }
    }

    #[test]
    fn parse_wit_no_headings() {
        let doc = "hello world";
        let mut parser = Parser::new(doc);

        let toc = parse_toc(&mut parser);

        assert_eq!(vec![TocElement::Html("<p>hello world</p>\n".into())], toc);
    }

    #[test]
    fn parse_with_single_heading() {
        let doc = "# I am an H1";
        let mut parser = Parser::new(doc);

        let toc = parse_toc(&mut parser);

        assert_eq!(
            vec![TocElement::Heading(h(1, "I am an H1"), Vec::new())],
            toc
        );
    }

    #[test]
    fn parse_with_single_toc_reference() {
        let doc = "[TOC]";
        let mut parser = Parser::new(doc);

        let toc = parse_toc(&mut parser);

        assert_eq!(
            vec![
                TocElement::Html("<p>".into()),
                TocElement::TocReference,
                TocElement::Html("</p>\n".into()),
            ],
            toc
        );
    }

    #[test]
    fn parse_with_nested_headings() {
        let doc = r#"
# Heading 1.1

## Heading 2.1

### Heading 3.1

## Heading 2.2

# Heading 1.2
"#;
        let mut parser = Parser::new(doc);

        let toc = parse_toc(&mut parser);

        assert_eq!(
            vec![
                TocElement::Heading(
                    h(1, "Heading 1.1"),
                    vec![
                        TocElement::Heading(
                            h(2, "Heading 2.1"),
                            vec![TocElement::Heading(h(3, "Heading 3.1"), Vec::new())],
                        ),
                        TocElement::Heading(h(2, "Heading 2.2"), Vec::new()),
                    ],
                ),
                TocElement::Heading(h(1, "Heading 1.2"), Vec::new()),
            ],
            toc
        )
    }
}
