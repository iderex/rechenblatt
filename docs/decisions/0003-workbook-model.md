# 0003 The workbook model

One component reads a document, once, into one model, and everything else in
this repository reads that model rather than the file.

Status: accepted
Date: 2026-08-08
Issue: #14

## Context

Two tracks in this repository want to look at a workbook and they want different
things from it. The renderer wants nearly everything a document says about
appearance: geometry, styles, themes, number formats, drawings, page setup. The
macro track, when it arrives, wants a narrower set of things it can also change,
and it wants them to behave like objects rather than like parts of a file.

The cheap answer is to give each one what it needs, which produces two readers
of one document. Two readers disagree eventually, and they disagree quietly:
each is correct about the document as it read it, and the difference only
surfaces as a rendering that does not match what a macro just changed. That is
the failure this project exists to complain about in other software, and
`docs/decisions/0002-track-order.md` is where the argument that the two tracks
share a reader was first made.

The line is worth drawing while the components are empty. A model retrofitted
onto two readers is a rewrite of both.

## The decision, in full

`crates/model` owns the document. It is the only component that opens a package,
walks a part, or knows what a part is called. Every other component asks it.

**Who reads it.** `crates/calc`, `crates/render`, `crates/macro` and
`crates/cli` all read the model, and none of them reads a document.
`docs/architecture.md` holds the edges between them and
`crates/cli/tests/component_edges.rs` refuses one the note does not have; that
graph and this record are about different things, and both are needed. An edge
says which component may call which. This record says what may be behind the
call: the model, and never the file.

**Who writes it.** The model writes itself while it reads. After that,
`crates/macro` is the only component that changes a model, because changing a
workbook is what the macro track is for, and it does so through an interface the
model owns rather than by reaching into its fields. The renderer and the
evaluator do not write: a renderer that can change what it draws from is a
renderer whose output is not a function of the document, and the evaluator's
results are values it returns rather than edits it makes. Issue #47 is where the
evaluator's own storage is decided, and issue #72 is where the macro track's
write surface is measured and bound.

**The model is the authority.** Any question about a document is answered from
the model. A consumer that finds the model cannot answer its question widens the
model; it does not go around it. That is a rule about where work goes rather
than a restriction on what can be built, and it is what keeps one answer per
question.

### What the model holds that nothing draws yet

The model holds what the document says, not what the renderer currently reads.
It carries parts no consumer uses today: the parts the macro object model will
want, the parts a later output format will need, and the parts whose only
current use is to be reported as unrepresented.

A model that holds only what is drawn is a rendering cache with a misleading
name, and it fails in a specific way. The next consumer needs something that was
never kept, so it reads the file, and the second reader is born for a reason
that looks entirely sensible in its own commit.

There is a second reason and it is the one that matters for the numbers this
project publishes. Fidelity is measured against what a document contains, so a
model that quietly drops a part reports a good score on a document it emptied.
Issue #18 is where what the model cannot represent becomes a list that travels
with the model, and this record is what makes that list possible: a thing has to
reach the model before the model can say it could not hold it.

### The eager and lazy split

Reading everything eagerly is slower and larger than reading what is needed, and
on a large workbook that is the difference between a service and a demonstration.
The answer is not to read less. It is to read later, and where the boundary sits
is part of this decision rather than an optimisation somebody adds afterwards.

The rule, so that a new part does not need a meeting:

- **A part whose cost is bounded by the document's structure is eager.** The
  workbook part, the sheet index, the relationship graph, the styles, the theme,
  the number formats, the defined names. There is one of each per workbook, they
  are small, and everything else is interpreted through them.
- **A part whose cost is bounded by the document's content is lazy.** Cell data
  per sheet, embedded images and other media, chart parts, the macro project.
  These scale with what somebody put in the document, and a workbook with two
  thousand sheets is a workbook where reading the second thousand eagerly is
  work nobody asked for.
- **A part that has to be read to answer a question about another part is
  eager**, whatever its size. Laziness that has to be resolved before the model
  can be described is not laziness.

Two properties hold whichever side a part falls on.

Laziness is invisible from outside. A lazy part is behind an accessor that
returns the same value however many times it is asked, so no consumer can tell
which side of the split a part was on, and moving a part from one side to the
other changes no caller. A lazy read can fail, and the accessor says so in its
return type rather than at the moment of construction.

