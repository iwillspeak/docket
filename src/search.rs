use crate::page::PageInfo;
use failure::Error;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

/// A Page in the Search Index
///
/// This POD struct is used to serialise the search index. It is consumed by
/// the `search.js` file as `search_index.json`.
#[derive(Serialize)]
struct SearchIndexEntry<'a> {
    /// The page's title.
    pub title: &'a str,
    /// The URL `slug` to use when linking to the page.
    pub slug: &'a str,
    /// The search term weights for this page.
    pub terms: &'a HashMap<String, i32>,
}

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
    let index_file = File::create(&search_index_path)?;
    let index: Vec<_> = pages
        .flat_map(|page| {
            page.search_index.as_ref().map(|index| SearchIndexEntry {
                title: &page.title,
                slug: &page.slug,
                terms: index,
            })
        })
        .collect();
    serde_json::to_writer(index_file, &index)?;

    Ok(())
}
