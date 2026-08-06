# 0001 Means

Rust, one workspace, one toolchain, and no second language anywhere in this tree
without a record of its own.

Status: accepted
Date: 2026-08-06
Issue: #1

## Context

The repository holds documents and workflow guards and no code. Every other issue
in the plan assumes a language without saying so, and an assumption nobody wrote
down is one nobody can disagree with on the merits.

What the work is decides this more than habit does. Three of the four large pieces
take bytes from a stranger and walk them: a zip container of XML parts, a compound
file holding a compiled macro project, and font files. One piece is a layout and
rasterisation engine whose output has to be identical between two runs. One piece
is an interpreter for a language somebody else defined. A memory-safety failure in
any of them is reachable from a document an operator was sent by email, which is
the population this project is aimed at.

## The four questions, in order

### Can the means carry a property a machine can refuse, a proof that runs, and a claim that cites the command behind it

A refusable property needs somewhere for the refusal to live that is not a
sentence in a document. Rust puts three such places in the build itself. A lint
denied at the workspace root fails the build rather than printing advice, which is
how the `unsafe` policy below is held. A dependency edge that crosses a boundary
this project declares is a compile error rather than a review comment, which is
what issue #8 asks for. An architecture rule can be written as a test that fails,
which is issue #101.

A proof that runs is the same harness in every case. The unit and integration
suites, the fuzz targets that issue #96 adds, the mutation run that issue #97
adds, and the coverage floor that issue #5 declares all attach to one test binary
and one command rather than to four apparatuses. That matters more here than the
individual gates do, because a proof nobody can run is a claim.

A claim that cites its command needs commands that produce the same answer twice.
A pinned toolchain and a tracked lock file give that, and the reproducible build
this project needs for its release is the same property stated at the artefact
level.

### Is anything outside this repository forcing it

No. The document formats are open specifications and no external runtime is
imposed by them. Nothing this project must interoperate with dictates what it is
written in.

Two surfaces are forced and neither is a precedent. The workflow files are YAML
with shell inside them because that is what the runner reads, and a container
image will carry a build file in the format the container runtime reads, which is
issue #104. Each is held to its minimum. Neither is a place to put logic that
could live in the workspace, and the shell inside a workflow stays short enough to
read in one screen for the same reason.

### Does it add a language, a runtime or a dependency the tree does not already carry, and is that cost named

It adds all of it, because the tree carries no code at all. The costs, named:

A compile-time tax on a codebase that will grow large. This is paid on every
change and it gets worse rather than better as the workspace fills.

A smaller pool of contributors than a scripting language would draw, and a
language nobody can be assumed to already know. The contributor guide cannot
assume the reader has written Rust before.

A toolchain a packager has to have. A distribution that builds from source with an
older compiler than the pin below cannot build this.

The rendering and text stack this needs is third-party and its licences constrain
what this project may be licensed as. They do not constrain it in a way that
forecloses the open licence question, which is the first entry of issue #111, but
they are a dependency on somebody else's terms:

    for r in harfbuzz/rustybuzz RazrFalcon/tiny-skia linebender/resvg tafia/calamine; do gh api repos/$r --jq .license.spdx_id; done
    MIT
    BSD-3-Clause
    Apache-2.0
    MIT

Those four are the candidates the plan was written against rather than a
dependency list. Which of them this project actually takes is decided by the
issues that need them, under the policy below.

### Is the result testable by the suite that will exist, or does it need a parallel apparatus nobody will maintain

By the suite that will exist. The three gates this project cannot do without are
fuzzing, mutation testing and a coverage measurement, and candidates for all three
exist as cargo subcommands that read the same workspace and the same tests:

    for r in rust-fuzz/cargo-fuzz sourcefrog/cargo-mutants taiki-e/cargo-llvm-cov EmbarkStudios/cargo-deny; do gh api repos/$r --jq '.full_name + " " + .license.spdx_id'; done
    rust-fuzz/cargo-fuzz Apache-2.0
    sourcefrog/cargo-mutants MIT
    taiki-e/cargo-llvm-cov Apache-2.0
    EmbarkStudios/cargo-deny Apache-2.0

That command shows the candidates exist under permissive terms and nothing more.
Which tool each gate uses is chosen where the gate is added, by issues #5, #96,
#97 and #99, and this record does not decide it.

The one place a parallel apparatus is genuinely needed is the harness for what
cannot be pure, which is issue #103, and it is named for what it needs rather than
folded into the default run.

## The toolchain and the minimum version

The toolchain is pinned in `rust-toolchain.toml`, which issue #3 lands, and the
minimum supported version is that pin. There is no second number, because there is
no library consumer to keep on an older compiler: what ships is a binary the
operator does not compile.

The initial pin is the stable release current on the date at the top of this file:

    gh api repos/rust-lang/rust/releases --jq '[.[] | select(.prerelease==false) | .tag_name][0]'
    1.97.1

