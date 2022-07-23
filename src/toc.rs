//! Tree of Contents
//!
//! This module defines a tree structre over the events of `pulldown_cmark`. The
//! tree can be rendered into HTML with `pulldown`, or quieried for the document
//! layout in order to produce navigation elements.

use std::{
    borrow::Borrow,
    iter::Peekable,
    os::raw::c_uchar,
    thread::{current, park_timeout},
};

use pulldown_cmark::*;

use crate::utils;

/// # A single ement in the TOC
///
/// Represents either a captured event from Pulldown or a header
/// element.
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
#[derive(Debug, PartialEq)]
pub(crate) struct Heading {
    /// The header level. H1 .. H6
    pub level: HeadingLevel,

    /// The raw contents for this heading.
    pub contents: String,

    /// The clean text for this header.
    pub clean_text: String,

    /// The fragment identifier, or slug, to use for this heading.
    pub slug: String,
}

#[derive(Debug, PartialEq)]
pub(crate) struct TocNode {
    /// The heading at this node
    heading: Heading,

    /// The TOC contents for this node.
    contents: Vec<TocElement>,
}

/// Get the inner text from a series of events. used to create a heading name
/// from a series of events, or to find the text that should be
pub fn events_to_plain<'a, I, E>(events: I) -> String
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

fn events_to_html(events: &mut Vec<Event>) -> Option<String> {
    if events.is_empty() {
        None
    } else {
        let mut result = String::new();
        pulldown_cmark::html::push_html(&mut result, events.drain(..));
        Some(result)
    }
}

pub(crate) fn parse_toc(markdown: &str) -> Vec<TocElement> {
    let parser = Parser::new_ext(markdown, Options::all());
    parse_toc_events(parser)
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

    while is_below(level, events.peek()) {
        match events.next() {
            Some(event) => match event {
                // If we see a heading tag then start building a heading
                ev @ Event::Start(Tag::Heading(..)) => {
                    if let Some(element) = events_to_html(&mut buffered) {
                        elements.push(TocElement::Html(element));
                    }
                    buffered.push(ev)
                }
                // If we see a heading end tag then recurse to parse any
                // elements owned by that theading
                Event::End(Tag::Heading(level, frag, class)) => {
                    buffered.push(Event::End(Tag::Heading(level, frag, class)));
                    let clean_text = events_to_plain(buffered.iter());
                    let slug = frag
                        .map(|s| s.to_owned())
                        .unwrap_or_else(|| utils::slugify(&clean_text));
                    elements.push(TocElement::Node(TocNode {
                        heading: Heading {
                            level,
                            contents: events_to_html(&mut buffered).unwrap_or(String::new()),
                            clean_text,
                            slug,
                        },
                        contents: parse_toc_at_level(Some(level), events),
                    }))
                }
                Event::Start(Tag::Link(LinkType::ShortcutUnknown, dest, title)) => {
                    if dest.as_ref() == "toc://marker" {
                        while let Some(event) = events.next() {
                            if let Event::Start(Tag::Link(LinkType::ShortcutUnknown, _, _)) = event
                            {
                                break;
                            }
                        }
                        if let Some(element) = events_to_html(&mut buffered) {
                            elements.push(TocElement::Html(element));
                        }
                        elements.push(TocElement::TocReference)
                    } else {
                        buffered.push(Event::Start(Tag::Link(
                            LinkType::ShortcutUnknown,
                            dest,
                            title,
                        )))
                    }
                }
                Event::End(Tag::Paragraph) => {
                    if in_toc(&buffered) {
                        buffered.truncate(buffered.len() - 4);
                        if let Some(html) = events_to_html(&mut buffered) {
                            elements.push(TocElement::Html(html));
                        }
                        elements.push(TocElement::TocReference);
                    } else {
                        buffered.push(Event::End(Tag::Paragraph));
                    }
                }
                // A normal event
                ev => buffered.push(ev),
            },
            None => break,
        }
    }

    if let Some(element) = events_to_html(&mut buffered) {
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

#[cfg(test)]
mod test {
    use super::*;
    fn h(level: HeadingLevel, contents: &str) -> Heading {
        let clean_text = contents.to_string();
        let slug = utils::slugify(&clean_text);
        Heading {
            level,
            clean_text,
            contents: format!("<{0}>{1}</{0}>\n", level, contents),
            slug,
        }
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

pub(crate) fn primary_heading(toc: &[TocElement]) -> Option<&String> {
    toc.iter().find_map(|element| match element {
        TocElement::Node(node) => Some(&node.heading.clean_text),
        _ => None,
    })
}

fn render_html_to(output: &mut String, tree: &[TocElement], current: &[TocElement]) {
    for element in current {
        match element {
            TocElement::Html(htm) => output.push_str(htm),
            TocElement::TocReference => render_toc_to(output, &tree),
            TocElement::Node(nested) => {
                // TODO: this should be a little less `String`y. Can probably take a writer inestad to format to.
                output.push_str(&format!(
                    "<a id='{0}' href='#{0}'>{1}</a>",
                    &nested.heading.slug, &nested.heading.contents
                ));
                render_html_to(output, tree, &nested.contents);
            }
        }
    }
}

fn render_toc_to(output: &mut String, tree: &[TocElement]) {
    output.push_str("<ul class='toc'>");
    for element in tree {
        if let TocElement::Node(node) = element {
            output.push_str(&format!(
                "<li><a href='#{0}'>{1}</a>",
                node.heading.slug, node.heading.clean_text
            ));
        }
    }
    output.push_str("</ul>");
}

pub(crate) fn render_html(tree: &[TocElement]) -> String {
    let mut output = String::new();
    render_html_to(&mut output, &tree, &tree);

    output
}
