use crate::page::PageInfo;
use failure::Error;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

/// Add terms from the given text to the search index
pub(crate) fn add_terms_to_index(text: &str, index: &mut HashMap<String, i32>) {
    for term in text
        .split(|c| c == '>' || c == '<' || char::is_whitespace(c) || char::is_ascii_punctuation(&c))
    {
        *index.entry(term.to_lowercase()).or_default() += 1;
    }
}

/// Write the built search indices to the output directory
pub(crate) fn write_search_indices<'a, I>(output_dir: &Path, pages: I) -> Result<(), Error>
where
    I: Iterator<Item = &'a PageInfo>,
{
    let search_index_path = output_dir.join("search_index.json");
    let mut index_file = File::create(&search_index_path)?;
    write!(index_file, "[")?;
    let mut first = true;
    for page in pages {
        if let Some(ref index) = page.search_index {
            if first {
                first = false;
            } else {
                write!(index_file, ",")?;
            }
            write!(
                index_file,
                r#"{{"slug":"{0}","title":"{1}","terms":{{"#,
                page.slug, page.title
            )?;
            let mut first_term = true;
            for (term, count) in index {
                if first_term {
                    first_term = false;
                } else {
                    write!(index_file, ",")?;
                }
                write!(index_file, r#""{0}":{1}"#, term, count)?;
            }
            write!(index_file, r#"}}}}"#)?;
        }
    }
    write!(index_file, "]")?;

    Ok(())
}
