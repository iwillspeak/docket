---
name: rust-asset-embedding
description: Working with Rust projects that embed static assets (CSS, JS, HTML, etc.) into the binary at compile time using include_str!(). Use when editing embedded assets appears to have no effect, when adding a new asset file to a Rust project, or when debugging why asset changes are not reflected in output.
---

# Rust Asset Embedding with `include_str!()`

## How it works

`include_str!("path/to/file")` reads a file at **compile time** and bakes its
contents into the binary as a `&'static str`. The file on disk is irrelevant at
runtime — only the compiled binary matters.

**Consequence**: editing an asset file (CSS, JS, SVG, etc.) has no effect until
the binary is rebuilt. This is the most common source of confusion.

## Editing an existing asset

1. Edit the file on disk (e.g. `assets/style.css`)
2. **Recompile**: `cargo build` or `cargo run`
3. The output directory will then contain the updated asset

Do not test asset changes without recompiling first.

## Adding a new asset

Three places must change together:

### 1. Create the asset file

```
assets/foo.js
```

### 2. Register it in the assets array

Find the `static ASSETS: [Asset; N]` declaration (commonly in
`src/render/layout/html.rs` or equivalent) and:

- Increment the array size literal `N` by 1
- Add an entry:

```rust
Asset::internal("foo.js", include_str!("../../../assets/foo.js")),
```

The path in `include_str!` is relative to the `.rs` source file, not the
project root.

### 3. Reference it in the template or code

Add whatever HTML/code reference is needed, e.g.:

```html
<script src="{root}foo.js" type=module></script>
```

## Common errors

**Array size mismatch**: Rust requires the size literal in `[Asset; N]` to match
the number of elements exactly. The compiler will catch this, but it's easy to
forget to update `N` when adding or removing an entry.

**Wrong relative path**: `include_str!` paths are relative to the source file
containing the macro call, not to `Cargo.toml` or the project root. Verify the
`../` depth matches the actual directory nesting.

**Stale output**: If asset changes seem absent after rebuilding, check that
`cargo run` is targeting the correct output directory. The binary writes asset
copies to the target on each run, overwriting previous versions.
