use super::renderable::Renderable;
use super::search::{TermFrequenciesBuilder, TermFrequenciesIndex};
use super::toc::*;
use super::util;
use log::debug;
use pulldown_cmark::*;
use std::borrow::Cow;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::{fs, io};

/// Page to be Rendered
pub struct Page<'a> {
    path: &'a Path,
    title: String,
    toc: Vec<TocElement>,
}

/// Result of page rendering
pub struct PageInfo {
    /// Title of the page.
    pub title: String,

    /// The slug of the page.
    pub slug: String,

    /// The path this page was rendered to.
    pub path: PathBuf,

    /// search terms for this document.
    pub search_index: Option<TermFrequenciesIndex>,
}

fn render_tree<'a, W: Write, I: Iterator<Item = &'a TocElement>>(
    tree: I,
    toc: &str,
    w: &mut W,
) -> io::Result<()> {
    for e in tree {
        match *e {
            TocElement::Heading(ref h, ref children) => {
                write!(
                    w,
                    "<h{0} id=\"{1}\"><a href=\"#{1}\">{2}</a></h{0}>",
                    h.level, h.slug, h.contents
                )?;
                render_tree(children.iter(), toc, w)?;
            }
            TocElement::Html(ref html) => {
                write!(w, "{}", html)?;
            }
            TocElement::TocReference => {
                write!(w, "{}", toc)?;
            }
        }
    }

    Ok(())
}

fn toc_to_html<'a, I>(tree: I, depth: i32) -> String
where
    I: Iterator<Item = &'a TocElement>,
{
    let mut rendered = String::new();

    rendered.push_str("<ol>");
    for e in tree {
        if let TocElement::Heading(ref h, ref children) = *e {
            if h.level > depth {
                continue;
            }
            rendered.push_str(&format!("<li><a href=\"#{0}\">{1}</a>", h.slug, h.contents));
            if h.level < depth {
                let child_toc = toc_to_html(children.iter(), depth);
                rendered.push_str(&child_toc);
            }
            rendered.push_str("</li>")
        }
    }
    rendered.push_str("</ol>");

    rendered
}

fn build_toc_search_index<'a, I>(toc: I, index: &mut TermFrequenciesBuilder)
where
    I: Iterator<Item = &'a TocElement>,
{
    for element in toc {
        match element {
            TocElement::Heading(header, children) => {
                index.add_terms(&header.contents);
                build_toc_search_index(children.iter(), index);
            }
            TocElement::Html(raw) => {
                index.add_terms(raw);
            }
            TocElement::TocReference => (),
        }
    }
}

impl<'a> Renderable for Page<'a> {
    fn get_slug(&self) -> String {
        util::slugify_path(self.path)
    }

    fn get_title(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.title)
    }

    fn write_nav<T: Write>(&self, file: &mut T) -> io::Result<()> {
        // Placehoder for the JS search
        write!(file, r#"<div id="docket-search"></div>"#)?;

        // List of sections in the page
        write!(file, "<h2>TOC</h2>")?;
        write!(file, "{}", toc_to_html(self.toc.iter(), 2))?;
        Ok(())
    }

    fn write_header<T: Write>(&self, file: &mut T, title: &str) -> io::Result<()> {
        write!(
            file,
            "<header class=\"page-heading\"><h1><a href=\"#\">{}</h1><a class=\"home-link\" href=\"../\">{}</a></header>",
            self.get_title(),
            title
        )
    }

    fn write_body<T: Write>(&self, file: &mut T) -> io::Result<()> {
        let toc = toc_to_html(self.toc.iter(), 3);
        render_tree(self.toc.iter(), &toc, file)
    }

    fn path_to_root(&self) -> Cow<'_, str> {
        "..".into()
    }

    fn get_search_index(&self) -> Option<TermFrequenciesIndex> {
        Some(self.build_search_index())
    }
}

impl<'a> Page<'a> {
    pub fn new(path: &'a Path) -> Self {
        let source = fs::read_to_string(path).expect("FIXME: Fallible page loading");

        Self::from_parts(path, &source)
    }

