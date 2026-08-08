# Code scanning

The target gate this project is held to requires a code scanning analysis, and
`docs/quality-parity.md` maps that row onto a gate for this language. This is
that gate: what it reads, what it refuses, why the analyser is the one it is, and
the part a green run does not cover.

## What it reads and what refuses

    bash .github/scripts/prove-code-scanning.sh
    bash .github/scripts/check-code-scanning.sh .

The first plants each construct and requires the refusal. The second is the gate.
Both need cargo and nothing else, and both run offline, so the check a
contributor runs is the check the workflow runs rather than a description of it.

The lints are in `.github/scripts/code-scanning-lints.txt`, one record per lint
with its severity and what it prevents. The checker holds none of them, so adding
a lint is adding a record and a leg. The threshold is one line in
`docs/quality-parity.md`, beside the reasoning for the rung, and the checker holds
no default for it: a run that cannot read the line stops rather than choosing one.

Two refusals.

`finding-at-or-above-the-threshold` is the analyser reporting a construct the
register places at or above the threshold, named with the file, the line and the
lint. A construct below the threshold is reported and uploaded and fails nothing,
which is what makes the threshold a decision rather than a word.

`suppression-without-a-reason` is a tracked source silencing a lint with no
`reason = "..."` in the attribute. A scanner without this accumulates silent
suppressions and stays green while covering less every month, and the reason is
the one thing a reviewer needs that the attribute does not otherwise carry.

Exit 2 is the check stopping rather than judging, and it is a failure and never a
pass: no register, a register holding a line no field name starts, a record
naming no lint or carrying a severity outside the vocabulary, a threshold the
parity document does not declare or declares twice, and no cargo on the path.

## Why clippy, and not the other two answers

The means was chosen for this artefact rather than carried over, and the
alternatives were real ones.

An external analysis service is what the target gate uses, and it would be the
closest match by name. It cannot be run at the commit being pushed on the machine
pushing it, so its proof leg - plant the construct, watch the gate go red - could
only ever be run on a server, and a guard whose proof nobody can execute locally
is a guard this repository has no way to show is biting. That is the first of the
three rules this project holds, and it is the one that decided this.

A second analyser installed from a package index would add a version that moves
without a commit saying so, a network fetch inside a gate, and a build script
from outside this tree running in the job that judges this tree.

Clippy is pinned in `rust-toolchain.toml` beside the compiler, so it arrives with
the toolchain a fresh clone already fetches, it runs offline, and its version
moves only in the commit that moves the pin. The SARIF this gate uploads is
written by the checker itself rather than by a converter fetched at run time, for
the same reason.

The cost is real and worth stating. Clippy is a lint pass over one function at a
time, not an analysis that follows a value from where it entered the process to
where it is used. It refuses the abort that is written down; it says nothing
about the one three calls away, and nothing at all about whether the value came
out of a document. An analyser that follows values would say more, and issue #95
is where a second lens is added rather than this one being replaced.

## This is not the lint gate run twice

The `Lint` job denies the compiler's and the linter's default set over every
target. Every lint in this register is off by default, so that job reaches none
of them, and the two gates cannot both go red for one reason.

The questions differ as well. That gate asks whether the code is correct and
idiomatic. This one asks a narrower thing: what a value that came out of somebody
else's document can do to the process that read it. That is why the register is
almost entirely aborts, wrapping arithmetic and unchecked indexing, and why a
lint about style would not belong in it however useful it is elsewhere.

## Where this stops

**It reads `--lib` and `--bins` and not the tests.** A panic in a test is a
failing test; a panic in the shipped code is the failure this register is about.
So a test may `unwrap` and this gate says nothing, which is deliberate. The
`Lint` job reaches every target, and
`.github/scripts/prove-code-scanning.sh` has a leg that plants the same abort in
an integration test and requires this gate to pass, so the bound is proved rather
than asserted here.

**It cannot tell a value from a document from any other value.** Every lint here
is syntactic. `clippy::indexing_slicing` refuses an index into a slice whether
the index came out of a package header or out of a constant three lines above,
and the second is why the low rung exists rather than a fourth refusal.

**It cannot judge a reason.** The suppression check reads whether the attribute
carries one, not whether it is true, and a suppression whose reason is wrong
passes. That is what the review is for.

**Its scan of suppressions is a read of text.** It matches `#[allow(` and
`#[expect(` in tracked Rust and follows the attribute to its closing bracket, so
the same spelling inside a string or a comment is counted, and a lint silenced by
something other than an attribute is not seen at all.

**A green run today says very little.** The tree holds six libraries whose whole
content is a header comment and a constant, so the scan reads almost nothing and
finding nothing is what it would do either way:

    cargo clippy --locked --workspace --lib --bins -- -A warnings -D clippy::unwrap_used

What the gate can be said to do today is refuse each construct in the register
when one is planted, which is what the proof runs and what makes the gate worth
having before the readers arrive rather than after. It is not evidence that the
shipped code is free of these constructs, because there is barely any shipped
code. That sentence stops being true in the milestone that lands the first
parser, and until then a green run here is a statement about the gate and not
about the software.
