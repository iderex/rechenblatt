# 0007 Output formats

The renderer emits two formats in the first release: a paginated vector document
and a paginated raster image sequence. Nothing else is an output, and the
structured description of what was drawn stays inside this project rather than
becoming a third one.

Status: accepted
Date: 2026-08-27
Issue: #35

## Context

`docs/decisions/0006-rendering.md` names the last stage of the pipeline and hands
this question over rather than answering it:

    git grep -n 'issue #35' -- docs/decisions/0006-rendering.md
    docs/decisions/0006-rendering.md:104:emits and what each is for is issue #35, and this record deliberately does not

The paragraph it sits in describes the vector writer as a library that takes a
page of already-decided drawing operations and writes them into a stream, and
says that naming the shape of the last stage is not the same as naming the
formats. This record names them.

Three candidates were on the table, and they are not alternatives to each other
in the ordinary sense, because each answers a different consumer.

A vector document keeps text as text, keeps a page a page, and is what a print
route or an archive wants. It also permits a comparison that is not a pixel
comparison, because the drawing operations survive into the file.

A raster image is what an image comparison needs and what a preview wants. It is
also where every tolerance question in this project lives, because two renderings
differing by one pixel of anti-aliasing are the same rendering to a reader and
different to a byte comparison.

A structured description of the display list is useful to nothing but a harness.
It is the cheapest of the three to produce, since stage 5 already builds the
thing, and it is the only one with no consumer outside this repository.

## The decision, in full

**The vector output is PDF.** One page per rendered page, in the order the
pagination stage produced them. It is the format the print and archive cases ask
for, it has the page model this project's fourth stage produces, and it is the
one the pipeline's floor already reaches:

    git grep -n 'Vector output' -- docs/decisions/0006-rendering.md
    docs/decisions/0006-rendering.md:102:**Vector output.** Writes a page of already-decided drawing operations into a

Which conformance level, and whether an archival profile is offered at all, is
not decided here. That is a question about long-term preservation with its own
obligations, it has no consumer in the first release, and taking it now would
bind the emitter to a profile before anything has been emitted.

**The raster output is PNG, one file per page.** Lossless, so a fidelity
comparison compares renderings rather than compression artefacts, and paginated
on the same boundaries as the vector output, so a difference can be pointed at
one page in both.

**The display list is not an output format.** It is a diagnostic surface, printed
by the command for a person or a harness that asks for it, and it carries no
promise of a stable shape. Making it a format would mean promising that shape to
whoever reads it, and the only reader is a comparison harness that does not
exist. Whether the fidelity comparison needs a structural reading at all is issue
#24, and issue #28 is the harness that would do it. If either lands on a
structural comparison, the promise is made then, with a consumer to make it to.

### What the comparison harness uses

The raster output, because a reference rendering produced by another renderer
arrives as an image and as nothing else. A comparison needing both sides to emit
a structure can only ever compare this project against itself.

That is the half this record settles. What the fidelity number is, and what
tolerance a pixel comparison carries, is issue #24, and where the references come
from is issue #29. This record decides which artefact the harness is handed, not
what it does with it.

### The raster output declares its resolution and its colour space

Both are stated by the run rather than inherited from a host, and both have a
documented default.

**Resolution: 96 pixels per inch by default.** A spreadsheet's geometry is
declared in points and in the device-independent pixel that maps to that number,
so a page rendered at it has one pixel where the document has one unit, and no
scaling decision is folded into the image before anybody has asked for one. A
higher value is a parameter, and a comparison run names its own value rather than
taking the default, because a number measured at whatever the default happened to
be is a number that moves when the default does.

**Colour space: sRGB, eight bits per channel, no colour management.** The colours
a workbook declares are sRGB values, and a renderer converting them through a
host display profile produces a different image on a different machine. That
would defeat two things at once: the byte-identical requirement of issue #42, and
the fidelity comparison, which cannot separate a colour-management difference
from a rendering one.

The rule and the default are what this record fixes. The parameter itself is
code, and there is no renderer in the tree to carry it. Issue #77 is the command
that would expose it and issue #79 is where a setting is validated.

### Fonts in the vector output

A vector document either carries the fonts it draws with or names them and hopes
the reader has them, and this is the only place a licence reaches into the
output.

**Where the font may be embedded, a subset of it is embedded**, holding the
glyphs the document actually used and no more. That is what makes a page render
the same on a machine that has never seen the font, which is the reason to emit a
vector document at all.

