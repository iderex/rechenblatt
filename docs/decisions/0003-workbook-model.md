# 0003 The workbook model

One model, read once, owned by `crates/model`. Every other component reads the
model rather than the document, and the suite refuses a component that names a
part of a document outside it.

Status: accepted
Date: 2026-08-07
Issue: #14

## Context

Two tracks in this repository want to look at a workbook, and they want
different things. The renderer wants nearly everything a document says about
appearance: styles, themes, number formats, drawings, page setup. The macro
runtime, when it comes, wants a narrow set of things it can also change.

The cheap answer is to give each one what it needs. It produces two readers of
one file, and two readers of one file disagree. They disagree on the documents
neither was written against, which is most of them, and the disagreement arrives
as a rendering that is wrong in a way nobody can locate, because the question
"what does this document say" now has two answers and no way to choose.

That is the failure this project exists to complain about in other software, so
it is settled here, before either reader exists.

## The decision, in full

There is one model. `crates/model` owns it, and it is the only component that
turns a document into one. Every other component reads the model.

`crates/calc`, `crates/render` and `crates/macro` are readers. So is
`crates/cli`, which hands the model to them. A question about a document is
answered from the model and never by opening the file a second time. The
workspace edges in `Cargo.toml` are the first half of that: the model depends on
nothing else here, so nothing it produces can be shaped by a consumer.

The model is written in two places and nowhere else. Its own reading writes it,
which is what building the model means. The macro track writes it through the
model's own interface, because a macro that changes a workbook is the whole
point of that track; issue #72 is where those surfaces are bound, one measured
surface at a time. A consumer that computes something from the model holds the
result beside the model rather than inside it: a calculated value is
issue #44's subject and not a field the renderer may write back.

### What the model holds beyond what the renderer needs today

Everything the document says, not everything the current consumer draws.

A model that holds what is currently drawn is a rendering cache with a
misleading name. It passes every test on the day it is written, because the
tests are written from the same list, and it fails the first time a second
consumer asks for a part nobody drew. That consumer then has one honest option
and one cheap one, and the cheap one is to open the file itself.

So the model holds parts the renderer ignores. It holds the ones the macro
object model will want, the ones a later milestone will draw, and the ones
nothing has a use for yet, because "nothing uses it" is a fact about today and
"the document does not say it" is a fact about the document. Where the model
genuinely cannot represent something, that is recorded rather than dropped, and
issue #18 is where the record and the rule that nothing is dropped in silence
are built.

### Eager and lazy

Reading everything is slower and larger than reading what is needed, and on a
workbook of the size a public body actually holds that is not a rounding error.
The answer is not to read less. It is to read later, and where the line falls is
part of this decision rather than an optimisation somebody adds afterwards.

Read eagerly: the package structure and the relationships between its parts, the
workbook part itself, the sheets that exist and their order and visibility, the
styles, the theme, the number formats, the defined names, and the record of what
could not be represented. Everything a question about the document *as a whole*
is answered from.

Read on demand: everything whose size follows the data rather than the
structure. The cells of a sheet nobody has asked about, the strings they share,
the drawings and charts anchored in it, embedded images, and the macro project's
bytes.

The rule for a part nobody has classified yet, in one sentence: it is eager if
its size is bounded by the shape of the document rather than by how much data
somebody put in it, or if a question about the document as a whole cannot be
answered without it. Otherwise it is lazy.

Lazy never means absent, and this is the half that is easy to get wrong. The
model records that an unread part is there, so a consumer can tell "not read
yet" from "not in this document". Those are different answers and a model that
returns the same thing for both has quietly become the silent-drop failure that
issue #18 is about, one indirection further down.

## Reasons, in the order they carry weight

A second reader is not added on purpose, so a rule about it has to be a refusal
rather than a paragraph. Nobody decides to write a second parser. Somebody needs
one part the model does not hold yet, opens the package for that one part
because it is four lines, and the second authority on what documents say now
exists and has a test suite.

The renderer is the broad consumer and the macro track is the deep one, and
`docs/decisions/0002-track-order.md` sequences the broad one first for exactly
this reason. A model shaped by the broad pass has somewhere to put what the deep
pass later needs. Two models shaped by their own consumers have nowhere to put
anything.

