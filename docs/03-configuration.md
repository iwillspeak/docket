# Configuration

Docket aims to replicate the features of `d`. Although this is built around limitation of configuration there are still a few settings that can be tweaked.

[TOC]

## Documentation Files

Each file in the source directory with the extension of `md`, `mdown` or `markdown` is rendered as a page.

## Page Uris

The Uri of a page is created from the file name by stripping leading numbers and replacing non-uri-safe characters with `-`. This allows leading numbers to be added to input file names to control the order in which pages appear in the main index.

## Page Titles

Pages should start with a level-1 markdown heading. This heading will be used as the page title. Pages without such a heading are named their file names.

## Index

To add content to the index page create a file called `index.md`. This content is rendered before the table of contents on the main page. The heading on the index page is the main documentation title.

## Documentation Title

The title of the documentation is the first folder in the path which isn't one of `docs` or `documentation`. To override this create a file called `title` with the title in it.

## Footer

The contents of `footer.md` will be added to the base of every page.
