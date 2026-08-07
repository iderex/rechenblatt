# Contributing

## Every change starts as an issue

Planning happens on the tracker before the code that depends on it exists. Open an
issue, get the shape of the change agreed there, then send a pull request that
closes it. A pull request that arrives without an issue behind it is a change
whose reasoning is only in the diff, and the diff is the worst place to keep it.

## From a clone to a green run

```
git clone https://github.com/iderex/rechenblatt.git
cd rechenblatt
```

Build the tree and run the suite:

```
cargo build --locked --workspace
cargo test --locked --workspace
```

`--locked` belongs to both commands. It makes a build that would change
`Cargo.lock` fail rather than update it, so the dependency set is the same here as
it was on the machine that built this commit before you. The compiler version is
pinned in `rust-toolchain.toml` and rustup fetches it on the first build.

`docs/test-harness.md` is where a test goes, what a failing one has to print, where
fixture documents live and what has to be beside them, and the coverage floor with
the reasoning for the number. Read it before writing the first test in a new
crate.

The guards below read the tracked tree rather than the code, and they need git,
bash and iconv and nothing else.

Reject bidirectional and invisible Unicode, the same expression the
`unicode-guard` workflow runs:

```
git grep -nIP '(*UTF)[\x{202A}-\x{202E}\x{2066}-\x{2069}\x{200E}\x{200F}\x{061C}\x{200B}-\x{200D}\x{2060}]' -- .
```

Any line printed is a failure and names the file and position. No output is the
pass. Note that `git grep` exits 1 when it finds nothing, which is the good case
here, so do not wire this into a script that treats a non-zero status as an error
without inverting it first.

Check the sign-off on every commit you are about to push, the same rule the `DCO`
workflow applies:

```
for sha in $(git rev-list --no-merges origin/main..HEAD); do
  expected="Signed-off-by: $(git show -s --format='%an <%ae>' "$sha")"
  if git show -s --format='%B' "$sha" | grep -qxF "$expected"; then
    echo "ok    $sha"
  else
    echo "FAIL  $sha is missing: $expected"
  fi
done
```

Check the tracked bytes, the same script the `tracked-bytes` workflow runs:

```
bash .github/scripts/check-tracked-bytes.sh .
bash .github/scripts/prove-tracked-bytes.sh
```

The first refuses a carriage return, a byte order mark or a non-UTF-8 encoding in
tracked text, and a tracked file whose declared type does not match its content.
The second proves that each of those refusals bites, and the workflow runs it
first for that reason. `docs/tracked-bytes.md` says what each one covers and where
it stops.

Two of the gates have no local form here, and that is stated rather than left to
be discovered on a red pull request. The workflow audit runs
`uvx --no-build zizmor@<version> --strict-collection --min-severity=low --format=plain .`,
which needs a Python package runner and network access to fetch the tool; the
version it pins is in `.github/workflows/zizmor.yml`. If you have neither, that
gate runs on the pull request and not on your machine. The dependency review
compares the pull request diff against an advisory database on the server and has
no local equivalent at all.

## The gates a pull request is judged by

Do not read a list of them here, because a list drifts against the workflows.

The set that exists in the tree:

```
git ls-files .github/workflows
```

The set that actually ran on a pull request, with each result:

```
gh pr checks <number>
```

Every gate is fail-closed. A scanner that cannot run reds the check rather than
passing it, so a red result never means the gate was skipped.

## Sign your work

Every commit carries a `Signed-off-by` trailer whose name and address match the
commit author, and the trailer is your assertion of [DCO.md](DCO.md). The workflow
refuses a pull request containing a commit without one.

```
git commit -s -m "..."
```

To add the trailer to commits you already made on a branch:

```
git rebase --signoff origin/main
```

That rewrites the commits on your branch, so do it before anyone else builds on
them.

## What a good issue contains

What is wrong, what the evidence is, and what done means. All three, because an
issue missing the third is one nobody can close and an issue missing the second is
an opinion.

Where the evidence is a number, the issue carries the command that produced it, so
a reader can run it again and get the same number or find out that it moved.

A line reading `Scope:` at column zero, naming the paths the change is expected to
touch. It is how a reviewer sees at a glance that a change reached somewhere its
issue never mentioned.

Anything exploitable does not go on the public tracker. [SECURITY.md](SECURITY.md)
holds the private route, what a reporter can expect and by when, and what is not a
vulnerability here. Read it there rather than here, so the two cannot drift.

## What a good pull request body contains

What changed and what failure it prevents. Not a restatement of the diff, which
the reader already has.

Which issue it closes, with the closing keyword, so the tracker does not need a
second pass.

The commands whose output backs any number in the body, run at the commit being
pushed rather than in a working tree that has moved on since. A number without its
command is a claim, and it is fine to write a claim as long as it says it is one.

Whether anybody other than the author has read it, stated plainly either way. A
body saying nobody else has read this is worth more than one that is silent about
it.

## Rules this repository holds that you cannot guess

The default test suite runs headless and unelevated: no display server, no
elevated rights, no host font directory and no network, and a test that needs a
real environment goes to a separate harness named for what it needs and is not
part of the default run.

That is checked rather than asserted. This runs the suite in a container with none
of those things present, and then runs four probes that each reach for one of them
and must all fail:

```
bash .github/scripts/run-sealed.sh suite
bash .github/scripts/run-sealed.sh probes
```

Both need docker and the first fetch of the pinned image needs the network. The
`Headless and unelevated` check runs exactly these two commands, so a contributor
without docker sees the same result on the pull request instead.

Code that reads bytes this project did not create stays behind a boundary. A
parser takes bytes and returns a value or a typed error: it opens no path it was
not handed, makes no network call, reads no clock, and never aborts the process on
malformed input. That is a contract rather than a style preference, because it is
what makes a fuzz target a wrapper instead of a rewrite. Issue #8 is where the
build refuses a dependency edge that crosses the boundary.

A decision that shapes the architecture goes in `docs/decisions/`, one file per
decision, numbered, stating the decision in a sentence at the top, then the
reasoning, the cost, the rejected alternatives and the condition that would
reverse it. `docs/decisions/0002-track-order.md` is the shape to copy. A decision
argued only in a pull request thread is one the next person has to reconstruct
from memory.

## Style

English in tracked text. No tool names, generated-by markers or attribution
banners in anything tracked.

A commit message states what changed and what failure it prevents. Where it
corrects something, it says what was wrong and how that was found. One topic per
commit and one topic per pull request.
