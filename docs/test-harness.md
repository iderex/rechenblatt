# The test harness

Every later issue in the plan ends in a test, so this is what those tests live
in. It is written before them rather than extracted from them afterwards.

## The one command

```
cargo test --locked --workspace
```

That is the whole default suite. Nothing else has to be started first and nothing
has to be installed beyond the pinned toolchain. `CONTRIBUTING.md` names the same
command beside the guards that read the tracked tree.

The default suite is pure. No display server, no elevated rights, no host font
directory and no network. Anything that needs a real environment goes to a
separate harness named for what it needs, which is issue #103, and is not part of
this run. Issue #7 is where that condition becomes a check rather than a sentence.

## Where a test goes

A test about one crate goes in that crate, next to what it tests. A test about the
tree, or about a file the tree carries, goes in that crate's `tests/` directory as
an integration test, because it exercises the crate the way something outside it
would.

`crates/model/tests/fixture_registry.rs` is the shape to copy for the second kind.
It reads a directory, returns everything wrong with it as values, and then has one
test per thing that can be wrong. That split is what makes the legs below possible:
the rule is a function, so a test can hand it a directory built for the occasion
instead of waiting for the real tree to be wrong.

## What a failing test prints

Enough to find the cause without running it again under a debugger.

Name the thing, not the comparison. `sheet.xlsx has no record beside it` locates
the problem; `assertion failed: found.is_empty()` does not, and neither does a
diff of two vectors the reader has to decode. Where a check produces a list, print
the list, one item per line, each naming its own path.

Say what the repair is where there is an obvious one. The registry's messages end
in the sentence that fixes them, because the person reading it is usually somebody
who has just added a file and does not yet know the rule.

The rule is a convention and nothing refuses a message that ignores it. It is
written down because a message is easy to write badly under time pressure and
impossible to notice afterwards, since a test that passes never prints it.

## Fixtures

Fixture documents live in `tests/fixtures/` and nowhere else. Every one of them
has a record beside it under the same stem, saying what it is for and where it
came from. `tests/fixtures/README.md` is the rule as a contributor meets it.

The rules are not cosmetic here. The bytes that reach a parser have to be exact,
so a fixture is a binary file and everything about it that a reviewer needs is in
a file beside it rather than in the file itself. What it is for decides whether a
change may alter its expected output. Where it came from decides whether this
repository may hold it at all, which for a real document is a licence question and
a personal-data question before it is a technical one.

A fixture with no record fails the run. This is the part worth arguing for: the
easy alternative is to skip it, and a skipped fixture leaves the suite green
whether or not anybody ever looks at the file again.

`crates/model/tests/fixture_registry.rs` refuses four things: a fixture with no
record, a record with no fixture, a record missing either required line, and a
subdirectory, which would hold fixtures the walk never sees. Each refusal has a
leg that builds a directory holding exactly that mistake and requires that
refusal and no other, and a neighbouring leg that changes the one thing back and
requires no refusal at all. A check that refuses everything fails the second kind
of leg; one that refuses nothing fails the first.

The scratch directories those legs use are built and removed by the test itself
rather than by a crate brought in for it. `docs/decisions/0001-means.md` says a
new dependency arrives with the issue that needs it, and twenty lines of
`std::fs` is not that issue.

## The coverage floor

Coverage floor: 80

That line is read by `.github/workflows/suite.yml` rather than the number being
written into the workflow, so the floor and the reasoning for it cannot drift
apart. A run that cannot read the line fails rather than assuming a default.

The measurement, at the commit this floor was set:

    cargo llvm-cov --locked --workspace --summary-only
    Filename                Regions  Missed Regions  Cover  Functions  Missed Functions  Executed  Lines  Missed Lines  Cover
    crates\cli\src\main.rs       57              10 82.46%          7                 1    85.71%     40             6 85.00%
    TOTAL                        57              10 82.46%          7                 1    85.71%     40             6 85.00%

Eighty-five measured, eighty declared. The five points are not slack for untested
code; they are there because the workspace is nearly empty and one added line
moves this number several points, and a gate that reds for arithmetic rather than
for test quality is one people learn to ignore.

The six uncovered lines are `main` itself, which prints three lines and exits. A
unit test cannot reach it and inventing one that could would raise the number
without testing anything. Issue #77 gives that binary something worth covering.

The floor is a floor. It is raised in a commit whose message says what raised it,
and it is never lowered to make a change fit: a change that would cross it
downward is a change that arrived without its tests. It is also a poor proxy for
whether the tests are any good, which is what issue #97 measures with mutation
testing and what should be believed over this number where the two disagree.

The number will be re-argued when the first component with real code in it lands.
A percentage over one forty-line file is a measurement of that file and not of
this project.

## What the gate runs

`.github/workflows/suite.yml`, on every branch and every pull request. It builds,
runs the suite, and then measures coverage against the floor above. All three with
`--locked`, so a run that would change the lock file fails rather than updating
it.

The coverage tool is compiled on the runner from a pinned version rather than
pulled in as a third-party action, which keeps the gate first-party in the same
way the sign-off gate is. It costs a couple of minutes per run and that is the
whole cost.