    pub fn from_parts(path: &'a Path, source: &str) -> Self {
        let events = Parser::new_ext(
            source,
            Options::ENABLE_TABLES | Options::ENABLE_FOOTNOTES | Options::ENABLE_TASKLISTS,
        )
        .collect::<Vec<_>>();
        let mut toc = parse_toc(events.into_iter());
        let mut title = path.file_stem().unwrap().to_string_lossy().to_string();

        if toc.len() == 1 {
            let first = toc.remove(0);
            if let TocElement::Heading(main_header, children) = first {
                title = main_header.contents;
                toc = children;
            } else {
                toc.push(first);
            }
        }

        debug!("TOC: {:?}", toc);

        Page { path, title, toc }
    }

    pub fn build_search_index(&'a self) -> TermFrequenciesIndex {
        let mut index = TermFrequenciesBuilder::default();
        index.add_terms(&self.title);
        build_toc_search_index(self.toc.iter(), &mut index);
        index.finalise()
    }
}

#[cfg(test)]
mod test {

    use super::super::page::Page;
    use std::path::PathBuf;

    #[test]
    fn page_has_search_terms() {
        let path = PathBuf::from("foo/bar.md");
        let page = Page::from_parts(&path, "Some sample text in some text");

        let index = page.build_search_index().into_raw();
        assert_ne!(0, index.len());
        let some_fq = index.get("some").cloned().unwrap_or_default();
        let sample_fq = index.get("sample").cloned().unwrap_or_default();
        let text_fq = index.get("text").cloned().unwrap_or_default();
        assert_eq!(some_fq, text_fq);
        assert!(some_fq > sample_fq);
    }

    #[test]
    fn index_of_example_markdown() {
        let path = PathBuf::from("foo/bar.md");
        let page = Page::from_parts(
            &path,
            r###"

# Down the Rabbit Hole

Either the well was very deep, or she fell very slowly, for she had
plenty of time as she went down to look about her, and to wonder what
was going to happen next. First, she tried to look down and make out
what she was coming to, but it was too dark to see anything; then she
looked at the sides of the well and noticed that they were filled with
cupboards and book-shelves: here and there she saw maps and pictures
hung upon pegs. She took down a jar from one of the shelves as she
passed; it was labelled "ORANGE MARMALADE," but to her disappointment it
was empty; she did not like to drop the jar for fear of killing
somebody underneath, so managed to put it into one of the cupboards as
she fell past it.

"Well!" thought Alice to herself. "After such a fall as this, I shall
think nothing of tumbling down stairs! How brave they'll all think me at
home! Why, I wouldn't say anything about it, even if I fell off the top
of the house!" (Which was very likely true.)

### Going Down?

Down, down, down. Would the fall _never_ come to an end? "I wonder how
many miles I've fallen by this time?" she said aloud. "I must be getting
somewhere near the centre of the earth. Let me see: that would be four
thousand miles down. I think--" (for, you see, Alice had learnt several
things of this sort in her lessons in the schoolroom, and though this
was not a _very_ good opportunity for showing off her knowledge, as
there was no one to listen to her, still it was good practice to say it
over) "--yes, that's about the right distance--but then I wonder what
Latitude or Longitude I've got to?" (Alice had no idea what Latitude
was, or Longitude either, but thought they were nice grand words to
say.)

        "###,
        );

        assert_eq!("Down the Rabbit Hole", page.title);

        // Check some of the relative frequencies of terms
        let index = page.build_search_index().into_raw();
        assert_ne!(0, index.len());
        let rabbit_fq = index.get("rabbit").cloned().unwrap_or_default();
        assert!(rabbit_fq > 0.0);
        let well_fq = index.get("well").cloned().unwrap_or_default();
        assert!(well_fq > rabbit_fq);
        assert_eq!(
            index.get("distance").cloned().unwrap_or_default(),
            rabbit_fq
        );
        assert!(index.get("down").cloned().unwrap_or_default() > well_fq);

        // Check terms are downcased
        assert_ne!(None, index.get("orange"));
        assert_eq!(None, index.get("MARMALADE"));

        // check we don't have any whitespace or other junk symbols in the index
        assert_eq!(None, index.get(""));
        assert_eq!(None, index.get("!"));
        assert_eq!(None, index.get("-"));
        assert_eq!(None, index.get(" "));
        assert_eq!(None, index.get("\t"));
        assert_eq!(None, index.get("("));
    }
}
