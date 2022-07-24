//! Tree of Contents
//!
//! This module defines a tree structre over the events of `pulldown_cmark`. The
//! tree can be rendered into HTML with `pulldown`, or quieried for the document
//! layout in order to produce navigation elements.

use std::{borrow::Borrow, iter::Peekable};

use pulldown_cmark::*;

use crate::utils;

/// # A single ement in the TOC
///
/// Represents either a pre-rendered HTML blob, a reference to insert the full
/// TOC, or a nested toc node.
#[derive(Debug, PartialEq)]
pub(crate) enum TocElement {
    /// Raw Pulldown events
    Html(String),

    /// TOC references
    TocReference,

    /// A node in the tree
    Node(TocNode),
}

/// # A heading
///
/// Headings from the raw markdown document with extra metadata to allow the TOC
/// to be rendered.
#[derive(Debug, PartialEq)]
pub(crate) struct Heading {
    /// The header level. H1 .. H6
    pub level: HeadingLevel,

    /// The raw contents for this heading.
    pub contents: String,

    /// The fragment identifier, or slug, to use for this heading.
    pub slug: String,
}

/// # TOC Node
///
/// A node in the TOC tree. A node has a heading that introduced the node and a
/// list of children.
#[derive(Debug, PartialEq)]
pub(crate) struct TocNode {
    /// The heading at this node
    pub heading: Heading,

    /// The TOC contents for this node.
    pub contents: Vec<TocElement>,
}

impl TocNode {
    pub fn nodes(&self) -> Nodes {
        Nodes(&self.contents, 0)
    }
}

/// Toc Element Iterator
///
/// This iterator performs a depth-first walk of the element tree
pub(crate) struct Elements<'a>(Vec<&'a TocElement>);

impl<'a> Elements<'a> {
    pub fn new(elements: &'a [TocElement]) -> Self {
        Elements(Vec::from_iter(elements.iter().rev()))
    }
}

impl<'a> Iterator for Elements<'a> {
    type Item = &'a TocElement;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0.pop();
        if let Some(TocElement::Node(node)) = next {
            self.0.extend(node.contents.iter().rev())
        }

        next
    }
}

/// Toc Node Iterator
///
/// Enumerates all the nodes within a given set of elements.
pub(crate) struct Nodes<'a>(&'a [TocElement], usize);

impl<'a> Iterator for Nodes<'a> {
    type Item = &'a TocNode;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(element) = self.0.get(self.1) {
            self.1 = self.1 + 1;
            if let TocElement::Node(node) = element {
                return Some(&node);
            }
        }

        None
    }
}

/// # Tree of Contents
///
/// The tree of contents is the basic unit of pages within the document tree. A
/// page contains a single Tree of Contents. The tree is a list of elements
/// which mirror the nesting of the document's heading structure.
///
/// A tree can be queried for information about the document's outline, primary
/// heading, or full contnet. The layout module uses the public API of the `Toc`
/// to render out page's contents, internal navigation, and title information.
#[derive(Debug)]
pub(crate) struct Toc(Vec<TocElement>);

impl Toc {
    /// # Parse a Tree of Contents
    ///
    /// Given a markdown string parse it and return a vector containing the
    /// top-level elements in the document's tree.
    pub fn new(markdown: &str) -> Self {
        let parser = Parser::new_ext(markdown, Options::all());
        Toc(parse_toc_events(parser))
    }

    /// # Primary Heading
    ///
    /// Get the first heading within the tree. If the tree contains no headings
    /// then `None` is returned.
    pub fn primary_heading(&self) -> Option<&String> {
        self.0.iter().find_map(|element| match element {
            TocElement::Node(node) => Some(&node.heading.contents),
            _ => None,
        })
    }

    /// # Get the Nodes Iterator
    ///
    /// Returns an iterator over the nodes within the root of the tree.
    pub fn nodes(&self) -> Nodes {
        Nodes(&self.0, 0)
    }

    /// # Depth-frist walk of the elements of the tree
    pub fn walk_elements(&self) -> Elements {
        Elements::new(&self.0)
    }

    /// # Unwrap the Inner Elements
    #[cfg(test)]
    fn into_inner(self) -> Vec<TocElement> {
        self.0
    }
}

/// Get the inner text from a series of events. used to create a heading name
/// from a series of events, or to find the text that should be
fn events_to_plain<'a, I, E>(events: I) -> String
where
    I: Iterator<Item = E>,
    E: Borrow<Event<'a>>,
{
    let mut text = String::new();

    for ev in events {
        match ev.borrow() {
            Event::Text(txt) => text.push_str(txt.as_ref()),
            Event::Code(code) => text.push_str(code.as_ref()),
            Event::Html(htm) => text.push_str(htm.as_ref()),
            _ => (),
        }
    }

    text
}

/// # Drain events to HTML
///
/// If the events vector contains any buffered events then return the rendred
/// HTML. If the buffer is empty then return `None`. This utility is used during
/// the TOC walk to ensure we always render HTML if we have events buffered, and
/// that we don't include spurious HTML nodes when there are no buffered events.
fn drain_events_to_html(events: &mut Vec<Event>) -> Option<String> {
    if events.is_empty() {
        None
    } else {
        let mut result = String::new();
        pulldown_cmark::html::push_html(&mut result, events.drain(..));
        Some(result)
    }
}

/// Parse a TOC tree from the headers in the markdown document
fn parse_toc_events<'a, I>(events: I) -> Vec<TocElement>
where
    I: Iterator<Item = Event<'a>>,
{
    parse_toc_at_level(None, &mut events.peekable())
}

