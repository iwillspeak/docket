# The `doctree`

A documentation set is a tree. The interior nodes in this tree are what we call
`Bale`s. The leaf nodes are `Page`s. All together this is the `doctree`.

A single node in the doctree should represent the _un-traversed_ state of that
node. Each unopend bale can contain any number of child items, along with it's
current index. If no index found on disk we should fabricate one from the folder
name. Later layouts _may_ choose to render this with a grid of child pages or
similar.

## Navigation

Each point in the tree needs to be targetable with navigation. To create a link
we need to know two things: the `slug` for the item, and the item's `title`.

```rs
pub struct NavItem {
    pub title: String,
    pub slug: Slug,
}
```

