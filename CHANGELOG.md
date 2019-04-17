# 0.3.0

 * Upgraded to Pulldown-cmark `0.4`. This version is a major change to
   the way Pulldown parses markdown and should provide some
   performance improvements on larger files.
 * Docket now has a working CI configuration!

# 0.2.5

 * Generated HTML has a `charset` meta tag. This should fix rendering
   of non-ASCII characters.
 
# 0.2.4

 * Parallel rendering of pages with Rayon.

# 0.2.3

 * Rust 2018 support.
 
# 0.2.1

 * Fixup to generated OTC to prevent an unnecessary
   redirect. Previously links were missing a trailing `/`. This causes
   a re-direct to the directory with the trailing `/` before the page
   is rendered.

# 0.2.0

 * File watcher support with the `-w`, and `--watch` flags (#2)