/// Parse the toc tree at a given header level.
fn parse_toc_at_level<'a, I>(
    level: Option<HeadingLevel>,
    events: &mut Peekable<I>,
) -> Vec<TocElement>
where
    I: Iterator<Item = Event<'a>>,
{
    let mut buffered = Vec::new();
    let mut elements = Vec::new();

    while let Some(event) = events.next_if(|event| is_below(level, event)) {
        match event {
            // If we see a heading tag then start building a heading
            Event::Start(Tag::Heading(..)) => {
                if let Some(element) = drain_events_to_html(&mut buffered) {
                    elements.push(TocElement::Html(element));
                }
            }
            // If we see a heading end tag then recurse to parse any
            // elements owned by that heading.
            Event::End(Tag::Heading(level, frag, _class)) => {
                // Not we didn't push the opening event _and_ we ignore the
                // closing one here too. This means we will only render the
                // _contents_ of the header, not the opening and closing tags.
                let slug = frag
                    .map(|s| s.to_owned())
                    .unwrap_or_else(|| utils::slugify(&events_to_plain(buffered.iter())));
                elements.push(TocElement::Node(TocNode {
                    heading: Heading {
                        level,
                        contents: drain_events_to_html(&mut buffered).unwrap_or(String::new()),
                        slug,
                    },
                    contents: parse_toc_at_level(Some(level), events),
                }))
            }
            // If we see a closing paragraph then check if we're looking at
            // a `[TOC]` reference. If we are then replace the paragraph
            // with a marker.
            Event::End(Tag::Paragraph) => {
                if in_toc(&buffered) {
                    buffered.truncate(buffered.len() - 4);
                    if let Some(html) = drain_events_to_html(&mut buffered) {
                        elements.push(TocElement::Html(html));
                    }
                    elements.push(TocElement::TocReference);
                } else {
                    buffered.push(Event::End(Tag::Paragraph));
                }
            }
            // A normal event
            ev => buffered.push(ev),
        }
    }

    // If we have any events left then make sure to append them here.
    if let Some(element) = drain_events_to_html(&mut buffered) {
        elements.push(TocElement::Html(element));
    }

    elements
}

/// Check if we have just seen a `<p>`, `[`, `TOC`, and `]`
fn in_toc(current: &[Event]) -> bool {
    let idx = current.len() - 1;
    if let Some(Event::Text(ref toc)) = current.get(idx) {
        if toc.as_ref() != "]" {
            return false;
        }
    } else {
        return false;
    }
    if let Some(Event::Text(ref toc)) = current.get(idx - 1) {
        if toc.as_ref() != "TOC" {
            return false;
        }
    } else {
        return false;
    }
    if let Some(Event::Text(ref toc)) = current.get(idx - 2) {
        if toc.as_ref() != "[" {
            return false;
        }
    } else {
        return false;
    }
    if let Some(Event::Start(Tag::Paragraph)) = current.get(idx - 3) {
        true
    } else {
        false
    }
}

// Check if the current event should live below the given heading level.
fn is_below(level: Option<HeadingLevel>, event: &Event) -> bool {
    level
        .map(|level| match event {
            Event::Start(Tag::Heading(ref next_level, ..)) => *next_level > level,
            _ => true,
        })
        .unwrap_or(true)
}

#[cfg(test)]
mod test {
    use super::*;
    fn h(level: HeadingLevel, contents: &str) -> Heading {
        let slug = utils::slugify(&contents);
        Heading {
            level,
            contents: format!("<{0}>{1}</{0}>\n", level, contents),
            slug,
        }
    }

    fn parse_toc(s: &str) -> Vec<TocElement> {
        Toc::new(s).into_inner()
    }

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
        let toc = parse_toc_events(parser);

        assert_eq!(2, toc.len());
    }

    #[test]
    fn parse_with_no_headings() {
        let doc = "hello world";
        let parser = Parser::new(doc);

        let toc = parse_toc_events(parser);

        assert_eq!(vec![TocElement::Html("<p>hello world</p>\n".into())], toc);
    }

    #[test]
    fn parse_with_single_heading() {
        let doc = "# I am an H1";

        let toc = parse_toc(doc);

        assert_eq!(
            vec![TocElement::Node(TocNode {
                heading: h(HeadingLevel::H1, "I am an H1"),
                contents: Vec::new()
            })],
            toc
        );
    }

    #[test]
    fn parse_with_single_toc_reference() {
        let doc = "[TOC]";

        let toc = parse_toc(&doc);

        assert_eq!(vec![TocElement::TocReference,], toc);
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

        let toc = parse_toc(doc);

        assert_eq!(
            vec![
                TocElement::Node(TocNode {
                    heading: h(HeadingLevel::H1, "Heading 1.1"),
                    contents: vec![
                        TocElement::Node(TocNode {
                            heading: h(HeadingLevel::H2, "Heading 2.1"),
                            contents: vec![TocElement::Node(TocNode {
                                heading: h(HeadingLevel::H3, "Heading 3.1"),
                                contents: Vec::new()
                            })],
                        }),
                        TocElement::Node(TocNode {
                            heading: h(HeadingLevel::H2, "Heading 2.2"),
                            contents: Vec::new()
                        }),
                    ]
                }),
                TocElement::Node(TocNode {
                    heading: h(HeadingLevel::H1, "Heading 1.2"),
                    contents: Vec::new()
                }),
            ],
            toc
        )
    }
}
