# Changelog

This page lists the main headline changes that make up each release of `docket`.
It is intended as a useful reference to the development history of the project,
but is by no means an exhaustive list of all changes.

## 0.6.1

 * Improved tokenisation for the search index. Prevents some trivial tokens from
   ending up in the search index when they shouldn't have been.

## 0.6.0

 * Add javascript search. A json search index is built when the site is rendred.
   Javascript on the frontend uses progressive enhancement to inject a search
   box.
 * Documentation improvements.

## 0.5.0

 * Enable more of `pulldown-cmark`'s Markdown extensions.
 * Support for highlighting code using `syntect` _or_ `hl-js`.

## 0.4.1

 * Added a feature `watch` which uses file system notifications to watch for
   changes in the source folder and re-build the site when it is changed. This
   allows users to opt-out of the `notify` crate dependency if the feature isn't
   needed.
 * Added a feature `par_render` which uses Rayon to render pages in parallel.
   This allows users to opt-out of the Rayon dependency if the feature isn't
   needed.

## 0.3.0

 * Upgraded to Pulldown-cmark `0.4`. This version is a major change to the way
   Pulldown parses markdown and should provide some performance improvements on
   larger files.
 * Docket now has a working CI configuration!

## 0.2.5

 * Generated HTML has a `charset` meta tag. This should fix rendering
   of non-ASCII characters.
 
## 0.2.4

 * Parallel rendering of pages with Rayon.

## 0.2.3

 * Rust 2018 support.
 
## 0.2.1

 * Fixup to generated OTC to prevent an unnecessary redirect. Previously links
   were missing a trailing `/`. This causes a re-direct to the directory with
   the trailing `/` before the page is rendered.

## 0.2.0

 * File watcher support with the `-w`, and `--watch` flags (#2)

## 0.1.7

 * Sort pages by name. The prefix is droped. This allows adding `01-` and
   similar numeric prefixes to pages to change their order within the generated
   site.

## 0.1.5

 * Add a `--version` flag to print out the version of `docket`.
 * Improved rendering of code blocks
 * Stop rendering the style inline in each page. Reduces page bloat.

## 0.1.4

 * Support mobile browsers with the `viewport` `meta` tag.

## 0.1.3

 * Initial "usable" version.