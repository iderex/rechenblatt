# The shape of this repository

The workspace split was made while the tree was empty, which is the only time it
is cheap. It is an argument about what may depend on what, and an argument that
lives only in a directory structure gets undone by the first person who finds it
inconvenient. So it is written here, and the parts of it a machine can hold are
held by a machine rather than by this document.

Read `docs/decisions/0001-means.md` first if you want to know why any of this is
in Rust, and `docs/decisions/0014-input-boundary.md` for the line the next
section is about.

## What is where, printed rather than listed

Nothing below lists the files in a component or the edges cargo actually
resolved. Both are printed:

```
git ls-files crates
cargo tree --workspace --edges normal
```

The second is the authority on the dependency set. This document is the
authority on what that set is *allowed* to be, which is a different thing and
the reason both exist.

## The components

**model** is what a document says, read once. It owns the workbook: cells,
styles, themes, number formats, drawings, page setup, and the parts nothing
downstream uses yet. It depends on no other component here, and that is the
whole point of it - everything reads the model, and the model reads the
document.

**calc** evaluates what the page needs. It reads the model and nothing else in
the workspace, so a change to how a document is drawn cannot reach how a value
is computed.

**render** turns a model into pages. It reads the model and calc.

**macro** is the compatibility track: it reads a workbook and changes one. It
depends on model and calc, and deliberately not on render. A macro runtime that
could reach the renderer would become a second renderer, which is the failure
`docs/decisions/0002-track-order.md` exists to avoid - two readers of one
document that disagree about it.

**host** is the filesystem, the network and the clock. It is empty until
milestone 9 and it exists now so that the capabilities have somewhere to be that
is not everywhere. Nothing on the parsing side may depend on it.

**cli** is what an operator runs. It depends on all five. Everything it can do
lives in a component beside it, so the engine is usable without the command and
the command is replaceable without the engine.

## What each component may not depend on

The rule runs one way, so there is no cycle to untangle and no component that
has to be built twice. It is not a position in the list above, and the sentence
that stood here said it was: a component depending on the ones below it and
never on the ones above it would permit the first of the three refusals below,
since render sits above macro there and macro may not depend on render. What
each component may depend on is written out one component at a time in
`crates/cli/tests/component_edges.rs`, which holds the graph as data a run
reads.

Three refusals are worth naming on their own, because each one is a thing
somebody will want to do:

- **macro may not depend on render.** A macro that renders is a second
  renderer. Where the macro track wants a rendering, it asks for one through
  the operator surface rather than by linking the drawing code.
- **model may not depend on calc.** A model that computes is a model that has
  an opinion about values, and the whole claim of this project is that the
  model says what the document says.
- **nothing on the parsing side may depend on host.** That is the input
  boundary and it has its own section below.

`crates/cli/tests/component_edges.rs` refuses an edge this note does not have
and names it. What stood here said `Cargo.toml` was the enforcement for the
first two, and it was not. A crate cannot USE what it does not depend on, which
is a different sentence: adding the dependency line is one line in one manifest,
after which the compiler is satisfied and nothing else here has an opinion. What
was actually behind those two rules was a reviewer noticing the line, which this
repository calls prose. It was found by trying to name the check when the rules
were being turned into tests, and finding that the named enforcement refused a
different thing.

## The input boundary

This project reads documents it did not create. The code that walks those bytes
is where a malformed file stops being a parsing problem and becomes somebody
else's problem, so it is held on one side of a line and handed what it needs
rather than reaching for it.

Every member declares its side in its own manifest. `model`, `calc`, `render`
and `macro` are the parsing side; `host` and `cli` may reach a capability.

The reason it is a line and not a habit: a parser that takes bytes and returns a
value or a typed error is a fuzz target with a wrapper around it, and one that
opens a path and logs as it goes is a rewrite away from being fuzzable at all.
Milestone 11 attaches a fuzzing gate to every surface that takes bytes from
strangers, and this is what decides where it can attach.

`crates/cli/tests/boundary.rs` refuses a crossing and names the edge. What it
covers and where it stops is argued in `docs/decisions/0014-input-boundary.md`
rather than restated here, including the part it cannot see: it reads declared
dependencies, not source, so a parsing component calling `std::fs` directly
passes it.

## The floor the rendering pipeline stands on

The input boundary asks what a crate can reach. There is a second question about
the same crate, and a library can pass the first while failing the second: what
does it decide? `docs/decisions/0006-rendering.md` is where that is argued, and
the rule it states is that the pipeline holds no document semantics in a
dependency. A shaping library may place a glyph; it may not decide which font a
cell asked for.

Answering it is a person reading a crate and writing down what it may decide.
`crates/cli/tests/rendering_floor.rs` refuses the commit that skips that reading:
a crate in the pipeline's manifest that the record has never been asked about.
Where it stops is in the file's own header rather than restated here, and the
first line of it is that it reads the record as text, so it can tell a name that
is absent from one that is there and not what a name there was written for.