Laziness is over bytes, not over a file. `docs/decisions/0014-input-boundary.md`
puts the model on the parsing side, which opens no path it was not handed, so a
model that reads a part later reads it out of what it was already given. A
document larger than the memory that holds it therefore needs something this
record does not provide, and issue #23 is where that is decided rather than
assumed.

## What the check does, and where it stops

`crates/cli/tests/document_parts.rs` refuses a component other than the model
that names a part of a document in its source, and refuses a workspace where no
member answers to the reader's name.

Nine legs stand behind it, beside the one test that judges this tree. Each plants
exactly one thing in a scratch workspace and requires that refusal and no other;
each has a neighbour that changes the one thing back and requires nothing to be
refused.

The near-miss it is aimed at is one line rather than a rewrite. The renderer
wants a theme colour the model does not carry yet, and somebody writes the name
of the theme part into `crates/render`. Nothing else about that change looks
wrong. It compiles. It adds no dependency, so the edges are unmoved. It adds no
capability, so the input boundary is unmoved. The two dependency checks in this
tree both pass a second reader that is written inside a component that was
already allowed to exist.

Where it stops, stated rather than left to be discovered.

It judges string literals, not behaviour. A part name assembled from pieces at
runtime passes it, and so does one read out of a configuration file. That is the
bound of any text scan, and it is why this check sits beside the two dependency
checks rather than replacing either.

It reads the double-quoted spans of a line, so a component may name a part in a
comment. Saying in prose which part the model read is how a component explains
itself; naming it in code is how a component reaches for it.

It does not read a member's own tests, on the line
`crates/cli/tests/boundary.rs` draws for dependencies and for the same reason: a
test is not the component.

Its marker list is the part names of one package shape and it is not the format
decision. Which formats the first release accepts is issue #15, and the first
entry of that answer that is not a container of named parts - a compound file
holding streams, for instance - is covered by nothing here until somebody adds
its names.

And it says nothing about the split above. Nothing in this tree refuses a part
read eagerly that this record says is lazy; that is a judgement about cost, the
suite is where it would show up as a number, and no check is written for it
today.

## What this costs

**The model is on the critical path for everything.** A renderer that needs a
property the model does not carry cannot proceed until the model carries it, so
one component becomes the place where two tracks queue. That is the cost of one
answer per question, and it is paid deliberately.

**A widening that serves one consumer lands in a component shared by all of
them.** The repair for a missing property is an edit to the model, which is the
component with the most readers and the most tests. That is more expensive than
a local read, and it is the expense that keeps the two readings identical.

**Some parts are read for nobody.** A part that no consumer uses is still read,
still held and still tested. The alternative is not reading it, and the section
above is what that costs instead.

**A lazy part is a second place a read can fail.** An accessor that can fail is
harder to use than a field, and the failure arrives later than the open did,
which is a worse moment to report it. It is bought for the large-workbook case
and it is not free.

## What would reverse this

A consumer whose needs are genuinely disjoint from the document. If the macro
object model turns out to want a view that no reading of the file produces, and
building it from the model costs more than reading the file twice, then the
premise here is wrong and the record is replaced rather than amended.

The other reversing condition is measured rather than argued: if the eager half
cannot be made small enough for the documents in the corpus, so that opening a
workbook to answer one question reads the whole of it, the split moves or the
model gains a mode. Issue #23 is where that measurement lands.

Reversing this means naming the consumer or the measurement and saying why one
model did not work. It does not mean restating this record with the sign
changed.

## Rejected alternatives

**A reader per consumer.** Each track parses what it needs, directly, and the
model is not a component at all. It is faster to write, faster to run, and every
question about a document then has as many answers as there are readers. The
project's whole claim is that the model says what the document says, which is a
claim only one reader can make. It was rejected for that, and the cost of the
rejection is written above rather than left implicit.

**One reader with a per-consumer view built at read time.** A middle position:
one parser, two shapes handed out, each tailored. It keeps one reading of the
bytes and loses the single answer, because a view is a decision about what to
drop and the two views drop different things. The same disagreement returns one
layer up, where it is harder to see.

**A model that holds only what the renderer draws today.** Smaller, faster and
testable against the thing being built. It makes every later consumer a reason
to read the file again, and it makes the unrepresented-content list of issue #18
impossible to populate, because a part that was never read cannot be reported as
one the model could not hold.
