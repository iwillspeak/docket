# New Layout Design

This design document aims to sum up my current thoughts and feelings about where I'd like this project to go. I feel that for medium sized projects Docket is starting to show the roughness around some edges. The plan is to allow docket to walk a tree of directories rather than just processing a single set of files from a single directory.

## Documentation Directory Layout

Currently Docket searches for markdown files in a single folder. Files are sorted asciibetically, and the prefix on the filenames is stripped to generate each page's slug. To extend this to directories we can treat them in the same way. A directory will be examined for markdown contents. If markdown files are found then we generate a slug and order it in the same way we would a page. When rendering we can then recursively walk the contents of the folder.

```
docs/
 + index.md
 + 01-wildlife.md
 + 02-plants/
 |  + trees.md
 |  + shrubs.md
 |  + index.md
 + 03-geology/
 |  + rocks.md
 |  + magma.md
```

### Pages and Bales

At each level of our tree there can be three kinds of item:

 * **Pages** - standard markdown documents such as `01-wildlife.md`
 * **Indexes** - chapter overviews and high-level introductions to the contents of a given 'bale'.
 * **Bales** - Directories containing one or more markdown files.

Here we group items together into "bales". Bales are collections of pages and inner bales. Bales are used as the unit of navigation. Given the example above there are three bales: The root bale, the `plants` bale, and the `geology` bale. 

## Page Layout and Navigation

Each rendered page contains three main items: content, TOC, and navigation. The content of the page is mainly the rendred markdown for the doucment, or the markdown of the bale's index. The TOC contains the overview of the current page. It should contain the H1 and H2 headings in the current document. The last item is the nagivation tree. This tree contains: The bales that are at the same level in the tree as the current page's bale, and the pages within the current document's bale.

Given the example above the navigation in `wildlife.md` would contain the Root bale and it's contents (wildlife, geology, plants). IN the plants bale we instead have the sibilings (trees, strhubs) as well as the bales that are siblings of the current one (Plants, geology).

**note**: I'm not sure yet if we also want the documents from the parent bale, as well as the bales themselves.

### Breadcrumbs

For deeply nested documents we should expose a breadcrumb trail to allow quick access to higher level topics.

## Search

 For a large amount of the tree it will not be reachable from any given page. The solution to this is search. We should include a search box on every page.

## Rendering

Rendering is currently split between the renderer, and the items that implement renderable. With bales in the mix it makes sense to split this out a little. The current rendering mixes in page structure with page location. Instead renderable should just be something with a single method that accepts a render context:

```rs
trait Renderable {
	fn render(&mut self, ctx: &RenderCtx) -> Result<RenderOutput, Error>;
}
```

Or something like that. Then we can have bundles be renderable. The render context will then expose methods to create files and write to them, as well as keeping state information about the current location and path to root.

Some of the current page layout is split between renderable and page. It would be better to introduce a new Layout type to abstract this.