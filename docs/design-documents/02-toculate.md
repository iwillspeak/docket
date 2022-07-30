# The Toc

Docket aims to understand documentation as a heriachy of nodes and leaves that
form a tree. The nodes in the tree are the bales The leaves in the tree are
the pages.

This high level heriachy will be navigable with the breadcrumbs, and with the
main navigation. Navigation should allow moving around within a bundle, and
navigating to sibling bundles, if any. Within a document however there is a
separate heirachy. This is the *Tree of Contents*, or TOC.


## Toculation

When a page is loaded we parse it with `pulldown`. This produces an iterator
of parser events. Pulldown can render these events into HTML. To extract a
TOC we want to pre-process this tree.

Pulldown produces events for a node starting, for content within a node, and for
a node ending. To build the TOC we want to trasnform a `Vec<Event<'a>>` into
a structure similar to the following:

```ml
type TocEntry<'a> =
    | Events of Event list
    | Node of HeadingLevel * Events list * TocEntry list
```

That is an item in the TOC is either a collection of events, or an interor
node. The interior nodes represent the headings. Heading nesting determines the
layout of this tree.

Once we have this sturcture we can then make sure TocEntry is also an
iterator of its events. We should also expose our own iterator API to walk the 
headings in the TOC rather than the full set of events from pulldown. This can
then be used to perform actions such as 'find the top level headings' which will
be needed to control the page names, as well asn rendering the "what's on this
page" section for in-page navigation.

## The `[TOC]` Command

The current Docket rendering supports a few additions to markdown:

 * Code blocks can be prefixed with ::lang to control highlighting
 * A paragraph of just `[TOC]` is replaced wit a rendered tree of the headings.

The first has probably run its course by now. We don't want to support some
janky old syntax for specifying the langauge of a block when fenced blocks can
be used instead. For the TOC however there's still a use. It's proposed we
keep that and potentially expand it a little. It might be nice to allow TOC to
be used to render trees at any level. That is a `[TOC]` at the root renders all
the nodes below it. I.e. a `[TOC]` beneath an H1 renders all the H2 within it.
We can then extend `[TOC]` with an argument for the heading _depth_ to render.

e.g.

```markdown
# One

## Two

[TOC 1]

## Three

[TOC 2]

### Four

#### Four point Five

#### Four going on Five

## Five

```

The `[TOC 1]` in `# Two` renders the `## Three` and `## Five` headings. The
second `[TOC 2]` call would then render `### Four`, `#### Four point Five`, and
`#### Four going on Five`.


This extneded form of `[TOC]` would add flexibility, and make the TOC useful to
show indexes for sub parts of a page. E.g. if a section is listing use cases the
`[TOC]` could be used at the beginning of the section to list just the usecase
headings. In this way headings and their children within a page are like
'lightweight pages' when it comes to TOC rendering.