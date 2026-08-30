# 0006 Rendering

This project writes its own rendering pipeline, from the model to a page, on top
of libraries that place glyphs and fill paths and decide nothing about what a
document means.

Status: accepted
Date: 2026-08-08
Issue: #34

## Context

There are three ways to get a document rendered and only one of them is this
project.

The first is to drive an existing office suite in a headless mode and capture
what it draws. It is cheap, it is what a great deal of server-side document
handling already does, and the fidelity of the result is the fidelity of that
suite. Since a fidelity gap in that suite is the reason this repository exists,
that route produces a wrapper around the problem rather than a remedy for it.

The second is to take an existing engine's rendering code and build on it. That
inherits the code and the licence in one move. This repository is published under
the GNU Affero General Public License version 3, so anything taken has to arrive
under terms that grant can carry, and a differently copylefted engine would settle
the question by what was copied rather than by anybody here.

The third is to write the pipeline and own every decision in it. It is expensive
and it is the only version in which a fidelity difference is something this
repository can fix.

## The decision, in full

Write the pipeline. Own every decision between the model and the page.

Sit it on libraries that do the two things somebody else does better than this
project will: turning a font and a run of text into positioned glyph outlines,
and turning outlines and paths into pixels or into a vector stream. Those
libraries are chosen for being permissively licensed and for having no opinion
about spreadsheets.

### The line the libraries may not cross

A dependency in this pipeline may decide where a glyph outline sits, what a
curve looks like when it is filled, and how a path is written into an output
stream. It may not decide what a document means: not which font a cell asked
for, not what a border does where two of them meet, not whether a number is too
wide for its column, not what a theme colour resolves to.

The rule, stated so a future dependency can be judged against it: **the pipeline
holds no document semantics in a dependency.** If swapping a library could
change what a document is understood to say, the library is on the wrong side of
this line and the semantics belong in this repository.

That is what keeps the fidelity number meaningful. A difference this project can
fix is one whose cause is code in this tree; a difference caused by a
dependency's idea of what a spreadsheet is would be one that is argued with
upstream and lived with meanwhile.

### The libraries, and what each may decide

Nothing here is in the dependency set yet.
`docs/decisions/0014-input-boundary.md` keeps `pure-may-depend-on` empty until
somebody has read a crate closely enough to say what it can reach, and each
library below arrives in the commit that needs it, with that reading done. This
record names the shape of the pipeline's floor, not a set of edges that already
exist.

Licences read on 2026-08-08, at the version each crate declared as its maximum
stable one:

```
for c in ttf-parser rustybuzz tiny-skia pdf-writer image; do
  curl -s -A "rechenblatt licence check" "https://crates.io/api/v1/crates/$c" |
    python -c "import sys,json;d=json.load(sys.stdin);v=d['crate']['max_stable_version'];print('$c', v, next(x['license'] for x in d['versions'] if x['num']==v))"
done
ttf-parser 0.25.1 MIT OR Apache-2.0
rustybuzz 0.20.1 MIT
tiny-skia 0.12.0 BSD-3-Clause
pdf-writer 0.15.0 MIT OR Apache-2.0
image 0.25.10 MIT OR Apache-2.0
```

That is the licence each crate declares for itself. It is not a reading of what
its own dependencies carry, and issue #89 is where the notices an operator has to
ship are derived from the tree rather than from this paragraph.

**Font parsing.** Reads a font file and answers questions about it: which glyph
a character maps to, what a glyph's outline is, what the metrics are. It may not
decide which font is used. Choosing a family, and substituting when the named
one is absent, is this project's decision and issue #38 is where it is made.

**Shaping.** Turns a run of text and a font into positioned glyphs, with
ligatures, kerning and the scripts that need reordering. It may not decide where
the run starts or stops, what it is styled with, or what happens when it does not
fit; those come from the model and from issue #40.

**Path rasterisation.** Fills and strokes paths into pixels, with anti-aliasing.
It may not decide what is drawn or in what order. Fill and border precedence is
issue #39 and it is document semantics.

**Vector output.** Writes a page of already-decided drawing operations into a
vector stream. It may not lay anything out. Which outputs the first release
emits and what each is for is issue #35, and this record deliberately does not
answer it: naming the shape of the last stage is not the same as naming the
formats.

**Raster image decoding.** Decodes the images a document embeds so they can be
placed. It decides nothing about placement, anchoring or scaling, which is issue
#63.

### The stages between the model and the output

Named here so a later issue can say which stage it changes, and so a fidelity
difference can be attributed to one of them rather than to rendering in general.