A fidelity number is only worth reading if one thing produced it. If the
renderer and the comparison harness reach the document by different routes, a
difference between them is a difference between two readers, and the number
stops being about rendering at all.

The input boundary is easier to hold with one crossing.
`docs/decisions/0014-input-boundary.md` puts every parsing component on one side
of a line; a single component that reads documents means the surface a hostile
file reaches is one component's public functions rather than a set that grows
each time a consumer gets impatient. Issue #96 attaches the fuzzing to that
surface, and it can only attach to a surface somebody can name.

## What the check does, and where it stops

`crates/cli/tests/model_ownership.rs` reads the workspace and refuses four
things: no member declaring that it reads documents, more than one declaring it,
an empty marker list, and a part of a document named in the source of a
component that reads the model.

The declaration is one line in a member's own manifest:

```toml
[package.metadata.rechenblatt]
reads-documents = true
```

What counts as naming a part of a document is data rather than code. The
workspace manifest carries `document-part-markers`, the text that appears in
code taking a document apart and nowhere else, and the check reads that list.
The list lives there so that whoever accepts a second input format adds its part
names in the commit that needs them, and so that the check's own source is not
something the scan has to be taught to skip.

Ten legs stand behind it. Each plants exactly one mistake in a scratch workspace
and requires that refusal and no other; each has a neighbour that changes the
one thing back and requires nothing at all. Two further tests read this tree
rather than a scratch one: one requires the workspace to hold up, and one
requires the component declaring the reading to be the model this record names.

Where it stops, stated rather than left to be discovered.

It scans each member's `src` directory and nothing else, so a document part
named in a component's own tests passes. That is the line
`crates/cli/tests/boundary.rs` already draws at `[dev-dependencies]`, and for the
same reason: a test is not the component.

It judges text, so a marker inside a comment or a string literal is refused
exactly as a parser would be. That is the intended reading rather than a
tolerated one. A component with reason to write a document part name down is a
component thinking about the file, and the repair is to reword the sentence.

It judges names and not behaviour. A component that reads a document part
without naming one, because something handed it the bytes, passes. Nothing in
the tree refuses that, and issue #101 is where the rest of the architecture
becomes tests.

A member whose declaration this check cannot read is treated as not declaring
it, which makes it scanned rather than exempt. A typo therefore tightens the
check; on the model it removes the only reader and the run says so.

## What this costs

The model is bigger than any one consumer needs, and it is bigger first. Parts
land in it before anything draws them, which means work whose payoff is a
milestone away and a reviewer asking why.

A consumer that wants something in a shape the model does not hold has to change
the model, which is a wider change than reaching for the file. That is the cost
of the rule working, and it is paid every time by the person least inclined to
pay it.

The lazy side is a source of bugs the eager side does not have. A part read on
demand is read at a moment nobody chose, and a consumer that holds a reference
across such a read is a shape this repository has not met yet. Issue #23 is
where reading a workbook larger than memory meets it first.

## What would reverse this

A part that the model cannot hold without becoming a union of two designs, where
holding it for one consumer makes it wrong for the other. Reversing this means
naming that part, saying which two consumers want incompatible shapes of it, and
saying why a derived representation held beside the model does not work.

A consumer keeping a derived representation of its own is not a reversal. It is
what the renderer is expected to do with geometry, and the test is where it got
the input: from the model rather than from the file.

Issue #23 is the case to watch. A workbook larger than memory may need a reader
that pulls, and a reader that pulls is one the current shape does not describe.
That is a measurement rather than an opinion, and it belongs in that issue.

## Rejected alternative

**A reader per consumer.** Each track reads what it needs, in the shape it wants
it, with no coordination. It is faster to start and it is the arrangement this
project was planned as an alternative to. Two readers produce two answers about
one file, and the first symptom is a fidelity difference that is really a
disagreement between two pieces of this repository. The counting is worse than
the bug: every measurement the fidelity milestone produces would be a
measurement of whichever reader the harness happened to use.

**One reader per format, sharing a model.** This is the version worth taking
seriously, and it is not rejected so much as deferred. A second input format
would need its own reading code, and putting it in the same component as the
first is not obviously right. What this record fixes is that there is one model
and one component that owns turning documents into it; how that component is
organised inside is not decided here, and issue #15 is where a second format
would first make it a real question.
