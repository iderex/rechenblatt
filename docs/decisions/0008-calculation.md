# 0008 Calculation

This project evaluates formulas rather than trusting the values a document
carries, uses the carried value where it is present and consistent, always
evaluates what the page depends on, and records for every displayed value which
of the two produced it.

Status: accepted
Date: 2026-08-08
Issue: #44

## Context

A workbook usually carries the last computed value of every formula cell beside
the formula itself. A renderer can read those values and never evaluate
anything. That is fast, it is correct for a file the incumbent suite last saved,
and it is wrong for a file written by a tool that left the cache empty, or stale,
or filled with what a different function library thought.

It also does not survive the rest of this plan. Conditional formatting rules are
formulas evaluated when the page is drawn, so a renderer that never evaluates
cannot paint the feature the gap analysis names first. The macro track changes
cells and needs the consequences of the change, and no cache in a file knows
about an edit made after it was written.

The opposite position is no better. Evaluating everything and ignoring the file
means every rendering waits on a full recalculation, and it means a document
whose formulas use a function this project has not implemented renders worse than
it would have if the answer already in the file had simply been read.

## The decision, in full

Build the evaluator. It lives in `crates/calc`, it reads the model and nothing
else in this workspace, and the model never computes: what a document says and
what a document's formulas mean are two questions, and
`docs/decisions/0003-workbook-model.md` is why the first one has a single
answer.

### The rule for a carried value

Where a formula cell carries a value, that value is used, provided it is
consistent. Consistent means all of:

- the document does not ask for a full calculation when it is opened,
- the cell's formula is not volatile, so its result does not depend on when it
  is asked,
- every cell the formula depends on is itself either a constant or a formula
  cell with a consistent carried value,
- and nothing in this session has changed a cell the formula depends on.

The third condition is what makes the rule closed under dependency: a value
inherited from a precedent that had to be evaluated is an evaluated value, and
calling it a carried one would be the quiet half-truth this whole record exists
to avoid.

### When evaluation is forced

- A formula cell with no carried value.
- Any cell that fails the consistency rule above.
- Every value a display decision depends on: the condition of a conditional
  formatting rule, a formula behind a data bar or a colour scale threshold, a
  defined name a displayed formula resolves through. These are evaluated at
  display time whatever the file carries, because the file carries the result of
  the cell and not the result of the rule. Issue #53 is where the rules reach
  the evaluator.
- Everything downstream of a change. When the macro track writes a cell, every
  cell that depends on it is evaluated, and their carried values are discarded
  rather than reconciled.

### How the origin of a value is recorded

Every displayed value carries which of three things produced it:

- **read**, taken from the document,
- **evaluated**, computed here,
- **unavailable**, neither: the formula uses something this project does not
  implement and the document carried nothing to fall back on.

This is a property of the value, not a line in a log. A fidelity difference in a
cell is then attributable before anybody opens a debugger: a difference on a
**read** value is a rendering or formatting difference, and a difference on an
**evaluated** value is a difference in this project's arithmetic. Guessing which
of those two it is has a cost that recurs on every difference, which is what
paying for the field once buys.

**unavailable** is never rendered as zero, as blank, or as an error the document
did not contain. It is a value that says what is missing, it is an entry in the
report, and issue #64 is where the same principle governs what the renderer will
not draw.

### When the evaluator and the carried value disagree

Both exist and they differ: that is a reportable event and never a silent
choice. The value carries the disagreement, with the cell, the formula, both
values and the comparison that separated them, and the report counts them.

The displayed value in that case is the carried one, and the reason is narrow:
the reference renderings the corpus is measured against were produced from the
file, so displaying this project's own answer instead would turn every
arithmetic difference into a rendering difference and hide both. The
disagreement is not suppressed by that choice; it is reported, which is the
opposite.

Comparing the two costs a full evaluation of a document that did not need one, so
it is what the measurement harness does on every corpus document rather than
what the operator surface does on every request. Issue #51 is that measurement,
and it is the number that says how much of this project's arithmetic agrees with
the arithmetic already in circulation.

### Precision and where rounding happens

