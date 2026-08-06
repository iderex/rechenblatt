# 0006 Rendering

The pipeline from the workbook model to a page is written in this workspace, on
top of third-party libraries for shaping, rasterisation and vector output, and
none of those libraries is allowed to decide what a document means.

Status: accepted
Date: 2026-08-06
Issue: #34

## Context

There are three ways to get a spreadsheet rendered and only one of them is this
project. The choice is made here because every issue in this milestone is
sequenced from it, and because two of the three would settle the licence question
in issue #111 as a side effect rather than as a decision somebody made.

`docs/decisions/0002-track-order.md` already put rendering first, on the argument
that it is the half that can be measured from the first week. That argument
assumes the difference a measurement finds is a difference this repository can
fix. Which of the three routes below is taken is what makes that assumption true
or false.

## The route that is taken

Write the pipeline. The model is read once by `crates/model`, and `crates/render`
turns it into pages, owning every decision between the two.

The libraries underneath it are held to work that has nothing to do with
spreadsheets. Somebody else can turn a run of text and a font into positioned
glyphs, extract an outline, fill a path with anti-aliasing, and write the bytes
of an output container, and each of those is a specialism this project would do
worse. Nobody else can decide what this document says a cell looks like, and that
decision is the entire product.

So the boundary is not a matter of taste about dependencies. It is drawn at the
line between drawing and meaning, and the rule below is what keeps it there.

## The two routes that are not taken

### Drive an existing suite headless and capture what it draws

This is cheap, it is what a great deal of server-side document handling already
does, and it produces something usable in weeks rather than years.

What it costs is the whole claim. The fidelity of the result is the fidelity of
that suite, and that fidelity is the reason this project was started, so the
output is a wrapper around the complaint rather than an answer to it. A
difference found by the harness would be a difference in somebody else's
codebase, which turns every fidelity issue in this tracker into a bug report.

It also loses the conditions the rest of the plan is built on. Two renders
producing identical bytes, issue #42, becomes a property of a large application
this project does not control. Bounding a hostile document before it reaches a
parser, issue #80, becomes bounding a process instead. And the substitution
report that issue #38 requires is only as honest as what that suite reports about
its own font matching.

### Take an existing engine's rendering code and build on it

This inherits the code and the licence in one move, and the licence half is the
part that cannot be undone later.

Read at the date above rather than remembered:

    gh api repos/ONLYOFFICE/core --jq .license.spdx_id
    AGPL-3.0
    gh api repos/LibreOffice/core/contents --jq '.[] | select(.name|startswith("COPYING")) | .name'
    COPYING
    COPYING.LGPL
    COPYING.MPL

One answer and three files. Taking code from either would settle the first entry
of issue #111 by merge rather than by decision, and relicensing afterwards needs
the agreement of everybody who has contributed by then. That is the reason this
route is rejected even where the code is good, and it is the same argument
`docs/decisions/0001-means.md` makes against C++ for the same reason.

The second cost is that inherited rendering code carries the inherited model
behind it, so the model this project builds once would be competing with a second
one shipped inside the drawing code.

## What a library may decide, and what it may not

A library under this pipeline may decide how a thing is drawn. It may not decide
what the thing is.

May decide: which glyphs a run of text and a font produce and where they sit
relative to one another, the outline of a glyph, how a filled path is
anti-aliased, how a stroke is joined, and the byte-level syntax of an output
container.

May not decide: which font a cell asks for, what a number format prints, where a
cell's box is on the page, what a theme slot resolves to, which border wins where
two meet, what happens when content does not fit its cell, whether a chart type
is drawn or marked as not drawn, and what is reported as unrepresented. Every one
of those is a document semantic, and every one of them is an issue in this
tracker rather than a setting in somebody else's crate.

The failure this prevents is specific and it is not hypothetical for a project
that measures itself: a library upgrade that changes a rendering, where nobody
can say whether the document now means something different or merely looks
different. Under this rule the second is possible and the first is not.

## The libraries the pipeline sits on

These are the candidates the plan was written against rather than a dependency
list. Which of them this project actually takes is decided by the issue that
needs each one, under the dependency policy in `docs/decisions/0001-means.md`,
and a new direct dependency arrives with that issue saying what removing it would
take.

    for r in harfbuzz/rustybuzz harfbuzz/ttf-parser RazrFalcon/tiny-skia typst/pdf-writer image-rs/image-png; do
      gh api repos/$r --jq '.full_name + " " + .license.spdx_id'
    done
    harfbuzz/rustybuzz MIT
    harfbuzz/ttf-parser Apache-2.0
    linebender/tiny-skia BSD-3-Clause
    typst/pdf-writer Apache-2.0
    image-rs/image-png Apache-2.0

