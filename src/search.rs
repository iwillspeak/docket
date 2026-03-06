//! Search Index Builder
//!
//! This module defines a builder API to produce term frequency indexes. These
//! indexes are serialisable. Once built search indexes are written out to the
//! root of the site and loaded by javascript to provide an interactive search.
//!
//! Terms are normalised before indexing: stopwords are removed, very short
//! words are dropped, and the remaining words are reduced to their stem using
//! the Snowball English (Porter 2) algorithm. The final JSON uses TF-IDF
//! weights so that words common across many pages are down-weighted relative
//! to words that are distinctive to a few pages.

use rust_stemmers::{Algorithm, Stemmer};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;

/// Common English stopwords to exclude from the search index.
///
/// This list must remain sorted so that `binary_search` works correctly.
static STOPWORDS: &[&str] = &[
    "a", "about", "an", "and", "are", "as", "at", "be", "been", "being",
    "but", "by", "can", "could", "did", "do", "does", "for", "from", "had",
    "has", "have", "he", "her", "him", "his", "i", "if", "in", "into", "is",
    "it", "its", "may", "might", "my", "no", "not", "of", "on", "or", "our",
    "out", "shall", "she", "should", "so", "than", "that", "the", "their",
    "them", "then", "these", "they", "this", "those", "to", "up", "us", "was",
    "we", "were", "which", "who", "will", "with", "would", "you", "your",
];

/// Returns `true` when `word` (already lower-cased) is a stopword.
fn is_stopword(word: &str) -> bool {
    STOPWORDS.binary_search(&word).is_ok()
}

/// Builder struct for search indices
///
/// Keeps track of term counts as they are added. Once all terms have
/// been observed the builder can be transformed into a `TermFrequenciesIndex`.
pub(crate) struct TermFrequenciesBuilder {
    term_count: u32,
    terms: HashMap<String, u32>,
    stemmer: Stemmer,
}

impl Default for TermFrequenciesBuilder {
    fn default() -> Self {
        Self {
            term_count: 0,
            terms: HashMap::new(),
            stemmer: Stemmer::create(Algorithm::English),
        }
    }
}

impl TermFrequenciesBuilder {
    pub fn add_terms(&mut self, text: &str) -> &mut Self {
        for term in text.split(|c| {
            c == '>' || c == '<' || char::is_whitespace(c) || char::is_ascii_punctuation(&c)
        }) {
            let term = term.trim().to_lowercase();
            // Skip empty tokens, very short words, and stopwords.
            if term.len() < 3 || is_stopword(&term) {
                continue;
            }
            let stemmed = self.stemmer.stem(&term).into_owned();
            self.term_count += 1;
            *self.terms.entry(stemmed).or_default() += 1;
        }
        self
    }

    /// Finalise the Search Index
    ///
    /// Convert the term counts into a term frequencies index.
    pub fn finalise(self) -> TermFrequenciesIndex {
        let total: f64 = self.term_count.into();
        let terms = self.terms;
        TermFrequenciesIndex(if total > 0.0 {
            terms
                .into_iter()
                .map(|(term, count)| (term, f64::from(count) / total))
                .collect()
        } else {
            HashMap::new()
        })
    }
}

#[derive(Debug)]
pub struct TermFrequenciesIndex(HashMap<String, f64>);

impl TermFrequenciesIndex {
    /// Unpack the inner frequenceis map from this index type
    #[allow(dead_code)]
    pub fn into_raw(self) -> HashMap<String, f64> {
        self.0
    }

    /// Obtain a reference to the frequencies map for this index
    #[allow(dead_code)]
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

/// Searchable Document Trait
///
/// This trait abstracts over elements that can appear within a search index.
pub trait SearchableDocument {
    /// Document Title
    fn title(&self) -> &str;

    /// Document Slug
    fn slug(&self) -> &str;

