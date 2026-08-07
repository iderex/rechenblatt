# What cannot be pure, and where it goes instead

The default suite is pure. No display server, no elevated rights, no host font
directory, no network, and `docs/test-harness.md` argues why and names the one
command that runs it.

A few things cannot be proved under that rule at all. Producing a reference
rendering with software that has to be installed, comparing against a suite
somebody else ships, checking a container on a real host, verifying a signed
artefact: each of those needs an environment, and each of them arrives wanting to
be a test. Letting one in is how the pure suite gets quietly polluted. It passes
on the machine it was written on, and the first contributor without whatever it
silently wanted gets a failure that names nothing.

So they go somewhere else, and the somewhere else is named for what it needs.

## The name is the point

`.github/scripts/needs-an-environment.sh` is the runner. The file is named for
its requirement rather than for its function, because the name is what somebody
scanning the scripts directory reads before they read anything else. A file
called `integration.sh` or `e2e.sh` says what the author had in mind. This one
says what the reader has to have.

It runs nothing without being told which entry to run:

    bash .github/scripts/needs-an-environment.sh

prints the register and exits non-zero, having run nothing at all. There is no
`all`, and that absence is deliberate rather than unfinished. Two entries can
need different environments, and a runner that took them together would report
one verdict over two requirements, which is the shape of a result nobody can act
on.

To see what is there and what each one wants:

    bash .github/scripts/needs-an-environment.sh list

## The register is data

`.github/scripts/needs-an-environment.txt` holds the entries, one record each.
The runner and the checker both read it and neither holds an entry of its own, so
adding one is writing a record and a leg in the proof. That is the same shape
`.github/scripts/invariants.txt` uses, and it is chosen for the same reason: a
reviewer reading a new entry should be reading the entry, not a diff of a walk.

Four fields, all required.

`Id` is the name the runner is invoked with and the name a result carries.

`Needs` is the environment, in the terms somebody reproducing it has to satisfy.
It is not a label. A result from this harness is only worth what somebody else
can reproduce, and this field is the whole of what they are given.

`Because` is why the entry cannot be proved in the pure suite, in one sentence.
This is the field that keeps the register from becoming the place awkward tests
are put. A test that is here because it is slow, or flaky, or annoying to write
purely, has no sentence that survives being read out loud.

`Run` is the command, from the repository root.

What the register holds today is printed rather than written here, because a list
in a document drifts against the thing it describes:

    sed -n 's/^Id: //p' .github/scripts/needs-an-environment.txt

And what each one needs, in the record's own words:

    sed -n 's/^Needs: //p' .github/scripts/needs-an-environment.txt

## Every result carries its environment

A number from here is not a number from the default suite, and the two are easy
to paste side by side once they are both text in a terminal. So the runner wraps
every run in a record: the entry, what it needed, why it is here, the commit, the
host, and the time it started. Every line it prints begins with
`needs-an-environment:`, and one of those lines says in words that the result may
not be quoted as a result of the default suite.

The record is printed before the command rather than after it, so a run that is
killed halfway still leaves behind the environment it was killed in.

Where it stops: the record goes to the run's output and nothing in this
repository stores it. A result that somebody wants to keep is kept by keeping the
block, and there is no mechanism that makes them. That is a real residual and it
is not softened here.

## What the checker refuses

`.github/scripts/check-needs-an-environment.sh` reads the register and refuses
four things. It runs in the pure suite's own gate, on every machine, because the
entries need an environment and the rule about the entries does not.

`entry-without-a-need` is an entry with no `Needs`, so a result from it cannot be
reproduced by anybody.

`entry-without-a-reason` is an entry with no `Because`, so nothing says why it is
out of the pure suite.

`entry-names-no-script` is an entry whose `Run` names a path that is not in the
tree, which is an entry that stopped being runnable without anything going red.

`duplicate-entry-id` is two entries under one id, where the runner takes the
first and the second would never run again.

Four other shapes stop the run rather than being refused, because a register the
checker could not read is not a register it can report as clean: no register at
all, a line no field name starts, a file with no entries in it, and a record with
no `Id` or no `Run`. An entry the runner can neither name nor execute is not an
entry, and judging the rest of the file around one would report a clean register
that is not one.

Each of those has a leg in `.github/scripts/prove-needs-an-environment.sh` that
builds a register holding exactly that mistake and requires that refusal and no
other, and a neighbouring leg that changes the one thing back and requires no
refusal at all. A checker that refuses everything fails the second kind; one that
refuses nothing fails the first.

## What this cannot judge, and it is the larger half

Whether an entry is here BECAUSE it genuinely cannot be pure is a judgement about
the thing the entry runs. No reading of the register makes it, and a `Because`
line saying something false passes exactly like a true one. What is checkable is
that somebody wrote the sentence down where a reviewer meets it. The review is
where a wrong one is caught, and this paragraph is here so that a green check is
not read as more than it is.

The same holds one step out. Nothing compares the register against the set of
things that ought to be in it, so an impure test that never got a record is
invisible to every route here. Issue #102 is where the default suite's own
exclusion set is decided, and it is the half that would catch that.

## It is not a merge condition

Requiring any of this in front of the default branch would fail the third of the
three conditions in `docs/required-checks.md`: an entry needs an environment, and
a change that did not touch anything reddens when that environment is missing or
slow or down. So the harness stays beside the gate rather than inside it, and
`docs/quality-parity.md` records the same thing where the parity map is argued.

The register's checker and its proof are a different matter. They read a file and
need nothing, which is why they run in the ordinary gate under the name
`Declare what needs an environment`, and `docs/checks.md` carries what reproduces
it.

One entry is also run by a job of its own, `Headless and unelevated`, on a runner
that does have a container. That job goes through this runner rather than around
it, so the route a contributor is told to use is the route the gate proves.
Whether that job becomes a merge condition is `docs/required-checks.md`'s
question and issue #93's, and it is not decided here.
