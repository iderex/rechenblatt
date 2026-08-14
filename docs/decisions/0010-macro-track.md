# 0010 The macro track

The macro track delivers an instrument first and grows a runtime out of it: it
reads the macros in a corpus of real documents, reports which language constructs
and which object model surfaces they actually use, and executes only the subset it
supports while refusing the rest by name.

Status: accepted
Date: 2026-08-06
Issue: #66

## Why the instrument and not the runtime

There are two products behind one description. One is a runtime that executes the
macros in a document so that a spreadsheet an organisation depends on keeps
working without the incumbent suite. That is the product the sovereignty argument
wants, and it is a very large surface with an object model a very large
application has been growing for three decades.

The other is the instrument. It is small, it is finishable, and it produces the
measurement that tells anybody, including whoever builds the first product, where
the effort should go. Every previous attempt at the first product ran out of
effort somewhere inside the object model, a long way short of its edge, and none
of them could say beforehand which surfaces mattered.

The instrument is also the only half of the macro track that can be scored from
the start, which is the same reason the rendering track was built first in
[0002-track-order.md](0002-track-order.md). A track that produces no number until
a parser, an interpreter and an object model all exist is a track that runs for a
long time on nobody's evidence.

## The rule by which something is implemented

A language construct or an object model surface is implemented because documents
use it, and a count does the ranking. This is the same rule the function library
follows on the calculation side: the corpus decides what the set holds, and every
addition to it comes out of a measurement. Nobody copies a reference manual into
it.

The measurement that ranks them is the macro corpus report, issue #69, which says
what real macros are made of. Issue #71 implements the core language in the order
that measurement ranks, and issue #75 reports compatibility per construct, each
figure measured, with no single score over the top.

Three consequences of stating the rule this way.

A construct that no corpus document uses is not implemented, however easy it
looks, because implementing it moves no number and adds a surface to maintain.

A construct that many corpus documents use is implemented even where it is
awkward, because the ranking is the authority and an awkward construct that
documents actually contain is exactly what an instrument is for.

Anything the parser could not read gets its own report and never lands in the
unsupported column. Issue #70 keeps the two apart, because a construct nobody
implemented and a construct the
parser choked on are different failures and only one of them is about the
language.

## What the macro track may assume, and what it may not

It may assume one workbook model, read once, owned by one component, and that the
model is the authority on what a document says. Reading a document a second time
inside the macro track is how two readers that disagree about one file get built,
which is the failure this project exists to complain about elsewhere.

It may assume that anything the model could not represent is recorded. Nothing is
dropped on the way in, so a macro touching a part the model does not hold
produces a reportable gap, and the answer it returns is never quietly wrong.

It may assume a suite that runs headless and unelevated. A new test in the macro
track inherits that condition and has no standing to negotiate it.

It may assume that a write through the object model triggers recalculation the
same way any other change does, because a macro that changes a cell and then
reads a dependent one is the ordinary case here.

It may not assume that the calculation engine evaluates everything a macro can
reach. The function library is bounded by what the corpus needs, and a macro
calling a function outside that bound meets a named refusal rather than a
silently wrong value.

It may not assume that any object model surface exists before it has been
measured and ranked. A surface is not added because the interpreter would be
tidier with it.

It may not assume a route by which a macro reaches anything outside the workbook.
That is the next section and it is not negotiable by convenience.

## Permanently out of scope

Anything that lets a macro reach the host is out of scope for this track, and the
default is not a starting position to be relaxed as the interpreter matures. A
macro may read and write the workbook it came from. It gets no filesystem, no
network, no process, no environment, no clock beyond one the caller supplies, and
no access to another open document.

Whether the software offers any configured route past that default at all is a
question this record does not answer, and it is not the interpreter's to answer
either. Issue #67 holds it, decides it before a line of a macro is interpreted,
and is where the capability set, the default and the granting route are written
down. This record's part is narrower and it is absolute: nothing in the macro
track is built that assumes such a route exists, so that the answer to #67
changes a configuration surface rather than the shape of the interpreter.

Also permanently out of scope: running a macro because a document asked to be
opened. Rendering executes nothing. Issue #74 is where that becomes a test with a
document whose open handler would leave an observable trace.

## What would make the runtime the priority

The instrument's own report is the condition. If it shows that a small,
enumerable set of constructs and object model surfaces covers most of what real
documents contain, the runtime stops being an unbounded surface and becomes a
finite piece of work, and the argument for holding it back disappears.

That is a number the macro corpus report produces, so the reversal is checkable.
#69's output is what it is checked against, and nobody's sense of how the work is
going enters it.

The condition that does not reverse it is demand. An operator asking for a macro
to run is the expected state of the world throughout this track and was priced in
when [0002-track-order.md](0002-track-order.md) was written.

## Rejected alternative

Building the runtime first and measuring afterwards. It produces no number until
three large pieces exist, and the first number it does produce is dominated by
whichever object model surfaces happened to be implemented first rather than by
anything about the documents. It also puts the permission question after the
interpreter instead of before it, which is the order that turns a compatibility
layer into a way of running a stranger's code on somebody's machine.