## Adding a component

A new component is a claim that something is separable, and the cost of being
wrong is a crate nobody can delete. Four things have to be true before one is
added, and if you cannot say all four, the code goes in an existing component
until you can.

**It has a name for what it is, not for where it sits.** `model` and `render`
name what they do. A component called `common` or `util` names where its code
ended up, and it is where the dependency rules go to die, because everything may
depend on it by definition.

**Its edges run one way and you can say which.** Write the line "X depends on Y
and not on Z" before writing the crate. If the honest answer is that the new
component and an existing one will depend on each other, they are one component
with a module boundary inside it.

**It declares which side of the input boundary it is on**, in
`[package.metadata.rechenblatt]` in its own manifest. There is no default: a
member with no side fails the suite, because a component added without one is
how the boundary erodes.

**Something is measurably better for it existing.** A separate fuzz target, a
capability that can be denied, a suite that can run without the rest. "It felt
tidier" is not one of those, and the workspace is already the shape the plan
argued for.

The mechanics are then: a directory under `crates/`, a manifest declaring
`publish = false` and inheriting the workspace lints, an entry in `members` in
the workspace manifest, an entry in `components()` in
`crates/cli/src/main.rs`, and an entry in the graph in
`crates/cli/tests/component_edges.rs` saying what the new component may depend
on. The suite requires the last two. A component wired into nothing builds,
passes its own tests, and no operator-facing thing knows it exists; a component
with no entry in the graph is one no check can judge, so it is refused rather
than allowed every edge it wrote for itself.

## Which of these rules a machine holds

A rule nothing refuses is an explanation of a rule. Each of the above is
therefore listed once with what holds it, so a reader can tell the two apart
without going looking.

| Rule | Held by |
| --- | --- |
| The edges, one component at a time | `crates/cli/tests/component_edges.rs` |
| macro may not depend on render | the same |
| model may not depend on calc | the same |
| A new component says what it may depend on | the same, which refuses a member with no entry |
| Nothing on the parsing side depends on host | `crates/cli/tests/boundary.rs` |
| Every member declares its side | the same |
| A parsing component links nothing from outside the workspace the workspace has not accepted | the same |
| A new component is named by the binary | the test beside `components()` in `crates/cli/src/main.rs` |
| Every path this note names is in the tree | `crates/cli/tests/documentation.rs` |
| A parsing component holds no route to the network | the `network-outside-host` record in `.github/scripts/invariants.txt`, which reads text rather than dependencies |
| The document is read once, by one component | `crates/cli/tests/document_parts.rs` |
| The pipeline holds no document semantics in a dependency | `crates/cli/tests/rendering_floor.rs`, which reads the record as text rather than the crate |
| A parsing component opens no path and reads no clock | nothing |
| A component is named for what it is rather than where it sits | nothing |
| Something is measurably better for a component existing | nothing |

The three rows saying nothing are two different cases and they are not
interchangeable.

The last two are judgements. No reading of this tree decides whether a name
describes a thing or whether a split earned its cost, so no check is owed for
them and writing one would produce a rule that passes for a component called
`util` as long as the file is spelled that way. The review is where a wrong
answer to either is caught.

The remaining one is a gap with work behind it. A parsing component calling
`std::fs` or reading a clock passes every check here, because the boundary check
reads declared dependencies rather than source and the pinned toolchain has no
lint that refuses a capability call by crate; the network third of that same
rule is refused only because a text scan happens to be able to see a socket
type.

Two of the rows above were that gap until recently and are worth naming as
having moved, because a reader who learned this table a week ago would still
believe them. A second reader of a document is now refused by name, and
`docs/decisions/0003-workbook-model.md` is where the rule and the check's bound
are argued. A dependency deciding what a document means is refused as far as a
reading of the record can reach, which is the section above.

Enforced here means a check in this tree refuses the violation at the commit
this note was last edited. A row saying nothing does is a statement about that
commit rather than a promise about the next one, and the landing that closes one
of these gaps is the landing that edits the row.

## What holds this document to the tree

Every path named here has to be there.
`crates/cli/tests/documentation.rs` refuses a path in this directory that does
not resolve, so a component renamed without its note going with it fails the
suite rather than leaving a document that reads correctly and points nowhere.
Issue #100 is where that grows into the rest of the documentation lint.

What refuses the rest is not listed here, because a list of checks drifts
against the checks. `docs/checks.md` names the commands that print what exists.

The table above is the one thing in this note that reads like such a list, and
it is the same exception `docs/checks.md` argues for itself: no command prints
which rule a check is about, so deleting the table would delete the answer
rather than move it somewhere derivable. What holds it is narrower than a
checker and worth saying plainly. Every path in it is refused by
`crates/cli/tests/documentation.rs` if it stops resolving, so a renamed test
reddens the suite. Nothing reads the rule column, and nothing compares a row
against what the named test actually refuses.