1. **Resolve.** Everything indirect becomes concrete. Theme colours, tints,
   indexed colours, scheme fonts, number formats, conditional formats that
   apply. The input is the model; the output is a description of every cell as
   it is meant to appear. Issue #20 is where the model's half of this lands.
2. **Lay out the sheet.** Column widths, row heights, merges, hidden rows and
   columns, frozen panes: the sheet becomes geometry with everything in a
   coordinate space. Issue #36.
3. **Fit the content.** Text is shaped and measured, and what does not fit its
   cell is resolved: overflow into empty neighbours, the fill behaviour for a
   number, clipping and wrapping. Issue #37 and issue #40.
4. **Paginate.** Page setup, scaling, breaks, repeated headers. A sheet becomes
   a sequence of pages. Issue #41.
5. **Build the display list.** Each page becomes an ordered list of drawing
   operations - fills, strokes, glyph runs, images, placeholders - with every
   semantic question already answered. This is the last stage that knows what a
   spreadsheet is.
6. **Emit.** The display list is written out. This stage knows nothing about
   documents: it takes operations and produces bytes.

The boundary between stage 5 and stage 6 is where the rule above is enforceable
by construction. A dependency reached from stage 6 cannot change what a document
means, because by then nothing about the document is left to decide.

## Reasons, in the order they carry weight

A fidelity gap this project cannot fix is not worth measuring. The whole plan is
a corpus, a number and a route from a difference to a repair, and every one of
those requires that the code producing the difference is code somebody here can
change.

The licence question stays open until it is answered on purpose. Building on
permissively licensed floors leaves every option in issue #111's first entry
available, including the copyleft ones. Taking a copyleft engine's rendering
code would remove three of the four options in a commit that was about drawing.

The expensive parts are not the ones a dependency solves. Shaping and
rasterisation are hard and they are solved. What is unsolved is the hundred
decisions about what a document says, and no library outside this repository is
going to make those correctly for a spreadsheet, because they are not spreadsheet
libraries.

A pipeline written here is a pipeline that can be made deterministic. Issue #42
requires two renders of one document to produce identical bytes, which is a
property of every stage at once, and it is far easier to hold in code this
repository controls than to obtain from a suite driven from outside.

## What this costs

**Time, in the amount that decides the project.** Text shaping is bought;
everything else in the six stages is written. That is the largest single cost in
the plan and it is chosen with the alternative named above rather than by
default.

**A long stretch with a low number.** A wrapper around an existing suite would
produce a high fidelity score in a fortnight. This route produces a low one for a
long time, and the corpus makes that visible on every pull request, which is the
point and is still a cost.

**Every gap is this project's gap.** With no engine underneath, there is nobody
to attribute a difference to. That is the same sentence as the first reason
above, read from the other side.

**A floor made of several libraries is several things to track.** Each one is a
version, a licence, an advisory feed and a notice an operator ships. Issue #89
and issue #99 are where that is held.

## What would reverse this

A library that turns out to hold document semantics after all, discovered when a
fidelity difference traces into it and cannot be fixed here. The response is not
to reverse this record but to move that decision into this tree; the record
reverses only if the pipeline cannot be written without such a library.

The measured condition: if the corpus number after the rendering milestone is far
enough below what driving an existing suite would produce, and the gap is not
closing between milestones, then the premise that owning the pipeline is what
makes a difference fixable has failed in practice. Reversing then means naming
that measurement and its date, not restating this record with the sign changed.

## Rejected alternatives

**Drive an existing suite headlessly and capture what it draws.** Cheapest by a
wide margin, and it is what much of the industry does. The fidelity of the result
is the fidelity of that suite, so this project's central claim would be a claim
about somebody else's code, and a difference a reader reports would be a
difference nobody here can repair. It also carries an operational cost that is
easy to miss: a full office suite in the artefact, with its own attack surface,
started for every document.

**Take an existing engine's rendering code.** It inherits years of correct
behaviour, and it inherits the licence with them. The two open suites this
project is measured against are copyleft, and one is network copyleft, so this
route decides issue #111's first entry by accident, in a direction that forecloses
the other options and cannot be undone later without removing the code. It also
inherits an architecture built around a different model of a document, which is
the thing this repository has decided to own.

**Write the shaping and rasterisation too.** Rejected in the other direction and
for the same reason as the first: a project that writes its own text shaping
spends its years on a solved problem and reaches the unsolved one later. The line
between the two is the one drawn above, and it is the line this record is
actually about.
