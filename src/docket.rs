use std::path::{Path, PathBuf, Component};
use std::io;
use std::io::prelude::*;
use std::fs::{File, create_dir_all};
use pulldown_cmark::{Parser, html};
use page::{Page, PageInfo};
use util::read_file_to_string;

/// Docket
///
/// Represents a documentation set. Repsonsible for finding files on
/// disk which are part of a given set of documentation and rendering
/// them out to HTML files.
#[derive(Debug)]
pub struct Docket {
    /// The documentation title, based on the directory name
    title: String,

    /// The index, if there is one
    index: Option<PathBuf>,

    /// The page footer, if there is one
    footer: Option<PathBuf>,

    /// The pages, in order
    pages: Vec<PathBuf>,
}

/// Checks if this is the expected file
fn is_file(path: &Path, name: &str) -> bool {
    path.file_stem().map(|p| p == name).unwrap_or(false)
}

impl Docket {
    /// Create a Docket Instance
    ///
    /// ## Arguments
    ///
    /// The path to search for documentation. Markdown files in this
    /// directory will be used for documentation.
    pub fn new(doc_path: &Path) -> io::Result<Self> {
        trace!("Searching for docs in {:?}", doc_path);
        if !doc_path.is_dir() {
            // TODO: Return Result<> instead?
            panic!("Not a directory");
        }

        let title = Path::canonicalize(doc_path)
            .unwrap()
            .components()
            .filter_map(|c| match c {
                Component::Normal(path) => path.to_owned().into_string().ok(),
                _ => None,
            })
            .filter(|s| s != "docs")
            .last()
            .unwrap();

        let mut index = None;
        let mut footer = None;
        let mut pages = Vec::new();

        for entry in doc_path.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let extension = ext.to_string_lossy();
                match extension.as_ref() {
                    "md" | "markdown" | "mdown" => {
                        match path {
                            ref p if is_file(&p, "footer") => footer = Some(p.to_owned()),
                            ref p if is_file(&p, "index") => index = Some(p.to_owned()),
                            _ => pages.push(path.clone()),
                        }
                    }
                    _ => (),
                }
            }
        }
        Ok(Docket {
            title: title.to_string(),
            index: index,
            footer: footer,
            pages: pages,
        })
    }

    /// Render to Html
    ///
    /// Creates a tree of HTML files in the given `output` directory.
    pub fn render(self, output: &Path) {
        trace!("Rendering docs to {:?} ({:?})", output, self);
        let footer = self.rendered_footer();

        let mut rendered_pages = Vec::new();
        create_dir_all(&output).unwrap();
        info!("Created outpud dir: {:?}", output);
        for path in self.pages.iter() {
            let output_name = path.file_stem().unwrap();
            let output_path = output.join(output_name).with_extension("html");
            info!("Rendering {:?} to {:?}", output_name, output_path);
            rendered_pages.push(Page::new(&path).render_with_footer(&footer, &output_path));
        }

        self.render_index(rendered_pages, output);
    }

    /// Render the index page
    fn render_index<P>(&self, pages: P, output: &Path)
    where
        P: IntoIterator<Item = PageInfo>,
    {
        let mut file = File::create(output.join("index.html")).unwrap();
        write!(
            file,
            "<head><title>{0}</title></head><body><h1>{0}</h1>",
            self.title
        ).unwrap();

        write!(file, "<ul>").unwrap();
        for page in pages {
            write!(file, "<li><h2><a>{}</a></h2></li>", page.title).unwrap();
        }
    }

    /// Render the Footer to a String
    fn rendered_footer(&self) -> String {
        let mut footer = String::new();
        if let Some(ref footer_path) = self.footer {
            let contents = read_file_to_string(&footer_path);
            let parsed = Parser::new(&contents);
            html::push_html(&mut footer, parsed);
        }
        footer
    }
}
