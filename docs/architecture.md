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

The rule is short and it runs one way. A component depends on the ones below it
in the list above and never on the ones above it, so there is no cycle to
untangle and no component that has to be built twice.

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

A crate cannot use what it does not depend on, so `Cargo.toml` is the
enforcement for the first two rather than a diagram of one. Adding the edge is
what a reviewer would see.

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
the workspace manifest, and an entry in `components()` in
`crates/cli/src/main.rs`, which the suite requires - a component wired into
nothing builds, passes its own tests, and no operator-facing thing knows it
exists.

## What holds this document to the tree

Every path named here has to be there.
`crates/cli/tests/documentation.rs` refuses a path in this directory that does
not resolve, so a component renamed without its note going with it fails the
suite rather than leaving a document that reads correctly and points nowhere.
Issue #100 is where that grows into the rest of the documentation lint.

What refuses the rest is not listed here, because a list of checks drifts
against the checks. `docs/checks.md` names the commands that print what exists.
