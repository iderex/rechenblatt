# What the default run does not run

The default run is one command, and `CONTRIBUTING.md` is where a contributor
meets it:

    cargo test --locked --workspace

A test carrying `#[ignore]` is not in that run. The attribute is one line, it
takes a test out of every green run anybody looks at, and until this register
landed nothing in the tree read it. So the count of tests drops by one, nobody
reads a count, and the change that did it is a single word inside a diff about
something else.

That is not a hypothetical shape. `docs/needs-an-environment.md` names the same
gap from the other side: nothing compared the harness register against the set of
things that ought to be in it, so a test that left the default suite and reached
no harness was invisible to every route here.

## The two registers are a pair

`.github/scripts/needs-an-environment.txt` says what cannot be proved in the
default suite and what runs it instead.
`.github/scripts/excluded-from-the-default-run.txt` says what was taken out of the
default run and which of those entries runs it.

They answer different questions and the interesting failure is between them. A
test excluded from the default run whose `Runs-in` matches no entry in the
harness register is excluded from both, which is a test nothing runs anywhere. It
still compiles, it still sits in the tree looking like coverage, and it has
stopped being a test. That is the failure this register exists against, and it is
`exclusion-runs-nowhere` below.

## The register is data

One record per test. Four fields, all required.

`Test` is the function's name, which is what `cargo test -- --exact` takes and
what a failure reports.

`In` is the source it is written in, from the repository root. It is what lets
the checker read the two sides against each other rather than trusting the
register.

`Because` is why it is not in the default run, in one sentence. This is the field
that keeps the register from becoming the place awkward tests are put. A test
that is here because it is slow, or flaky, or annoying to write purely, has no
sentence that survives being read out loud.

`Runs-in` is the `Id` of the harness entry that does run it.

The current contents of the register are printed by a command, and this document
does not copy them out, because a list in a document drifts against the thing it
describes:

    bash .github/scripts/check-excluded-from-the-default-run.sh .

That prints every excluded test with the source it is in and the entry that runs
it, and then judges. It is the command the set is printed by, and it derives the
list from the tree as well as from the register, so a set that has fallen behind
the sources cannot print as though it had not.

## What the checker refuses

`.github/scripts/check-excluded-from-the-default-run.sh` reads the register, the
harness register and every `.rs` file in the tree. It runs in the ordinary gate on
every machine, because it reads files and needs nothing at all - the tests it is
about need an environment, and the rule about them does not.

`exclusion-without-a-reason` is a record with no `Because`, so nothing says why
the test is out of the default run.

`exclusion-runs-nowhere` is a record whose `Runs-in` is empty or names an entry
the harness register does not hold, which is the pair above coming apart.

`record-names-no-test` is a record whose `In` holds no `#[ignore]`d function of
that name. It catches a path with a letter wrong, a renamed function, and the one
worth naming: a test that came BACK into the default run and left its record
behind. A register that only read itself would report that as clean.

`test-excluded-without-a-record` is an `#[ignore]`d function in the tree that no
record names. This is the direction that stops the set falling behind the code,
and it is the one that fires on the commit that adds the attribute rather than
weeks later.

`duplicate-exclusion` is two records for one test name, where the second says
nothing the first did not while being able to disagree with it.

Five other shapes stop the run rather than being refused, because a register the
checker could not read is not one it can report as clean: no register at all, a
line no field name starts, a record with no `Test`, a record with no `In`, and a
missing harness register - `Runs-in` cannot be judged against a register that is
not there, and half a judgement reported as a clean one is the single output this
check may not produce.

An empty register over a tree that excludes nothing is NOT one of those. It
passes, and that is deliberate. A check that treated an empty register as
unjudgeable would make the honest state the red one and would teach somebody to
write a record for a test that does not exist.

Each refusal has a leg in `.github/scripts/prove-excluded-from-the-default-run.sh`
that builds a tree holding exactly that mistake and requires that refusal and no
other, and a neighbouring leg that changes the one thing back and requires no
refusal at all. A checker that refuses everything fails the second kind; one that
refuses nothing fails the first. That proof is pure: it compiles nothing and runs
no cargo, because what is under test reads two files and a directory of sources.

## Where this stops, and it is the larger half

IT READS `#[ignore]` AND NOTHING ELSE, because that is the mechanism this
repository takes a test out of the default run with. There are other ways to be
outside that run and this check sees none of them: a test behind a `cfg` feature
nothing enables, a test in a file no module declares, a test whose harness is not
registered in a manifest. Each of those is a test the default run does not run
and this register does not know about. Nothing here refuses one, and a reader who
takes a green run as "every test in the tree is either run or recorded" is taking
it as more than it is.

It cannot judge whether a `Because` is true. A sentence saying something false
passes exactly like a true one. What is checkable is that somebody wrote it down
where a reviewer meets it, and the review is where a wrong one is caught.

It cannot judge whether the entry named by `Runs-in` really runs the test. It
checks that the entry exists. `sealed-probes` runs every ignored test in the
workspace, so today the two coincide, and an entry that ran only some of them
would satisfy this check while leaving the rest unrun.

It says nothing about a test that needs a display, a network route, host fonts or
elevated rights and was NOT excluded. Such a test runs in the default suite and
fails there on its own assertion rather than at collection, and the failure names
whatever the assertion happened to say. Issue #102 asks for that failure to name
the cause instead, and this register is not it: no reading of a source tells you
that a test needs a display until it runs. The one thing in the tree that refuses
a source for reaching outside the sealed environment is the font scan in
`crates/model/tests/environment.rs`, which covers fonts and not the other three.

## It is a merge condition candidate, not a merge condition

Like every other check here it is advisory today.
`docs/quality-parity.md` carries the rule for which checks belong in front of the
default branch and `docs/required-checks.md` is where that is decided. This one
reads files, needs nothing, and cannot be red for a reason outside the change,
which is the shape those documents call eligible. It is not in the required set
they name, and nothing is required by the ruleset today in any case.
