# Syntax Highlighing

Currently when docet walks the tree to produce a TOC blocks of code are
highlighed. We support two highling modes: js, and Syntect. Syntect is the
preferred mode. We should keep it that way. JS highlight may be a good fallback
however if Syntect can't be built on a given environment for some reason.

## Sepration of Highlight and TOC

Rather than have highlighting intertwined with TOC generation instead it would
be nice if TOC was solely responsible for the heirachical transformation of the
document, and the highlihging was done in a separate pass. If both TOC and
highlight produce and consume iterators of events it should be possible to
acchive this separation.

## Filtering the Events

From the pulldown event stream we need to listen for block starts. If a block
is stated we initiate a highligter for that syntax. If none can be found we
fall back to some default highligh mode. For each line in the codeblock we can
then render it out with the highlither. These events will be transformed and
passed on to the consumer as pulldown HTML events.

