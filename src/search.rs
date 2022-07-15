use crate::page::PageInfo;

use crate::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

/// Builder struct for search indices
///
/// Keeps track of term counts as they are added. Once all terms have
/// been observed the builder can be transformed into a `TermFrequenciesIndex`.
#[derive(Default, Debug)]
pub(crate) struct TermFrequenciesBuilder {
    term_count: u32,
    terms: HashMap<String, u32>,
}

impl TermFrequenciesBuilder {
    pub fn add_terms(&mut self, text: &str) -> &mut Self {
        for term in text.split(|c| {
            c == '>' || c == '<' || char::is_whitespace(c) || char::is_ascii_punctuation(&c)
        }) {
            let term = term.trim();
            if !term.is_empty() {
                self.term_count += 1;
                *self.terms.entry(term.to_lowercase()).or_default() += 1;
            }
        }
        self
    }

    /// Finalise the Search Index
    ///
    /// Convert the term counts into a term frequencies index.
    pub fn finalise(self) -> TermFrequenciesIndex {
        let total: f64 = self.term_count.into();
        let terms = self.terms;
        TermFrequenciesIndex(
            terms
                .into_iter()
                .map(|(term, count)| (term, f64::from(count) / total))
                .collect(),
        )
    }
}

pub struct TermFrequenciesIndex(HashMap<String, f64>);

impl TermFrequenciesIndex {
    /// Unpack the inner frequenceis map from this index type
    #[allow(dead_code)]
    pub fn into_raw(self) -> HashMap<String, f64> {
        self.0
    }

    /// Obtain a reference to the frequencies map for this index
    pub fn as_raw(&self) -> &HashMap<String, f64> {
        &self.0
    }

    /// Iterate over the terms in this index
    #[allow(dead_code)]
    pub fn iter_terms(&self) -> impl Iterator<Item = &String> {
        self.0.keys()
    }

    /// Iterate over the term frequencies in this index
    #[allow(dead_code)]
    pub fn iter_frequencies(&self) -> impl Iterator<Item = (&String, &f64)> {
        self.0.iter()
    }
}

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
    pub terms: &'a HashMap<String, f64>,
}

/// Write the built search indices to the output directory
pub(crate) fn write_search_indices<'a, I>(output_dir: &Path, pages: I) -> Result<()>
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
                terms: index.as_raw(),
            })
        })
        .collect();
    serde_json::to_writer(index_file, &index)?;

    Ok(())
}

#[cfg(test)]
pub mod test {
    use crate::search::TermFrequenciesBuilder;

    #[test]
    pub fn empty_search_indx() {
        let builder = TermFrequenciesBuilder::default();
        let index = builder.finalise();

        assert_eq!(0, index.as_raw().len());
        assert_eq!(0, index.iter_terms().count());
        assert_eq!(0, index.iter_terms().count());
        assert_eq!(0, index.into_raw().len());
    }

    #[test]
    pub fn index_with_terms() {
        let mut builder = TermFrequenciesBuilder::default();
        builder.add_terms("a test a string");
        let index = builder.finalise();

        assert_eq!(3, index.iter_terms().count());
        let mut terms: Vec<_> = index.iter_terms().cloned().collect();
        let index = index.into_raw();
        terms.sort();
        assert_eq!(vec!["a", "string", "test"], terms);
        assert!(index.get("a").unwrap() > index.get("test").unwrap());
    }
}
