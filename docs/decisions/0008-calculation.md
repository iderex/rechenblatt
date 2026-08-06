# 0008 Calculation

An evaluator is built. The value a document already carries is used where it is
present and consistent, evaluation happens where it is not and wherever display
depends on it, and every displayed value records which of the two produced it.

Status: accepted
Date: 2026-08-06
Issue: #44

## Context

A workbook usually carries the last computed value of every formula cell beside
the formula, so a renderer can display a document without evaluating anything.
That is fast, and for a file the incumbent suite saved a moment ago it is also
correct.

It stops being correct for a file a tool wrote without filling the cache, for one
whose cache is stale, and for one whose cache was written by something that
computed a different answer. And it does not survive the rest of this plan at
all. Conditional formatting rules are formulas evaluated at display time, which
is milestone 6 and the gap issue #52 puts first, so a renderer that never
evaluates cannot paint it. The macro track changes cells and needs the
consequences, which is milestone 8.

That the cache exists in the accepted format, that a document can ask to be fully
recalculated on load, and that a stale cache is common in files written by other
tools are claims here rather than measurements. Issue #51 is where they become a
number, because it scores the evaluator against the values corpus documents
already carry, and that comparison is the same measurement read from the other
side.

## The rule about the value a document carries

The cached value is used when it is present and consistent, and consistent means
three things at once: the cell carries a value, the document does not ask for a
full recalculation, and nothing the cell depends on has been evaluated to a
different answer in this run.

Evaluation is forced in four cases. When any of those three conditions fails.
When the value feeds a display decision rather than being one, which covers
conditional formatting conditions, data bar and colour scale bounds, and chart
series ranges. When a macro has changed a cell, because the consequences of that
change are the whole point of the macro track. And when the operator asks for it,
which is a switch rather than a guess.

The first case is worth stating in the direction it actually runs: consistency is
not a property of a cell, it is a property of a cell and everything under it. A
cell whose cached value is intact but whose input this run computed differently
is not consistent, and taking its cache would publish a workbook that disagrees
with itself.

## How a value's origin is recorded

Every value the model can display carries which route produced it: read from the
document, or evaluated here. It is a property of the value rather than a log
line, for the same reason `docs/decisions/0002-track-order.md` gives for the
model generally, and because a fidelity difference is attributable only if the
report can say which of the two the number came from.

A fidelity difference on a cell whose value was read from the document is a
reading or a rendering problem. The same difference on a cell this project
evaluated is a calculation problem. Those go to different issues and often to
different people, and guessing which one it is from the picture is how a
milestone is spent in the wrong place.

## When the evaluator and the document disagree

A disagreement is reported. It is never resolved silently in either direction.

Where the evaluator runs and produces a different answer from the cached value,
both values are kept, the cell is recorded as disagreeing, and the count of
disagreements is available per document without anybody reading a picture. The
evaluated value is what is displayed, because the alternative is displaying a
number this project believes is wrong.

The reason it is reported rather than merely handled: a disagreement is the
single most informative event this engine can produce. It is either a defect in
this evaluator, or a document whose cache is stale, or a document written by
something that computes differently, and all three are things the corpus should
be able to tell somebody. Issue #51 is where that count becomes the measurement
of this half of the project.

## Precision and rounding

Arithmetic is binary floating point, the same IEEE 754 double the accepted format
stores and the incumbent computes in. Display rounding is applied at display and
never to the stored value, so a value that was read or evaluated survives
unchanged into the model.

No decimal type is invented for this. That is the position and it is worth its
own paragraph, because a decimal engine is the intuitive answer to a spreadsheet
and it is the wrong one here. The values a document carries were computed by an
engine working in binary floating point, so an engine that computes in decimal
disagrees with them in the last bits on arithmetic neither side got wrong. That
turns the disagreement count above from a signal into noise and destroys the one
measurement this project has for whether its evaluator is right. The purpose here
is to render what a document says, not to compute better than the tool that wrote
it.

The rounding position is a display concern, and where it lands is issue #50,
which holds the value types and the coercions with it.

Three things in this section are claims rather than measurements, and they are
what the plan was written against rather than what anything here has checked:
that the accepted format stores a number as a binary floating point double, that
the incumbent computes in the same, and that it rounds a displayed result to
fifteen significant decimal digits with a further correction on a final
subtraction of nearly equal numbers. Issue #51 is what confirms or refutes all
three against documents rather than against memory, and it is worth reading the
disagreement count with that in mind the first time it appears.

## The bound on the function library

The function set the incumbent offers is large and most of it never reaches a
page. This project implements what the corpus needs and refuses the rest by name,
which is issue #49.

The rule by which a function is added is a corpus document that needs it. Not a
list somebody copied from a specification, not a function that seems likely, and
not a batch added because they are in the same family. The issue adding a
function names the corpus document that requires it and the effect on the
measured number, so the function set grows from evidence and can be read back as
a history of what documents actually contain.

What this costs is a project that will not answer yes to "does it support
function X" for a long time. That is the honest state, and it is better than the
alternative, which is a function implemented from a specification, never
exercised by a document, and wrong in the case that matters.

The bound moves in one direction only. A function is never removed because no
current corpus document uses it, since the corpus grows and a removal would make
a previously passing document fail for a reason unrelated to any change.

## What this record does not decide

It does not decide how a formula is parsed, which is issue #45, nor how the
dependency graph is built and ordered, which is issue #47, nor what a cycle does,
which is issue #48, nor the value types and coercions, which is issue #50. Those
implement this decision and each is free to argue with it in its own body rather
than quietly departing from it.

Later calculation issues reference this record rather than restating it. Measured
at the date above, none of them restates the rule about the cached value:

    for n in 45 46 47 48 49 50 51 53; do
      printf '#%s ' "$n"
      gh issue view $n --repo iderex/rechenblatt --json body --jq .body |
        grep -ciE 'cached value|cached-value|last computed value'
    done
    #45 0
    #46 1
    #47 0
    #48 0
    #49 0
    #50 0
    #51 1
    #53 0

Two matches, and neither is a restatement. Issue #46 is a different rule: a
reference into another workbook resolves to the value the document carries and
never opens a file, which is a statement about not touching the filesystem, the
input boundary in `docs/decisions/0014-input-boundary.md`, and it stands whatever
this record says. Issue #51 names the cached value because comparing against it
is the measurement, which is this record being used rather than repeated.

The grep is a coarse instrument and the sentence it supports is narrow: it finds
the phrase, and a restatement written in other words would pass it. What it rules
out is the cheap kind of drift, a later issue repeating this rule in the words
this record uses, and that is all it is offered as.

Nothing refuses a restatement that arrives later. The command above is what a
reviewer runs, and widening it into a check over issue bodies is not something
this tree does today.

## What would reverse this

A measurement showing the cache can be trusted, from issue #51: if corpus
documents disagree with a correct evaluator so rarely that the evaluation is
buying nothing, the cheaper engine is the one that reads and does not compute.

That reversal is bounded even if the number supports it. Conditional formatting
and the macro track both need evaluation of things no cache holds, so what such a
number could retire is evaluating cells that already carry a value, and never the
evaluator itself.

The opposite reversal has a condition too. If the disagreement count turns out to
be dominated by this project's own defects rather than by stale caches, that is
not a reason to trust the cache. It is a reason to fix the evaluator, and the
record says so here so the argument is not available later as a shortcut.
