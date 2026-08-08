# What this project is for, and what it is not for

This software reads spreadsheet documents and renders them, and it is meant to
run on the machine of the person who already holds those documents. Everything
below follows from that sentence.

`NOTICE.md` carries the short form and points here. `README.md` says what the
software is. `SECURITY.md` says who is being protected and what counts as a
vulnerability. This document is the one that says which uses the maintainers
consider outside the project, so that a reader does not have to infer them from
the absence of a statement.

## What it is for

Opening a workbook an operator received or produced, and getting a page out of
it that matches what the incumbent suite would have produced, without sending
the document anywhere.

Running the macros that arrived inside such a workbook, as far as the
compatibility track has got, so that a spreadsheet an organisation depends on
keeps working.

Measuring how well any of that is done. The corpus and the published numbers
exist so that the claim can be argued with rather than believed.

The operator this is aimed at is named in `SECURITY.md`: somebody running the
software beside their own files, on their own machine, and not a hosted service
with a tenant per document.

## Rendering a document is not permission to have it

The software takes a path and reads the bytes at it. It has no way to know
whether the person who ran it is entitled to the document, and it does not try
to find out. A faithful renderer is also a good tool for reading a file somebody
was not meant to read, and that is worth saying rather than leaving implied.

So the entitlement is the operator's. Whether a document may be processed at
all, on what lawful basis, for how long the output is kept and who may see it
are decisions the person deploying this makes, and no setting in this software
makes them. Issue #85 is where that division gets written out in the terms the
law uses, and until it lands this paragraph is the whole of it.

## What the maintainers consider outside this project

Reading documents the operator has no right to read. Bulk extraction from a
collection somebody else assembled is the case this is aimed at, and the fact
that the software would do it well is the reason to name it.

Turning the macro track into a way of executing code from strangers with reach
outside the workbook. The default is the opposite of that and is not a starting
position to be relaxed, which is argued in
`docs/decisions/0010-macro-track.md`.

Using the published fidelity and compatibility numbers as a claim about somebody
else's software without the versions, the platform and the date that were
recorded beside them. A number quoted without those is a different number.

Deploying this as a document conversion service on the open network and calling
it the thing this project describes. The premise here is that documents stay on
the host, and an operator who exposes the surface has taken on a question this
project has not answered for them.

## What a macro may reach

A macro gets the workbook it arrived in and nothing else. What that excludes and
why is in `docs/decisions/0010-macro-track.md` rather than here, because a list
restated in two places is a list that will disagree with itself.

Two things that record settles and this one repeats only as pointers. Nothing in
the macro track is built assuming a route past that default exists. And
rendering a document executes nothing, so opening a file is not a decision to
run the code inside it.

Whether the software will ever offer a configured route past the default is not
decided, and this document does not assume an answer in either direction. Issue
#67 holds that question.

## What this document does not claim

It does not claim the software prevents any of the uses named above. There is no
check inside a renderer that can tell a document its operator may read from one
they may not, and a claim otherwise would be a false assurance in the one place
that most needs a true one.

It also does not claim that misuse is somebody else's problem. The maintainers
choose what to build, what to refuse to build, and what to say plainly, and this
document is the third of those. Where a design choice can narrow a misuse
without costing the operator anything, the plan takes it, and the macro
capability default is the largest example in the tree today.

## Where a disagreement with this goes

The public tracker, as an issue, in the ordinary way. A disagreement about the
uses named here is a disagreement about what the project is, so it belongs where
the rest of the plan is argued rather than in a private thread. Anything
exploitable goes the other way, and `SECURITY.md` has that route.
