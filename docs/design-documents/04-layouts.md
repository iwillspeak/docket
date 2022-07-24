# Layouts and Rendering

Currently we walk the tree of bales and render out the indexes and pages within
them. Moving more of the knowledge of the tree structure into the render contex
will hopefully simplify some of the job of rendering.

[TOC]

## Legacy

In the old approach we had a seprataion between the `Renderer`, and the
`Renderable`. This abstracted over the index and standar pages. In our new
apprach pages are just pages. We still have some disconect betwen indexes and
standard pages however, see `nav_prefix`.

## Proposed Structure

When we walk a tree we initially create a root render context. This context
contains just the title of documentaiton. We can also add more "global" state
later, such as the global assets, and the highlighter.

When we render an item in a render context we then do one of two things:

 * If we are rendering a plain page we create a `PageCtx` containing page
   specific information.
 * If we are rendering a bale we create a nested child `RenderCtx`. We can then
   create some special `PageCtx` to represent the index and render the index
   with that.

Render contexts should contain information abot the current bale, as well. This
allows navigation information to be quieried from the render context. This might
be either an Enum burried within the render cotnex, or a pair of types:
`RenderState`, and `RenderCtx`. In this approach the render context is the root
immutable information that should be used for all rendering, such as site title,
highlighter etc. The render state is then the bale-speciifc information. A
render state is what is then used for page layout and rendering.

### The Layout Abstraction

Pages shoudl implement some trait that allows them to be quieried for items to
be used in the final rendered page. They shouldl not however know about the
final stucture of the page. Pages really only need to expose some `<article>`
which represents the page's content, and some `<ul>` that represents the TOC
tree.

With the page's responsibilities tied down this way it should allow us to
abstract away actual layout to some _third_ item. I'm calling this the Layout.
When a render happens we'll need to provide some layout. This will probably live
on the render ctx. Layouts are given access to a file that's open for write, the
current render state, and the renderable. A layout's responsibility is solely
that of putting HTML bits and bytes onto a page.

## Libraries

Seprating rendering, layout, and renderables like this allows _some_ portion of
`docket` to be moved into a library. A client would then be able to create a
`docket` from any directory, make a `render_ctx`, and render to a target
location. A builder API for render conntexts along with making the
`Highlighter`, and `Layout` traits public would allow a user to inject a custom
layout.

## Printing of Things

The renderable trait that is passed to a `Layout` should expose the toc, title,
and content of the document. We can expose these as opaque items which implement
the `Display` trait. That way we can format directly into whatever target buffer
is being written to.

E.g.:

```rust
trait Renderable {
  fn toc(&'a self, ctx: &'b RenderCtx) -> Toc<'a, 'b>;
  fn content(&'a self, ctx: &'b RenderCtx) -> Content<'a, 'b>;
}

type Content<'a> {
  toc: &'a TocTree;
  ctx: &'b RenderCtx;
}

impl<'a> Display for Content<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      // Create a highlighter iterator using the current highlighter, consuming
      // events from our main TOC. Pull those events down into Pulldown and have
      // it render directly to the output stream.
      html::write_html(Highlighted(self.ctx.highlighter, self.toc.iter()))
        .map_err(|_| fmt::Error)
    }
}
```