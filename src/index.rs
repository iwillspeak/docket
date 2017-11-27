use std::io::{self, Write};
use std::borrow::Cow;
use std::path::PathBuf;
use renderable::Renderable;
use page::PageInfo;
use pulldown_cmark::{Parser, html};
use util::read_file_to_string;

pub struct Index {
    /// The project title
    title: String,

    /// The path to the markdown source
    path: Option<PathBuf>,

    /// Pages to include in this index.
    pages: Vec<PageInfo>,
}

impl Renderable for Index {
    fn get_slug(&self) -> String {
        "".into()
    }

    fn get_title<'t>(&'t self) -> Cow<'t, str> {
        (&self.title[..]).into()
    }

    fn write_body<T: Write>(&self, file: &mut T) -> io::Result<()> {

        write!(file, r#"<h1><a href="">{}</a></h1>"#, self.title)?;

        if let Some(ref index_md) = self.path {
            debug!("found index file, rendering");
            let contents = read_file_to_string(&index_md);
            let parsed = Parser::new(&contents);
            let mut rendered = String::new();
            html::push_html(&mut rendered, parsed);
            write!(file, "{}", rendered)?;
        }

        // List of pages in the index
        debug!("listing pages in index");
        write!(file, r#"<ul class="index-toc">"#)?;
        for page in self.pages.iter() {
            write!(file, r#"<li><h2><a href="{}">{}</a></h2></li>"#, page.slug, page.title)?;
        }
        write!(file, "</ul>")?;
        
        Ok(())
    }
}

impl Index {
    pub fn new(title: String, md_source: Option<PathBuf>, pages: Vec<PageInfo>) -> Self {
        Index {
            title: title,
            path: md_source,
            pages: pages,
        }
    }
}
