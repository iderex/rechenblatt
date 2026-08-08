# The invariants a reading of the tree can refuse

Some rules here are text facts. A source file either names a host font directory
or it does not, and deciding which needs no judgement about what the code means.
A rule of that shape does not have to wait for somebody to notice it in a diff,
and one that does wait is a rule that holds until the first busy afternoon.

This is the gate for those, and only those. What it covers is narrow on purpose:
the rules that are not text facts are named at the bottom of this document as
the rules they are, rather than left looking like they are covered here.

## The list is data

`.github/scripts/invariants.txt` holds the invariants, one record per invariant.
`.github/scripts/check-invariants.sh` applies them and carries no pattern of its
own. Adding an invariant is writing a record and a leg; it is not editing a walk.

Two reasons, both about the check rather than about taste.

A reviewer reading a new invariant should be reading the invariant. In a record
the id, the files it reads, the sentence saying what it prevents and the pattern
sit together in four lines, and a reviewer who disagrees with the pattern is
disagreeing with something they can see all of.

And a checker whose own source held the patterns would have to be taught to skip
itself, which is a hole in the shape of a rule. The list is not a subject of any
invariant either, for the same reason and by the same mechanism: the walk never
judges the file that declares it, and a leg in the proof holds it to that even
when a record's globs would otherwise reach it.

What the list holds today is printed rather than written here, because a list in
a document drifts against the thing it describes:

    sed -n 's/^Id: //p' .github/scripts/invariants.txt

And what each one prevents, in the sentence the record carries:

    sed -n 's/^Prevents: //p' .github/scripts/invariants.txt

## What a record says

Four required fields and one optional one. The required four are the id a refusal
prints, the shell globs saying which tracked paths the invariant reads, the
sentence saying what failure it prevents, and the extended regular expression.
A record missing any of them stops the run: a list read as three invariants where
the file holds four would report a clean tree it never fully examined, and that
is worse than a red one.

Nothing is scanned by default. Every invariant says which files it reads, so
widening one is a visible change to that record rather than a side effect of
adding a pattern. Documents are outside every glob in the list today, and that is
deliberate: a document has to be able to name a host font directory in order to
say that one is refused, and a check that refused that sentence would be refusing
its own disclosure.

The optional field is an exception, a tracked path prefix and the reason it is
excepted. It is how a file that is genuinely ABOUT a pattern gets to name one,
and the second kind of case turned up while this gate was being written: the
proof plants a host font directory in a scratch tree and requires the refusal, so
it cannot do its job without holding the string it is about. What is excepted and
why is printed rather than restated here:

    sed -n 's/^Except: //p' .github/scripts/invariants.txt

The register fails closed in both directions: an exception with no reason is
refused, and an exception whose prefix matches no tracked path is refused as
well, so an exception cannot outlive the file it was written for. Both directions
have a leg. Every run prints each exception it honoured, so a green run cannot be
read as a tree with none.

## What a whole-file exception costs

An exception is a prefix, so it can name a directory, a file, or everything under
either. `crates/model/tests/environment.rs` is excepted from two invariants as a
whole file, and that is wider than a prefix naming one line could be, because the
register has no such prefix. The cost is exact and worth stating: inside that
file, neither of those two invariants refuses anything ever again. A socket
opened there for a reason that is not the probe, and an absolute path written
there by somebody who has forgotten what the file is for, both pass.

It is still the right shape rather than a hole. The file is a scan for host font
directories and four probes that each reach for something a sealed environment
does not have, so every string those invariants match is the subject of the test
holding it. A pattern cannot tell a subject from a dependency, and neither can a
narrower prefix. What holds that file instead is
`.github/scripts/run-sealed.sh probes`, which requires every one of those probes
to FAIL in the sealed environment and requires each failure to carry a cause, so
a probe that stopped reaching for the thing it names stops passing that gate.

The general rule this is the first instance of: an exception is granted where the
pattern's match is the test's subject, and the reason field says which subject.
An exception granted because a rule was inconvenient has no such sentence, and
that is what a reviewer is checking for.

## Running it

    bash .github/scripts/prove-invariants.sh
    bash .github/scripts/check-invariants.sh .

`.github/workflows/invariants.yml` runs those two, in that order, under the check
name `Enforce greppable invariants`. The order is the same one the other guards
that read the tracked tree use: a checker that has stopped biting reports a clean
tree, so the proof that it still bites runs first.

The proof builds a repository per leg, holding exactly the violation that leg is
about, and requires the checker to refuse that id and no other. A neighbouring
leg changes the one thing back and requires no refusal at all. A checker refusing
everything fails the second; one refusing nothing fails the first.

The violations are written by the proof rather than committed as fixtures. A
tracked file holding a host font path is exactly what this gate refuses, so a
fixture of one would either red this repository or need the exception it exists
to test.

The lists those legs are judged against are written in the proof as well, and not
read from the real one. A leg that reads the real list proves the state of the
tree on the day it ran; these prove the checker.

## Where this instrument stops

It reads text and it matches patterns, so everything below is outside it and
saying so is the point of this section.

It cannot tell a name from a type. `secret-in-a-plain-string` reads the shape
`name: String` and decides on the name, so it is a rule about how a field is
spelled. The pattern was narrowed once already for exactly that reason: `token`
on its own matched a parameter holding a backticked token in this repository's
own documentation check, which is a false refusal, and a rule that refuses valid
work is one people silence rather than obey. It now requires a qualified spelling
and a leg holds it there. A credential in a field called `t` passes.

It cannot tell where a value came from. That is what keeps the first rule named
for this gate out of the list, below.

It reads the working tree rather than the index, unlike the tracked-bytes
checker beside it. That is not an oversight: this one is about what the text
says, and a pattern reads the same either way.

A pattern is a floor. It holds the spellings somebody has actually written and
it will not catch one nobody has written yet, which is the same bound every
vocabulary of this kind has.

## The two rules named for this gate that it does not refuse

Both were named as invariants for this check. Neither is one, and each is written
here and in `docs/quality-parity.md` as the rule it actually is, rather than left
in the list as a rule nothing refuses.

Whether a value interpolated into a log line came from a document is a property
of where that value came from. No reading of the text decides it: the same
format string is correct or a disclosure depending on what was bound to the
variable three functions earlier. The enforceable neighbour is a format string
that is not a literal, and that is a different rule from the one that was named,
so it is not quietly substituted here. Issue #81 is where what a log may contain
is decided.

A fixture without a provenance record is refused, and not by this gate.
`crates/model/tests/fixture_registry.rs` refuses it in the suite, and
`tests/fixtures/README.md` is where a record's shape is written. Adding a second
refusal here would give one rule two places to be repaired and two places to
disagree.
