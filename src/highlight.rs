use std::io::Write;

use log::debug;
use pulldown_cmark::{CodeBlockKind, Event, Tag};

pub(crate) trait Highlighter {
    /// # Highlight a Code Block
    ///
    /// Returns a list of the events to emit to the TOC to represent the block.
    fn hl_codeblock<'a>(&self, name: &str, block: &str) -> Vec<Event>;

    /// # Write any HTML header required for highlighting on this page.
    fn write_header(&self, out: &mut dyn Write) -> std::io::Result<()>;
}

pub use js_hl::HighlightJsHighlighter;
#[cfg(feature = "syntect-hl")]
pub use syntect_hl::SyntectHighlighter;

#[cfg(feature = "syntect-hl")]
mod syntect_hl {
    use std::io::Write;

    use log::debug;
    use pulldown_cmark::Event;
    use syntect::{
        self, highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet,
    };

    use super::{to_default_events, Highlighter};

    pub struct SyntectHighlighter {
        ss: SyntaxSet,
        ts: ThemeSet,
    }

    impl SyntectHighlighter {
        /// # Create a New Highlighter
        pub fn new() -> Self {
            let ss = SyntaxSet::load_defaults_newlines();
            let ts = ThemeSet::load_defaults();
            SyntectHighlighter { ss, ts }
        }
    }

    impl Highlighter for SyntectHighlighter {
        fn hl_codeblock(&self, name: &str, block: &str) -> Vec<Event> {
            let syntax = self
                .ss
                .find_syntax_by_extension(&name)
                .or_else(|| self.ss.find_syntax_by_name(&name))
                .unwrap_or_else(|| self.ss.find_syntax_plain_text());

            debug!("source name: {0}, syntax: {1:?}", name, syntax.name);

            let theme = &self.ts.themes["InspiredGitHub"];
            let highlighted = highlighted_html_for_string(&block, &self.ss, &syntax, theme);
            match highlighted {
                Ok(html) => vec![Event::Html(html.into())],
                Err(_) => to_default_events(&name, &block),
            }
        }
        fn write_header(&self, _out: &mut dyn Write) -> std::io::Result<()> {
            Ok(())
        }
    }
}

mod js_hl {
    use std::io::Write;

    use pulldown_cmark::Event;

    use super::{to_default_events, Highlighter};

    pub struct HighlightJsHighlighter;

    impl Highlighter for HighlightJsHighlighter {
        fn hl_codeblock(&self, name: &str, block: &str) -> Vec<Event> {
            to_default_events(&name, &block)
        }

        fn write_header(&self, out: &mut dyn Write) -> std::io::Result<()> {
            const HLJS_VERSION: &str = "10.5.0";
            write!(
                out,
                r#"
                <link rel="stylesheet"
                  href="//cdnjs.cloudflare.com/ajax/libs/highlight.js/{0}/styles/default.min.css">
                <script src="//cdnjs.cloudflare.com/ajax/libs/highlight.js/{0}/highlight.min.js"></script>
                <script>hljs.initHighlightingOnLoad();</script>"#,
                HLJS_VERSION
            )
        }
    }
}

fn to_default_events<'a, 'b>(name: &'a str, block: &'a str) -> Vec<Event<'b>> {
    let name = CodeBlockKind::Fenced(name.to_owned().into());
    vec![
        Event::Start(Tag::CodeBlock(name.clone())),
        Event::Text(block.to_owned().into()),
        Event::End(Tag::CodeBlock(name)),
    ]
}

#[cfg(feature = "syntect-hl")]
lazy_static! {
    static ref GLOBAL_SYNTECT_HL: SyntectHighlighter = SyntectHighlighter::new();
}

/// # Get the Active Highlighter
///
/// Returns a reference to a shared highlighter.
pub(crate) fn get_hilighter() -> &'static dyn Highlighter {
    static GLOBAL_JS_HL: HighlightJsHighlighter = HighlightJsHighlighter;
    #[cfg(feature = "syntect-hl")]
    if std::env::var("DOCKET_FORCE_JS_HL").is_err() {
        debug!("Using syntect for highlighting.");
        return &*GLOBAL_SYNTECT_HL;
    }
    debug!("Using Javascript highlighter.");
    &GLOBAL_JS_HL
}