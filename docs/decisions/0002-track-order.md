# 0002 Track order

Rendering fidelity is built first, the macro compatibility track second, on one
workbook model built once so that it serves both.

Status: accepted
Date: 2026-08-06
Issue: #2

## Context

The repository promises two things: complex macros that run, and documents that
look right. They share a substrate and nothing else. Doing both halves at once is
how a project of this shape drowns, so one of them is built first and the other
waits. The reasoning belongs here rather than in the memory of whoever started,
because every later issue is sequenced from it.

## Reasons, in the order they carry weight

Fidelity can be measured from the first week and macro conformance cannot. A
rendering difference is a picture beside a picture and a number that falls out of
comparing them. Macro conformance produces no number at all until a parser, an
interpreter and an object model all exist. That is a long stretch of work with no
evidence attached to it, and the first number that finally appears is dominated by
whichever object model surfaces happened to be implemented first rather than by
anything about the language.

The gap this project was planned against is bounded on the rendering side and
unbounded on the macro side. The rendering misses are specific document features:
complex conditional formatting, nested conditionals, embedded charts. A macro
compatibility layer is a language plus an object model that a very large
application has been growing for three decades, with no natural edge. Every
attempt at one has found the edge to be where the effort stopped rather than where
the surface ended.

Both tracks need the same workbook model, and building it under rendering pressure
produces the better version of it. Rendering touches nearly every part of a
document broadly: styles, themes, number formats, drawings, page setup. A macro
object model touches a narrow set deeply. A model shaped by the broad pass has
somewhere to put what the deep pass later needs. A model shaped by the deep pass
first has to be widened everywhere afterwards.

Rendering is the half with no hardware in it. It runs headless, unelevated, in a
container, deterministically, which is what the rest of the plan is built on. The
macro track will want process isolation and a resource ceiling, and it is cheaper
to inherit a suite that already proves those than to invent them alongside an
interpreter.

## What this costs, and who is not served

An operator whose problem is a macro that does not run gets nothing from the first
release. That is the direct cost of this order and it is not softened here.

The macro track starts later and therefore lands later. If the project stops
early, it stops holding half of what the readme promises, and the half it holds is
the rendering half.

The second half is not deferred to a vague future. Milestone 8 exists, its first
issue is a decision record of its own, and its first deliverable is a measurement
rather than an implementation, so the macro half also begins by producing
evidence rather than by producing code that cannot yet be scored.

## What the second track may assume about the first

Milestone 8 is designed against this list rather than against a guess.

It may assume one workbook model, read once, owned by one component, and that the
model is the authority on what a document says. A question about a document is
answered from the model and not by opening the file a second time.

It may assume the model holds parts that the renderer does not draw, including
parts no consumer uses yet, and that anything the model could not represent is
recorded rather than dropped.

It may assume a suite that runs headless and unelevated, with no display server,
no host font directory and no network, and that a new test in the macro track
inherits that condition instead of negotiating it.

It may assume the fidelity harness exists and can score a rendering, so a macro
that changes a workbook can be scored by rendering the result.

It may not assume that the calculation engine evaluates everything a macro can
reach, nor that any object model surface exists before it is measured and ranked.
It may not assume a route by which a macro reaches anything outside the workbook.
It may not assume that the renderer will be reshaped to suit it: where the two
tracks want different things from the model, the model grows and the renderer does
not become a second reader.

## What would reverse this

If the corpus, once built, shows that the remaining rendering gaps are narrow and
already closing in the existing open suites, the reason to spend years on them
goes away and the balance shifts to the macro half.

That is a measurement rather than an opinion, and issue #33 is the issue that
produces it. It scores the open suites on the same corpus, the same harness and
the same references as this project, with versions, platforms and dates attached.
Reversing this record means citing that measurement, not restating this paragraph
with the sign changed.

## Rejected alternative

Building both tracks at once. It costs the thing that makes either half
defensible, which is that the work is scored while it happens. Two half-built
tracks produce one number that is not yet meaningful and one that does not exist,
and the model ends up shaped by whichever track shouted last.
