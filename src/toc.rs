//! Tree of Contents
//!
//! This module defines a tree structre over the events of `pulldown_cmark`. The
//! tree can be rendered into HTML with `pulldown`, or quieried for the document
//! layout in order to produce navigation elements.

use std::iter::Peekable;

use pulldown_cmark::*;
use log::error;

use crate::build;

/// # A single ement in the TOC
///
/// Represents either a captured event from Pulldown or a header
/// element.
#[derive(Debug)]
pub(crate) enum TocElement<'a> {
    /// Raw Pulldown events
    Events(Vec<Event<'a>>),

    /// TOC references
    TocReference,

    /// A node in the tree
    Node(TocNode<'a>),
}

/// # A heading
#[derive(Debug)]
pub(crate) struct Heading<'a> {
    /// The header level. H1 .. H6
    pub level: HeadingLevel,

    /// The heading's fragment ID, if any,
    pub frag: Option<&'a str>,

    /// The heading's fragment ID, if any,
    pub classes: Vec<&'a str>,

    /// The raw contents for this heading.
    pub contents: Vec<Event<'a>>,
}

#[derive(Debug)]
pub(crate) struct TocNode<'a> {
    /// The heading at this node
    heading: Heading<'a>,

    /// The TOC contents for this node.
    contents: Vec<TocElement<'a>>,
}

/// Parse a TOC tree from the headers in the markdown document
pub(crate) fn parse_toc<'a, I>(events: I) -> Vec<TocElement<'a>>
where
    I: Iterator<Item = Event<'a>>,
{
    parse_toc_at_level(None, &mut events.peekable())
}

fn parse_toc_at_level<'a, I>(level: Option<HeadingLevel>, events: &mut Peekable<I>) -> Vec<TocElement<'a>>
where
    I: Iterator<Item = Event<'a>>,
{
    let mut heading = None; 
    let mut elements = Vec::new();

    while is_below(level, events.peek()) {
        match events.next() {
            Some(event) => match event {
                // If we see a heading tag then start building a heading
                Event::Start(Tag::Heading(level, frag, classes)) => {
                    heading = Some(Heading {
                        level,
                        frag,
                        classes,
                        contents: Vec::new(),
                    });
                }
                // If we see a heading end tag then recurse to parse any
                // elements owned by that theading
                Event::End(Tag::Heading(level, ..)) => {
                    match heading.take() {
                        Some(heading) =>                     elements.push(TocElement::Node(TocNode { heading, contents: parse_toc_at_level(Some(level), events)})),
                        None => error!("Invalid or mis-matched heading at {level}"),
                    }
                },
                // A normal event
                _ => match heading {
                    // We're building a heading, so add it to the heading
                    Some(ref mut head) => head.contents.push(event),
                    // Add this as an event to the current events elemnet
                    None => match elements.last_mut() {
                        Some(TocElement::Events(evs)) => evs.push(event),
                        _ => elements.push(TocElement::Events(vec![event])),
                    },
                },
            },
            None => break
        }
    }

    elements
}

fn is_below(level: Option<HeadingLevel>, event: Option<&Event>) -> bool {
    match level {
        Some(level) => match event {
            Some(Event::Start(Tag::Heading(ref next_level, ..))) => *next_level > level,
            None => false,
            _ => true,
        },
        None => true,
    }
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

#[cfg(test)]
mod inegration_tests {
    use pulldown_cmark::Parser;

    use super::parse_toc;

    #[test]
    fn parse_example_doc_toc() {
        let parser = Parser::new(
            "
# Heading 1.1

para one

## Heading 2.1

para two

para three

### Heading 3.1

```code
block four
```

## Heading 2.2

<img src=example.com/png>

# Heading 1.2

> last bit",
        );
        let toc = parse_toc(parser);

        assert_eq!(2, toc.len());
    }
}

#[cfg(all(test, nope))]
mod test {

    use super::*;

    fn h(level: HeadingLevel, contents: &str) -> Heading {
        let slug = utils::slugify(contents);
        Heading {
            level,
            contents: contents.into(),
            slug,
        }
    }

    #[test]
    fn parse_with_no_headings() {
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
