use pulldown_cmark::*;

/// A node in the table of contents
#[derive(PartialEq, Debug)]
pub struct TocNode {

    /// The header level of this node.
    ///
    /// Not always the same as the depth.
    level: i32,

    /// The body of the header
    header: String,

    /// A list of child nodes, if there are any
    children: Vec<TocNode>,
}

/// Parse a TOC tree from the headers in the markdown document
pub fn parse_toc<'a, P : IntoIterator<Item=Event<'a>>>(parser: P) -> Vec<TocNode> {
    let mut nodes = Vec::new();
    let mut current = None;
    let mut parser = parser.into_iter();
    while let Some(event) = parser.next() {
        match event {
            Event::Start(Tag::Header(level)) => {
                current = Some(TocNode{
                    level: level,
                    header: String::new(),
                    children: Vec::new(),
                })
            },
            Event::End(Tag::Header(_)) => {
                current = match current {
                    Some(node) => {
                        nodes.push(node);
                        None
                    },
                    None => None,
                }
            }
            Event::Text(text) => {
                current = match current {
                    Some(mut node) => {
                        node.header.push_str(&text);
                        Some(TocNode {
                            level: node.level,
                            header: node.header,
                            children: node.children,
                        })
                    },
                    None => None
                }
            },
            _ => {}
        }
    }
    nodes
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn parse_wit_no_headings() {
        let doc = "hello world";
        let mut parser = Parser::new(doc);

        let toc = parse_toc(&mut parser);

        assert_eq!(0, toc.len());
    }

    #[test]
    fn parse_with_single_heading() {
        let doc = "# I am an H1";
        let mut parser = Parser::new(doc);

        let toc = parse_toc(&mut parser);

        assert_eq!(vec![
            TocNode {
                level: 1,
                header: "I am an H1".to_owned(),
                children: Vec::new()
            }
        ], toc);
    }
}
