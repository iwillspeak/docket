use std::{
    fs, io,
    path::{self, Path},
};

use log::trace;

use crate::{
    doctree::{self, Bale},
    error::{Error, Result as DocketResult},
    render,
};

/// Docket
///
/// Represents a documentation set. Responsible for opening the bales of
/// documentation and traversing them to render out to HTML.
#[derive(Debug)]
pub struct Docket {
    title: String,
    doctree_root: Bale,
}

impl Docket {
    /// Open the given `path` as a documentaiton collection
    ///
    /// Once opned a documentaiton collection has a title, and can be rendreed
    /// to a target path.
    pub fn open<P: AsRef<Path>>(path: P) -> DocketResult<Self> {
        if !path.as_ref().is_dir() {
            Err(Error::SourcePathNotADirectory(path.as_ref().into()))?;
        }
        Ok(Docket {
            title: title_from_path(path.as_ref())?,
            doctree_root: doctree::open(&path)?,
        })
    }

    /// Render to HTML
    ///
    /// Renders the documentation set. Creates a tree of HTML files into the
    /// given `target` directory.
    pub fn render<P: AsRef<Path>>(self, target: P) -> DocketResult<()> {
        trace!(
            "Rendering documentation for {} to {:?}",
            self.title,
            target.as_ref()
        );
        render::render(target, self.title, self.doctree_root)?;
        Ok(())
    }
}

/// Calculate the title of the documentation set from the given path.
fn title_from_path(path: &Path) -> std::result::Result<String, io::Error> {
    let title_file = path.join("title");
    Ok(if title_file.is_file() {
        fs::read_to_string(title_file)?
    } else {
        Path::canonicalize(path)?
            .components()
            .filter_map(|c| match c {
                path::Component::Normal(path) => path.to_owned().into_string().ok(),
                _ => None,
            })
            .filter(|s| s != "docs")
            .last()
            .unwrap_or_else(|| String::from("Documentation"))
    })
}

#[cfg(all(test, fixme))]
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