The edition is declared in the workspace manifest rather than restated here, so
the two cannot drift.

The pin moves in a commit whose message says what needed the newer compiler. It
never moves as a side effect of another change, and it never moves to whatever was
newest on the day somebody looked.

## The dependency policy

`Cargo.lock` is tracked, and every build and every gate runs against it in a mode
that fails rather than updating it. A build that would change the lock is a
finding, not a build. Issue #3 lands the file and issue #99 makes the lock and the
pins a gate.

A new direct dependency arrives with the issue that needs it, and that issue says
what the dependency does, under what licence, and what removing it would take.
Issues #89 and #90 collect those terms into what an operator receives.

A dependency that compiles C or C++ into this binary needs its own record here. It
is not forbidden, because parts of a text stack genuinely are C libraries with
Rust wrappers, and pretending otherwise would push the decision into a build file
where nobody argues it. It is recorded, because it puts unsafe code back on the
path this decision exists to keep safe, and the record has to say why that
particular one is worth it.

`unsafe` in this workspace's own crates is refused by a lint denied at the
workspace root. An exception is a named allow at the narrowest scope with the
reason in the source beside it, and it is visible in the diff rather than in a
build setting. Issues #98 and #101 turn the parts of this that are greppable into
checks.

A dependency update lands as its own change with its own reason. Folding one into
a feature change puts two topics in one commit and hides the one a reader most
needs to see.

## Rejected alternatives

### C++

It is what both incumbent open suites are written in, and choosing it would make
it possible to lift code from them. That is the reason to reject it. Taking code
takes its licence with it, which turns the first entry of issue #111 into a
decision made by accident rather than one somebody made, and it puts an unsafe
language on the path that parses a document an operator was sent.

What would reverse it: nothing short of the untrusted-input work leaving this
process entirely, so that no byte from a stranger is walked by code in this tree.
Even then the licence half of the argument stands on its own and would have to be
answered by the licence decision rather than by a merge.

### Go

Memory-safe, compiles to a single binary, and the cheapest of the four to staff.
It loses on the rendering half rather than on the safety half: the text shaping
and rasterisation available to it is not at the quality this project's whole claim
rests on, and a fidelity project that cannot place a glyph exactly has nothing to
measure.

What would reverse it: a number, not an opinion. If a text and rasterisation stack
in Go scores on this project's corpus what the stack named above scores, the
argument is gone. Issue #33 builds the harness that would produce such a
comparison and issue #38 is where the font and shaping problem is answered, so a
reversal cites those and not this paragraph with the sign changed. The claim that
the Go stack is not at that quality is a claim: it is the reason the plan was
written this way, and no measurement in this repository backs it yet.

### A scripting language

It would make the macro interpreter pleasant to write and everything else worse.
Two conditions the rest of the plan is built on are the ones it costs: two renders
of one document produce identical bytes, which is issue #42, and the release is
one artefact an operator runs with nothing installed beforehand, which is issues
#104 and #106.

What would reverse it: dropping both of those conditions. Dropping one is not
enough, because either alone is sufficient reason to keep the compiled means.

## A second language needs its own record

A second language, runtime, or format carrying its own toolchain does not enter
this tree because it was convenient in the moment. It enters with a record under
`docs/decisions/` that names what forces it and the smallest surface it is held
to, and it is argued there rather than in a pull-request thread.

The two forced surfaces named above already exist. They are written down here so
that a third is not justified by pointing at them.

## How these records are named

`docs/decisions/<number>-<slug>.md`. Four digits, a hyphen, a lowercase
hyphenated slug, `.md`. The angle brackets are what makes this a shape rather
than a file name: `crates/cli/tests/documentation.rs` resolves every path these
documents name, and a placeholder written to look like one would be a claim that
a record called that is there.

The number is assigned once and never reused, and it is not the issue number: the
issue is named in the header instead, so a record and the discussion that
produced it stay linked without the file name having to carry it. Take the first
number no issue has already named, which is read out of the tracker rather than
guessed at:

    gh issue list --repo iderex/rechenblatt --state all --limit 200 \
      --json number,body \
      --jq '.[] | select(.body | test("docs/decisions/[0-9]")) | .number'

The header is the title line, then the decision in a sentence or two, then
`Status`, `Date` and `Issue`. `docs/decisions/0002-track-order.md` is the shape to
copy.

Every record in the directory matches that pattern, and the count of ones that do
not is the check:

    git ls-tree -r --name-only origin/main -- docs/decisions/ | grep -vcE '^docs/decisions/[0-9]{4}-[a-z0-9-]+\.md$'
    0

That is a command a reader can run rather than a list of file names, which would
drift the first time a record is added. Nothing refuses a name that breaks the
pattern today; the command above is what a reviewer runs, and turning it into a
gate is part of issue #100.