Shaping is rustybuzz, which turns a run of text and a font into positioned glyph
identifiers. Font file reading and outline extraction is ttf-parser.
Rasterisation of filled and stroked paths is tiny-skia. Vector output is
pdf-writer and raster encoding is image-png, and which output formats exist at
all is issue #35 rather than a consequence of this list.

Two things the command above does not tell you, said here rather than left to be
discovered. It reads the licence GitHub detects for a repository, which is a
single identifier, so a crate published under a dual licence is flattened to one
of them by this route; the terms an operator actually receives are collected by
issues #89 and #90 from the crates rather than from the repositories. And the
third line was asked for under a name that has moved: the request names
`RazrFalcon/tiny-skia` and the answer names `linebender/tiny-skia`, because the
API follows the redirect. `docs/decisions/0001-means.md` quotes the older
spelling, which was correct when it was written and still resolves.

## The stages between the model and the output

Named so that a later issue can say which stage it changes, and so that a
difference the harness reports can be attributed to one of them rather than to
rendering in general.

Geometry comes first: the sheet becomes boxes, from column widths, row heights,
merges, hidden rows and columns, and frozen panes. Issue #36 builds it, and every
position downstream is measured from it, which is why it is first and why it is
where a large share of fidelity differences will turn out to live.

Content resolution is not a stage here at all. What a cell's text is, which
colour a theme slot means, which font it asks for and which number format code
applies to it are resolved in the model before the renderer sees them, by issues
#17, #19, #20 and #21. The renderer applies a resolved format code and never
derives one of its own, it does not re-read the document, and it holds no second
opinion about what the document says.

Text shaping and placement follows, issue #37, and fitting after it, issue #40,
because whether a value spills into its neighbour or becomes a row of hash marks
is decided from the shaped width and the geometry together.

Painting produces a paint list: an ordered sequence of primitives in this
project's own vocabulary, with no type from any library above in it. Fills,
borders and their precedence are issue #39. Charts and drawings are the chart
milestone, and what is deliberately not drawn leaves a placeholder in this same
list, issue #64, so that an object nobody drew is a thing the list holds rather
than a gap in it.

Pagination takes the paint list and the page setup and produces pages, issue #41.

The device consumes a page of the paint list and emits bytes. It is the only
stage that knows which output format it is producing, which is what keeps issue
#35 a decision about outputs rather than a change that reaches back up the
pipeline.

Determinism, issue #42, is a property of the paint list rather than of the
device: the list is ordered before anything is drawn, so two runs cannot differ
because a collection iterated differently.

## The rule that keeps meaning out of the dependencies

No type from a rendering dependency appears in the model or in the paint list.
The paint list is this project's own vocabulary, and the device is the only place
a library type is constructed.

That is what makes a library swap a change to how a page is drawn and never a
change to what a document means. It is also the version of this decision a
machine can hold: it is a dependency direction and an architecture rule, which is
what issue #101 turns into a test, in the same way `crates/cli/tests/boundary.rs`
already refuses a crossing of the input boundary.

Until #101 lands, this section is prose. Nothing in the tree refuses a library
type that reaches the paint list today.

## What this costs

Years, and the cost is not softened here. Text shaping edge cases, the automatic
axis algorithm behind a chart, and the font problem in issue #38 are each large
enough to be somebody's whole project, and this route signs up for all of them.

It also means every fidelity difference is this repository's to fix, which is the
point of the route and is also its bill. A wrapper would have let a difference be
reported upstream and waited.

## What would reverse this

A measurement rather than an opinion. Issue #33 scores the existing open suites
on this project's corpus, with the same harness, the same references, and
versions, platforms and dates attached. If a suite driven headless scores on that
corpus what this pipeline scores, at a cost this project can pay, then the first
rejected route is the cheaper answer and this record is overturned by citing
those numbers.

Two things that do not reverse it. One expensive feature turning out to be more
expensive than expected is the cost this record already accepted. And a licence
change at either suite removes an argument against the second route without
supplying an argument for it, because the model half of that objection stands on
its own.
