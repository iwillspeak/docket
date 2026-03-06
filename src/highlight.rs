//! Syntax Highlighting
//!
//! This module provides a trait for highlighinng code blocks and a pair of
//! implementations. The `SyntectHighligher` uses the Syntect crate to render
//! the code to highlighted HTML at build time. The `HighlightJsHighlighter`
//! uses the Highlight JS library at runtime to process codeblocks in the user's
//! browser.
//!
//!  If the `syntect-hl` feature is enabled then both highlighters will be
//!  available, and syntect perferred. HighlightJS is always avaiable.

use std::io::Write;

use log::debug;
use pulldown_cmark::{CodeBlockKind, Event, Tag};

pub(crate) trait Highlighter {
    /// # Highlight a Code Block
    ///
    /// Returns a list of the events to emit to the TOC to represent the block.
    fn hl_codeblock<'a>(&self, name: Option<&str>, block: &str) -> Vec<Event<'_>>;

    /// # Write any HTML header required for highlighting on this page.
    fn write_header(&self, out: &mut dyn Write) -> std::io::Result<()>;
}

pub use js_hl::HighlightJsHighlighter;
#[cfg(feature = "syntect-hl")]
pub use syntect_hl::SyntectHighlighter;

#[cfg(feature = "syntect-hl")]
mod syntect_hl {
    use std::io::Write;

    use once_cell::sync::OnceCell;
    use pulldown_cmark::Event;
    use syntect::{
        highlighting::ThemeSet,
        html::{css_for_theme_with_class_style, ClassStyle, ClassedHTMLGenerator},
        parsing::SyntaxSet,
        util::LinesWithEndings,
    };

    use super::{to_default_events, Highlighter};

    const LIGHT_THEME: &str = "InspiredGitHub";
    const DARK_THEME: &str = "base16-ocean.dark";

    fn class_style() -> ClassStyle {
        ClassStyle::SpacedPrefixed { prefix: "hl-" }
    }

    pub struct SyntectHighlighter {
        ss: SyntaxSet,
        ts: ThemeSet,
        header_cache: OnceCell<String>,
    }

    impl SyntectHighlighter {
        /// # Create a New Highlighter
        pub fn new() -> Self {
            SyntectHighlighter {
                ss: SyntaxSet::load_defaults_newlines(),
                ts: ThemeSet::load_defaults(),
                header_cache: OnceCell::new(),
            }
        }
    }

    impl Highlighter for SyntectHighlighter {
        fn hl_codeblock(&self, name: Option<&str>, block: &str) -> Vec<Event<'_>> {
            let syntax = name
                .and_then(|n| self.ss.find_syntax_by_token(n))
                .unwrap_or_else(|| self.ss.find_syntax_plain_text());

            let mut generator =
                ClassedHTMLGenerator::new_with_class_style(syntax, &self.ss, class_style());
            for line in LinesWithEndings::from(block) {
                if generator
                    .parse_html_for_line_which_includes_newline(line)
                    .is_err()
                {
                    return to_default_events(name, block);
                }
            }
            let inner = generator.finalize();
            let html = format!("<pre class=\"hl-code\"><code>{}</code></pre>\n", inner);
            vec![Event::Html(html.into())]
        }

        fn write_header(&self, out: &mut dyn Write) -> std::io::Result<()> {
            let header = self.header_cache.get_or_try_init(|| -> std::io::Result<String> {
                let io_err = |e: syntect::Error| {
                    std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                };
                let light_css = strip_background_color(
                    &css_for_theme_with_class_style(&self.ts.themes[LIGHT_THEME], class_style())
                        .map_err(io_err)?,
                );
                let dark_css = strip_background_color(
                    &css_for_theme_with_class_style(&self.ts.themes[DARK_THEME], class_style())
                        .map_err(io_err)?,
                );
                let dark_auto =
                    prefix_css_selectors(&dark_css, "html:not([data-color-mode])");
                let dark_prefixed =
                    prefix_css_selectors(&dark_css, "html[data-color-mode=\"dark\"]");
                Ok(format!(
                    "<style>\n{}\n@media (prefers-color-scheme: dark) {{\n{}}}\n{}</style>",
                    light_css, dark_auto, dark_prefixed
                ))
            })?;
            out.write_all(header.as_bytes())
        }
    }

    /// Strip `background-color` declarations from generated theme CSS.
    ///
    /// This lets the design's own `pre` rule control the container background
    /// consistently across light and dark modes.
    fn strip_background_color(css: &str) -> String {
        css.lines()
            .filter(|line| !line.trim_start().starts_with("background-color"))
            .map(|line| format!("{}\n", line))
            .collect()
    }

    /// Prefix each CSS selector in `css` with `prefix`.
    ///
    /// Used to scope a theme's CSS under a manual dark-mode override selector.
    fn prefix_css_selectors(css: &str, prefix: &str) -> String {
        let mut result = String::new();
        for chunk in css.split('}') {
            let trimmed = chunk.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(brace) = trimmed.find('{') {
                let selector = trimmed[..brace].trim();
                let props = trimmed[brace + 1..].trim();
                result.push_str(&format!("{} {} {{\n    {}\n}}\n", prefix, selector, props));
            }
        }
        result
    }
}

mod js_hl {
    use std::io::Write;

    use pulldown_cmark::Event;

    use super::{to_default_events, Highlighter};

    pub struct HighlightJsHighlighter;

    impl Highlighter for HighlightJsHighlighter {
        fn hl_codeblock(&self, name: Option<&str>, block: &str) -> Vec<Event<'_>> {
            to_default_events(name, &block)
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

fn to_default_events<'a, 'b>(name: Option<&'a str>, block: &'a str) -> Vec<Event<'b>> {
    let kind = match name {
        Some(name) => CodeBlockKind::Fenced(name.to_owned().into()),
        None => CodeBlockKind::Indented,
    };
    vec![
        Event::Start(Tag::CodeBlock(kind.clone())),
        Event::Text(block.to_owned().into()),
        Event::End(Tag::CodeBlock(kind)),
    ]
}

/// # Get the Active Highlighter
///
/// Returns a reference to a shared highlighter.
pub(crate) fn get_hilighter() -> &'static dyn Highlighter {
    static GLOBAL_JS_HL: HighlightJsHighlighter = HighlightJsHighlighter;

    #[cfg(feature = "syntect-hl")]
    if std::env::var("DOCKET_FORCE_JS_HL").is_err() {
        use once_cell::sync::Lazy;
        static GLOBAL_SYNTECT_HL: Lazy<SyntectHighlighter> =
            Lazy::new(|| SyntectHighlighter::new());
        debug!("Using syntect for highlighting.");
        return &*GLOBAL_SYNTECT_HL;
    }

    debug!("Using Javascript highlighter.");
    &GLOBAL_JS_HL
}