The arithmetic is binary floating point, in the double precision every
implementation of this format uses, and not an invented decimal type.

The reason is measurement rather than taste. Every carried value in every corpus
document was produced by binary floating point, so a decimal evaluator would
disagree with the corpus in the last digits of a large fraction of cells while
being, in some abstract sense, more correct. This project's claim is about
matching documents, and the arithmetic that produced those documents is the
arithmetic to use.

Rounding is display rounding and it happens at the display boundary. The
evaluator returns the unrounded result; the number format decides what is shown,
including the fifteen significant decimal digits the incumbent shows for a
general number and the rounding a format applies on top of that. An evaluator
that rounds inside itself makes every later sum wrong in a way nobody can trace,
and that is expensive enough to settle before the first function is written.

The incumbent's own departures from the format - its treatment of certain near
integers and its date arithmetic including the day that does not exist - are
compatibility behaviour rather than arithmetic. They are implemented where they
are met and each one is a named case with a corpus document behind it, rather
than a general licence to be approximately right.

### The bound on the function library

The function set of the incumbent is enormous and most of it never reaches a
page. This project implements what the corpus needs.

A function is added when a corpus document needs it. That is the whole rule, and
it means the set grows from a measurement rather than from a list somebody
copied out of a reference. Issue #49 is where the set is implemented and where
what is not implemented is refused loudly rather than silently returning
something plausible.

An unimplemented function does not produce an error value the document could
have contained, because a reader cannot tell that apart from a document that
really holds one. It produces **unavailable**, it names the function, and the
report counts the name so the next function to implement is the one the corpus
asks for most.

## What this costs

**Two answers to keep straight.** Every value has an origin and some values have
two candidates. That is a field on every value, a branch in the reporting, and a
sentence in every document that quotes a number.

**A full evaluation on the measurement path.** Comparing against carried values
means evaluating documents that did not need evaluating, on every corpus run.
That is bought deliberately: the comparison is the only thing that says whether
the evaluator is right.

**The displayed value is the file's, so this project's own arithmetic is not
what a reader sees.** A user who wants this project's answer to a formula gets
it from the report rather than from the page. That is the honest trade for a
fidelity number that measures rendering rather than arithmetic, and it is
revisited if the two are ever measured to agree closely enough for it not to
matter.

**Binary floating point brings its own surprises.** Sums that do not associate
and a general format that hides the last bits are inherited rather than solved.
The alternative was disagreeing with every document in the corpus.

## What would reverse this

A measurement, from issue #51, showing that carried values in real documents are
so often stale or absent that the carried-value rule is a fast path nobody takes.
Then the rule inverts: evaluate always, and the carried value becomes only a
comparison.

The other direction reverses it too. If evaluation turns out to be far more
expensive than the plan assumes on documents of the size operators actually have,
the forced-evaluation set shrinks to what display strictly requires, and the
record says so with the timing that showed it.

Either way, reversing means naming the measurement and its date. Not restating
this record with the sign changed.

## Rejected alternatives

**Trust the file and never evaluate.** The fastest thing that could work, and it
is what a renderer built only for files the incumbent saved would do. It cannot
paint a conditional format, it cannot survive a macro changing a cell, and it
renders a file with an empty cache as a page of blanks while reporting nothing
wrong. It is also unfalsifiable: with no evaluator there is no second opinion, so
a document whose cached values are wrong renders wrongly and passes every check
here.

**Evaluate everything and ignore what the file carries.** Clean, and it throws
away the best available check on this project's own arithmetic. It also makes an
unimplemented function fatal to a cell that already had a perfectly good answer
in the file.

**A decimal arithmetic type.** More correct in isolation and wrong for this
purpose, because it disagrees in the last digits with every document the corpus
holds. It is worth revisiting only if a corpus measurement shows the documents
themselves were produced by something other than binary floating point, which is
a claim no reading of the format supports today.

**Implementing the incumbent's function list.** Years of work, most of it for
functions no page has ever displayed, and it produces a project that is one
release behind a moving list forever. The corpus decides the set instead, which
is the same principle the rest of this plan runs on.
