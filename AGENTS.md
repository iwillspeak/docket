# docket — Agent Notes

A Rust static-site generator: Markdown in, styled HTML out.
Binary defaults: `-s . -t build/`. Working docs target is `output/` (gitignored).

## Build and run

```bash
# Build and regenerate docs (use every time assets/ or src/ changes)
cargo run -- -s docs/ -t output/

# Watch mode (requires `watch` feature, on by default)
cargo run -- -s docs/ -t output/ --watch

# Just compile
cargo build
```

`output/` is gitignored and safe to trash. Avoid using other directories for
iteration.

## Architecture in 60 seconds

```
src/
  main.rs           Entry point; parses args, calls build() or watch_and_build()
  docket.rs         Docket struct — open a source dir, render to target dir
  doctree.rs        Bale (directory) and Page (markdown file) tree types
  toc.rs            Toc — parses pulldown-cmark events into a heading tree;
                    also drives the in-page [TOC] macro
  render.rs         RenderContext / RenderState — walks the doctree
  render/
    layout.rs       Layout trait
    layout/html.rs  HtmlLayout — THE template; all HTML lives here
  highlight.rs      SyntectHighlighter (compile-time, default) or
                    HighlightJsHighlighter (runtime, fallback)
  asset.rs          Asset enum: Internal (embedded string) or Disk (file copy)
  search.rs         Term-frequency index builder → search_index.json
  args.rs           CLI via docopt
  utils.rs          slugify_path, path_or_default
assets/
  style.css         All CSS — embedded at compile time
  dark.js           Dark/light mode toggle
  search.js         Client-side search using search_index.json
  nav.js            Mobile drawer toggle (sidebar + TOC)
  feather-sprite.svg SVG icon sprite
```

**Bale** = a directory of docs (interior node). Has optional `index.md` (frontispiece) and `footer.md`.
**Page** = a single Markdown file (leaf node). Slug derived from filename.
**Site title** = last path component of source dir, with "docs" filtered out; or contents of a `title` file in the source dir.

**Auto-index**: if a bale has no `index.md` (and is non-empty), `render.rs` calls `generate_auto_index` to synthesise a `Page` (via `Page::synthetic`) containing a bullet list of links to the bale's children. This is rendered as the bale's `index.html` so nav links always have a landing page.

## Assets are embedded at compile time

`include_str!()` is used in `src/render/layout/html.rs`. This means:

- **Editing `assets/style.css` or any `.js` file requires `cargo build` / `cargo run` to take effect** — file edits alone do nothing to the output.
- `docs-build/style.css` and `output/style.css` are copies written by the renderer, not symlinks.

### Adding a new asset (e.g. `foo.js`)

Two places must change together:

1. Create `assets/foo.js`
2. In `src/render/layout/html.rs`:
   - Increment the `ASSETS` array size literal: `static ASSETS: [Asset; N]`
   - Add `Asset::internal("foo.js", include_str!("../../../assets/foo.js")),`
   - Add `<script src="{root}foo.js" type=module></script>` to the HTML template string

## The HTML template

Everything is in one big `write!(..., r##"..."##)` in `HtmlLayout::render()` in
`src/render/layout/html.rs`. Template variables use Rust's `{name}` format syntax.

Key variables available in the template:
- `{site_name}`, `{page_title}`, `{bale_title}` — titles at different levels
- `{root}` — relative path from current page back to site root (e.g. `../`)
- `{breadcrumbs}` — rendered `<ol>` of the path hierarchy
- `{navs}` — rendered `<ul>` of sibling pages in current bale
- `{nav_prefix}` — relative path to current bale root
- `{toc}` — rendered in-page heading tree
- `{content}` — full rendered HTML of the page
- `{footer}` — footer HTML (from `footer.md` or default)

## Syntax highlighting

The default highlighter is **Syntect** with the `"InspiredGitHub"` theme — a **light** theme hardcoded at `src/highlight.rs:63`. It emits `style="background-color:#ffffff"` as inline styles on every `<pre>`, which breaks dark mode.

Current CSS workaround in `style.css`: `pre { background-color: var(--col-bg-dimmed) !important }` plus dark-mode span overrides.

A proper fix would be to pick a theme based on `prefers-color-scheme` (not currently possible since highlighting is done at build time, not runtime) or to switch the default Syntect theme to something dark and accept light mode looking different.

**Escape hatch**: set `DOCKET_FORCE_JS_HL=1` to skip Syntect entirely and use
highlight.js at runtime instead — avoids all inline style issues.

## CSS breakpoint structure

Three tiers, all in `assets/style.css`:

| Range | Grid | Sidebar | TOC |
|---|---|---|---|
| `< 800px` | 1 column | drawer (☰ button) | drawer (≡ button) |
| `800px – 979px` | 2 columns: `content toc` | drawer (☰ button) | in-grid sticky |
| `≥ 980px` | 3 columns: `sidebar content toc` | in-grid sticky | in-grid sticky |

### CSS cascade trap (hit twice during development)

Global rules declared **after** a `@media` block override the media query's rules
when specificity is equal, regardless of the media condition. Classic symptom:
`display: none` inside a media query does nothing because a later global rule
sets `display: flex` on the same selector.

Two patterns used here to fix it:
1. **Invert the default**: declare `display: none` globally, then `display: flex`
   inside a `@media (max-width: ...)` block. Used for `.nav-toggle` / `.toc-toggle`.
2. **Raise specificity in the media query**: use a more-specific selector like
   `.doc-grid .sidebar .drawer-close` inside the media query to beat the global
   `.drawer-close` rule. Used for `.drawer-close`.

## TOC rendering rules (as of 2026-03-07)

`render_toc_to` in `src/render/layout/html.rs` applies these rules when building
the navigation trees:

- **Single H1**: sidebar and the `[TOC]` macro both skip the H1 and start at H2,
  so the page title is not repeated.
- **Multiple H1s**: every H1 is shown along with its direct H2 children.
- **Sidebar depth**: renders headings down to **H4** (`HeadingLevel::H4`).
- **Inline `[TOC]`**: wrapped in `<nav class='inline-toc'>` with a
  `<p class='toc-section-label'>In this section</p>` label; also starts at H2 when
  there is a single H1.

The fix for the previous borrow-checker bug: `Nodes` is a consuming iterator, so
calling `.count()` then `.next()` is a use-after-move. Solution: collect into a
`Vec` first, check `len()`, then call `into_iter()`. `render_toc_node_to` was also
made generic (`impl Iterator<Item = &'a TocNode>`) so it accepts both `Nodes` and
`vec::IntoIter`.

## Test / preview pages

`docs/examples/` is **gitignored** and used as a scratch area for test pages.
Do not commit it, but feel free to create or modify files there freely.

To add a new test scenario:

1. Create `docs/examples/<NN>-<name>.md` with the heading structure you want to
   exercise.
2. Optionally place a `[TOC]` macro in the body to test inline TOC rendering.
3. Run `cargo run -- -s docs/ -t output/` to rebuild.
4. Browse `output/examples/<name>/index.html` to inspect the result.

### Current test pages

| File | What it tests |
|---|---|
| `01-single-h1-deep.md` | Single H1 → sidebar starts at H2; `[TOC]` shows "In this section"; H4 sidebar depth |
| `02-multi-h1.md` | Three H1 roots → main TOC lists all H1s + H2 children |
| `03-api-reference.md` | Single H1 with H2→H3→H4 nesting; sidebar depth stress test |
| `sub-section/` | Index-free sub-bale → auto-index generation |

