# Formatting and lint

Two gates, and the point of both of them is that nobody has to argue. A
formatting preference discussed in a review is review attention spent somewhere
it does not belong, and a lint nobody agreed to is the same argument wearing a
different hat.

## The commands

Format the whole tree, writing:

```
cargo fmt --all
```

Check it without writing, which is what the gate runs:

```
cargo fmt --all -- --check
```

Any output is a failure and names the file and the lines. No output is the pass.

Lint every target in every crate, including the tests:

```
cargo clippy --locked --workspace --all-targets
```

`--locked` belongs on that command for the same reason it belongs on the build:
a run that would rewrite `Cargo.lock` fails instead of rewriting it.

Nothing above passes a rule on the command line, and that is deliberate. See
below.

## The check names

`Formatting`, `Lint` and `Prove the format and lint gates bite`, declared in
`.github/workflows/suite.yml` beside the build and the suite.

A name is a contract rather than a label, and `docs/checks.md` is the one place
that carries every name in this repository beside the command that reproduces it.
It is there rather than here so that there is one such place, and a checker
refuses a name that only the workflow or only the document knows about. A second
copy of the same three rows in this file would be exactly the drift that checker
exists against.

## Where the rules live

`rustfmt.toml` for the formatter and the lints table in `Cargo.toml` for the
linter. Both are tracked, so the tool is handed the same rules on a laptop and on
a runner, and changing a rule is a change to the tree that a reviewer sees.

The alternative is a flag on the command line, and it fails in one specific way:
the workflow gets `-D warnings` and the contributor does not, so the contributor
runs the gate, sees it pass, pushes, and meets a red check for something their
own run declined to tell them. The command in this document and the command in
the workflow are the same string, and that is the property worth keeping.

`rustfmt.toml` sets one thing, `newline_style`, and its reason is in the file: the
default infers line endings from the file being rewritten, so a formatter run on
a system holding CRLF writes CRLF back, and `docs/tracked-bytes.md` is the gate
that then refuses it.

`Cargo.toml` denies `warnings`, which is one line covering both tools. When cargo
hands that denial to clippy it reaches clippy's own lints as well as the
compiler's, so a lint in `clippy::all` fails the run without `clippy` appearing
anywhere in the declaration. That was measured rather than assumed, and it is why
there is no second line saying `clippy::all = "deny"`: adding one moved no run in
either direction, and a guard whose removal changes nothing is not a guard.

`clippy::pedantic` is not enabled. Run it and see what it costs:

```
cargo clippy --locked --workspace --all-targets -- -D clippy::pedantic
```

At the commit this document landed on, that refuses two places:
`must_use_candidate` against `components` in `crates/cli/src/main.rs`, and
`case_sensitive_file_extension_comparisons` against the `.md` test in
`crates/model/tests/fixture_registry.rs`. Neither is a defect, and the argument
for the exception each one would carry belongs beside that code rather than
inside the commit that builds the gate. The two sites are described rather than
pasted because the command prints paths the way the host writes them, and this
run was not on the host the workflow uses.

Widening the set is worth doing in a commit that says what it caught and carries
the exceptions beside the code, rather than inside the commit that builds the
gate.

There is no `clippy.toml`. It configures individual lints - thresholds, allowed
names, the minimum supported version - and cannot set a lint level, which is the
thing this gate needed. The one entry it would plausibly carry, `msrv`, clippy
already reads from `rust-version` in `Cargo.toml`, so writing it again would be a
second copy of a number that can drift against the first.

## What the proof covers, and where it stops

```
bash .github/scripts/prove-format-and-lint.sh
```

Nine legs. Each one unpacks `HEAD` into a scratch directory, plants exactly one
mistake in a crate, and requires the gate to refuse it and to name the file. Each
one has a neighbour with that single thing changed back, which requires a green
run: a gate that refuses everything fails the neighbour, and one that refuses
nothing fails the first leg.

Three legs ask a different question: they plant the mistake and then take the
tracked declaration away. For the linter, removing the denial from `Cargo.toml`
has to turn both a clippy lint and a compiler lint green, which is what says that
line is doing the refusing rather than a tool default underneath it. That is how
the redundant clippy declaration described above was found. For the formatter,
removing `rustfmt.toml` has to leave the run red, because its one setting is
about line endings and not about the indent the leg planted; a green run there
would mean the file was refusing something nobody claimed it refuses.

Where it stops. The proof reads `HEAD` and not the working tree, so it judges the
commit and a change that is not committed yet is not in it. It plants its
mistakes in one crate, so it proves the gate reaches that crate rather than every
crate. And it says nothing about whether the rule set is the right one, which is
a judgement and belongs in the review.
