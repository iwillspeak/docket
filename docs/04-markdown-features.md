# Markdown Features

Markdown rendering is based on the [CommonMark](http://commonmark.org) spec and rendered by [`Pulldown`](https://crates.io/crates/pulldown-cmark). There are, however, a few extensions.

## Tree of Contents

When placed on its own in a paragraph `[TOC]` is replaced with a rendered tree based on the headings within the current page.

## Highlighting

Code blocks are highlighted with [Syntect](https://crates.io/crates/syntect) or
[`highlight.js`](https://highlightjs.org). Syntect is the default. Syntect can
be disabled at compile / install time by not including the `syntect-hl` feature,
or at documentation build time by setting the `DOCKET_FORCE_JS_HL` environment
variable. 

By default the type of each block is inferred automatically. Fenced code blocks
can be used to add a hint about the type of code:

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

## Tables

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

## Task Lists

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