**Where the font may not be embedded, it is referenced by name and the page
records a substitution** in the register carrying everything else the model could
not represent, so the output declares that it is not what the document asked for.
A page silently referencing a font the reader does not have would look correct to
the renderer and wrong to the reader, with nothing in between saying which
happened.

**What decides whether a font may be embedded is what the font file declares**,
read out of the file in hand. This project does not read licence prose and does
not judge it, and that limit is stated here rather than left to be discovered: a
font whose file permits embedding may still be one an operator is not licensed to
redistribute, and that is the operator's decision rather than the renderer's.

The rule is written to survive entry 5 of issue #111, which is open and asks
whether this project ships metric-compatible substitute fonts at all. Each of its
three answers changes which files are in hand and changes none of the three
sentences above, because every one of them is about the file being drawn with
rather than about which files ship. Issue #38 is where the substitution decision
itself is made.

### Both outputs are byte-identical across runs

Named here to say that it binds both emitters, and not restated:

    git grep -n 'Issue #42' -- docs/decisions/0006-rendering.md
    docs/decisions/0006-rendering.md:158:A pipeline written here is a pipeline that can be made deterministic. Issue #42

It reaches the emitter harder than it reaches the stages above it, because a
vector writer has several places to put a timestamp, an identifier or a
non-deterministic ordering that no earlier stage produced. Whichever emitter
lands, it lands with those held down.

## Reasons, in the order they carry weight

**Two consumers, two formats, and neither can serve the other.** The comparison
needs pixels, because that is what a reference from another renderer is. The
print and archive case needs text to stay text. Choosing one format would leave
one of the two served by a conversion done outside this project, which is where a
difference nobody here can fix comes from.

**The rasteriser stays in this tree.** Emitting only a vector document and
letting somebody else turn it into an image would put the last step of every
fidelity measurement in a program this repository does not control, so two runs
could differ for a reason nobody here can repair. That is the failure
`docs/decisions/0006-rendering.md` rejects a headless office suite for, arriving
at the other end of the pipeline.

**A format with no consumer is a promise with no reader.** The display list is
the cheapest of the three to emit and the only one whose shape would be fixed by
a reader that does not exist. Holding it as a diagnostic keeps stage 5 free to
change while the pipeline is being written, which is exactly when it will.

**Two formats is the smallest set answering the plan.** Every issue in the
rendering and fidelity milestones asks for one of the two, and none of them asks
for a third.

## What this costs

**Two emitters rather than one**, each with its own determinism problem and its
own set of things that can quietly vary between runs.

**The tolerance question is not avoided, only located.** It lives in the raster
comparison, and PNG being lossless removes compression from the argument without
removing anti-aliasing from it.

**A vector document is a licence surface.** Fonts travel inside it, which is why
the rule above exists, and it means an operator's output can carry something an
operator's licence has an opinion about. Issue #89 is where the notices are
derived.

**No archival profile.** An operator needing one does not get it in the first
release, and the sentence above says so rather than leaving them to find out from
a validator.

## What would reverse this

The harness landing on a structural comparison. If issue #24 decides the fidelity
number is computed over drawing operations rather than over pixels, the display
list becomes a format with a reader and this record's third decision is wrong.
Reversing then means naming that decision, not arguing this one again.

A measured reason to drop the vector output. If the print and archive case has no
user by the first release, and every number this project publishes is measured on
the raster path, then PDF is a format kept for an argument rather than for a
consumer. That is a count of who asked for it, with a date, and not a judgement
about elegance.

## Rejected alternatives

**Raster only.** Cheapest, and it serves the comparison completely. It gives up
text staying text, gives up the page as a unit an operator can print or archive,
and gives up the only comparison available that carries no tolerance. It also
makes this project's output uniquely unhelpful for the case an operator most
often has, which is turning a spreadsheet into something they can send.

**Vector only, rasterised outside.** Attractive because it is one emitter, and it
moves the last step of every measurement into a program this repository does not
control. Two renderings of one document could then differ because a converter
changed, and no fidelity number would be attributable to this tree.

**SVG as the vector format.** One file per page, readable as text, easy to
compare with a diff. It has no page model, so pagination would be re-expressed
per file rather than carried by the format, and font embedding in it is a second
mechanism to get right for the same rule. The comparison advantage it appears to
offer is the display list's advantage, and the display list is cheaper and
already built.

**The display list as a first-class output.** It costs almost nothing today and
it fixes the shape of stage 5 to whatever the first reader wanted. Every stage in
the pipeline is expected to move while the rendering milestone is written, and
stage 5 is the one that changes whenever a semantic question is answered.
