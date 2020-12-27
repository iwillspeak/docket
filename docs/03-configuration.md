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

## Markdown Features

Markdown rendering is based on the [CommonMark](http://commonmark.org) spec and rendered by [`Pulldown`](https://crates.io/crates/pulldown-cmark). There are, however, a few extensions.

### Tree of Contents

When placed on its own in a paragraph `[TOC]` is replaced with a rendered tree based on the headings within the current page.

### Highlighting

Code blocks are highlighted with [Syntect](https://crates.io/crates/syntect) or
[`highlight.js`](https://highlightjs.org). Syntect is the default. By default
the type of each block is inferred automatically. Fenced code blocks can be used
to add a hint about the type of code:

    :::nohighlight
    ```py
    def foo():
        pass
    ```

Which renders as:

```py
def foo():
    pass
```

In addition the first line of the code block can specify a type hint if it begins with `:::`:

```nohighlight
    :::c
    #include <stdio.h>
        
    void main() {
        printf("hello world!");
    }
```

Which renders as:

    :::c
    #include <stdio.h>
        
    void main() {
        printf("hello world!");
    }

### Tables

Tables are supported. Markdown tables are defined with `|` and `---` using the
GFM tables extension.

```markdown
Hello | Wolrd
------| -----
TESt | thing
123 | `123`
more | *stuff*
test | sfasf
```

Which renders as:

Hello | Wolrd
------| -----
TESt | thing
123 | `123`
more | *stuff*
test | sfasf

### Task Lists

Task lists are supported with the syntax from GFM.

```
 * [ ] Milk
 * [x] Eggs
 * [x] Flour
```

Which renders as:

 * [ ] Milk
 * [x] Eggs
 * [x] Flour