    /// Search index for this document, if any
    fn search_index(&self) -> Option<&TermFrequenciesIndex>;
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
    /// The TF-IDF term weights for this page.
    pub terms: HashMap<String, f64>,
}

/// Write the built search indices to the output directory
///
/// TF scores from each page's index are combined with corpus-wide IDF values
/// to produce TF-IDF weights. Terms that appear in every indexed page receive
/// an IDF of zero and are omitted from the output to keep the index compact.
pub(crate) fn write_search_indices<'a, I, D>(output_dir: &Path, pages: I) -> Result<(), io::Error>
where
    I: Iterator<Item = &'a D>,
    D: SearchableDocument + 'a,
{
    let pages: Vec<&D> = pages.collect();
    let num_docs = pages.len();

    // Count how many documents each term appears in (document frequency).
    let mut doc_freq: HashMap<&str, usize> = HashMap::new();
    for page in &pages {
        if let Some(index) = page.search_index() {
            for term in index.iter_terms() {
                *doc_freq.entry(term.as_str()).or_default() += 1;
            }
        }
    }

    let search_index_path = output_dir.join("search_index.json");
    let index_file = File::create(&search_index_path)?;

    // Pre-compute IDF for every term so the value is only calculated once
    // regardless of how many pages contain the term.
    // IDF = ln((1 + N) / (1 + df)).  Evaluates to ≤ 0 when df == N (i.e. the
    // term is present in every page), and those terms are omitted from the
    // output because they carry no discriminating power.
    let idf: HashMap<&str, f64> = doc_freq
        .iter()
        .filter_map(|(&term, &df)| {
            let v = ((1 + num_docs) as f64 / (1 + df) as f64).ln();
            if v > 0.0 { Some((term, v)) } else { None }
        })
        .collect();

    // Build TF-IDF weighted entries.
    let index: Vec<_> = pages
        .iter()
        .flat_map(|page| {
            page.search_index().map(|tf_index| {
                let terms = tf_index
                    .iter_frequencies()
                    .filter_map(|(term, tf)| {
                        idf.get(term.as_str()).map(|&term_idf| (term.clone(), tf * term_idf))
                    })
                    .collect();
                SearchIndexEntry {
                    title: page.title(),
                    slug: page.slug(),
                    terms,
                }
            })
        })
        .collect();

    serde_json::to_writer(index_file, &index)?;

    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::TermFrequenciesBuilder;

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
        // "a" is a stopword and will be removed; "test" and "string" survive.
        builder.add_terms("a test a string");
        let index = builder.finalise();

        assert_eq!(2, index.iter_terms().count());
        let mut terms: Vec<_> = index.iter_terms().cloned().collect();
        let raw = index.into_raw();
        terms.sort();
        // Both stems happen to equal the original words for these inputs.
        assert_eq!(vec!["string", "test"], terms);
        // "test" and "string" appear once each, so equal frequency.
        assert_eq!(raw.get("test"), raw.get("string"));
    }

    #[test]
    pub fn stemming_merges_word_forms() {
        let mut builder = TermFrequenciesBuilder::default();
        builder.add_terms("compile compiler compilers");
        let index = builder.finalise();

        // All three forms should collapse to the same stem.
        assert_eq!(1, index.iter_terms().count());
        let stem = index.iter_terms().next().unwrap().clone();
        assert_eq!("compil", stem);
    }

    #[test]
    pub fn stopwords_are_excluded() {
        let mut builder = TermFrequenciesBuilder::default();
        builder.add_terms("the quick brown fox");
        let index = builder.finalise();

        // "the" is a stopword; "quick", "brown", "fox" survive.
        assert_eq!(3, index.iter_terms().count());
        assert!(!index.as_raw().contains_key("the"));
    }

    #[test]
    pub fn short_words_are_excluded() {
        let mut builder = TermFrequenciesBuilder::default();
        builder.add_terms("go do it now run");
        let index = builder.finalise();

        // "go" and "it" are 2 characters (too short); "do" and "it" are also
        // stopwords.  "now" and "run" are 3 characters and are not stopwords,
        // so they are indexed.  All indexed terms must be ≥ 3 characters long.
        for term in index.iter_terms() {
            assert!(term.len() >= 3, "short term {:?} should not be indexed", term);
        }
    }
}

