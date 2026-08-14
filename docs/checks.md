# The checks

The jobs a proposed change sets off, the name each one answers to, and the
command that reproduces it here.

## What exists, printed rather than written down

The workflows in the tree:

```
git ls-files .github/workflows
```

The jobs a given pull request actually ran, with each result:

```
gh pr checks <number>
```

Every check name a job in this tree can produce, which is the same extraction the
checker below makes:

```
awk '/^jobs:/{j=1;next} j&&/^[^ #]/{j=0} j&&/^    name:/{sub(/^    name: /,"");print}' .github/workflows/*.yml
```

Those three answer three different questions and none of them is the same
question. The first is what the tree carries, the second is what a given pull
request got, and the third is the set of names a rule could be written against.

## The names, and what reproduces each one

A name in this table is a contract, not a label. A rule that requires a check
matches it by its literal name, and a job with no `name:` produces a check named
after its job id instead, so adding, removing or retyping one of those lines
renames the check and detaches any rule that required the old name. Nothing goes
red while that happens, which is the whole problem with it.

| Check | What reproduces it here |
| --- | --- |
| `Formatting` | `cargo fmt --all -- --check` |
| `Lint` | `cargo clippy --locked --workspace --all-targets` |
| `Prove the format and lint gates bite` | `bash .github/scripts/prove-format-and-lint.sh` |
| `Build and suite` | `cargo build --locked --workspace --all-targets`, then `cargo test --locked --workspace`, then the coverage floor |
| `Headless and unelevated` | `bash .github/scripts/needs-an-environment.sh sealed-suite`, then `bash .github/scripts/needs-an-environment.sh sealed-probes`; both need docker, and the first fetch of the pinned image needs the network |
| `Declare what needs an environment` | `bash .github/scripts/prove-needs-an-environment.sh`, then `bash .github/scripts/check-needs-an-environment.sh .`, then `bash .github/scripts/needs-an-environment.sh list`; `docs/needs-an-environment.md` argues what it covers |
| `What the default run excludes` | `bash .github/scripts/prove-excluded-from-the-default-run.sh`, then `bash .github/scripts/check-excluded-from-the-default-run.sh .`; `docs/excluded-from-the-default-run.md` argues what it covers |
| `Names match the document` | `bash .github/scripts/prove-check-names.sh`, then `bash .github/scripts/check-check-names.sh .` |
| `Code scanning` | `bash .github/scripts/prove-code-scanning.sh`, then `bash .github/scripts/check-code-scanning.sh .`; `docs/code-scanning.md` argues what it covers |
| `Refuse ambiguous tracked bytes` | `bash .github/scripts/prove-tracked-bytes.sh`, then `bash .github/scripts/check-tracked-bytes.sh .` |
| `Enforce greppable invariants` | `bash .github/scripts/prove-invariants.sh`, then `bash .github/scripts/check-invariants.sh .`; `docs/invariants.md` argues what it covers |
| `Reject Trojan Source Unicode` | the `git grep` expression in `CONTRIBUTING.md` |
| `DCO sign-off` | the sign-off loop in `CONTRIBUTING.md` |
| `Audit workflows (zizmor)` | no local form without a Python package runner and network access |
| `Dependency review` | no local form; it compares the diff against an advisory database on the server |
| `Scorecard analysis` | no local form; it does not run on a pull request at all |

`docs/format-and-lint.md`, `docs/test-harness.md` and `docs/tracked-bytes.md` are
where the first five are argued. The two with no local form are disclosed in
`CONTRIBUTING.md` as well, so a contributor meets that fact before a red check
tells them.

`Headless and unelevated` is the one row whose command needs something a
contributor may not have. It reaches the sealed run through
`docs/needs-an-environment.md`'s runner rather than around it, so the route a
contributor is told to use is the route the gate proves, and the run carries the
environment it happened in. The two rows beside it read files and need nothing at
all, which is why they are separate checks rather than one: one reads that
runner's register, and the other reads the set of tests the default run does not
run and refuses one that reaches neither the run nor the runner.

## Why a table here is not the drift it looks like

A list in a document drifts against the thing it describes, and the usual repair
is to delete the list and name the command that prints it. Both commands are
above. The table stays because a name alone is useless: the reader wants the
command that reproduces the check, and no command in the world prints that.

A checker refuses the drift, which is the whole reason the table is safe to
keep:

```
bash .github/scripts/check-check-names.sh .
```

It reads every job out of `.github/workflows/` and every backticked name out of
the first column above, and refuses three things: a job with no explicit `name:`,
a job whose name has no row here, and a row here that no job produces. So a
workflow edit that renames a check fails until this table is edited with it, and
a row that describes a gate nobody runs fails too.

Where it stops. It compares names, so a row whose command is wrong passes; that
is what a review is for. It reads jobs, so a check run created by something other
than a job in this tree - code scanning uploads one under the tool's own name -
is neither required nor refused here. And it says nothing about which names a
rule on the default branch requires, because that is a repository setting and not
a fact of the tree. Today that answer is none:

```
gh api repos/iderex/rechenblatt/rulesets/20487256 \
  --jq '[.rules[] | select(.type == "required_status_checks")] | length'
0
```

Which of these names stand in front of the default branch is answered in
`docs/required-checks.md`, which names six of them and gives the reason for each.
The answer is not yet in the ruleset: the command above still returns zero, so
these gates are run and read rather than required, and a red one is a mark beside
a change that can still be merged. That document says what moves a name from this
table into the required set, and that the ruleset and the document change
together.

## The shape every workflow here holds

- Every action is pinned to a commit hash with its version in a comment beside
  it. Nothing printed by the following is the pass:

      grep -rn 'uses:' .github/workflows/ | grep -v '@[0-9a-f]\{40\} #'

- Every checkout runs with `persist-credentials: false`, so no step leaves the
  token in `.git/config` for a later step to find.
- Permissions are declared read-only at the workflow level and widened only on
  the job that needs more. The jobs that upload findings need
  `security-events: write`, and each declares it for itself rather than the
  workflow granting it to everything beside it.
- No path filter skips a job. A skipped job reports nothing rather than
  reporting success, and a rule requiring that check reads nothing as never
  satisfied, so the change waits on a gate that decided it had nothing to do.
  There is no path filter in the tree:

      grep -rn 'paths:\|paths-ignore:' .github/workflows/

- Every gate is fail-closed. A scanner that cannot run reds its check rather
  than passing it, so a red result never means the gate was skipped.
