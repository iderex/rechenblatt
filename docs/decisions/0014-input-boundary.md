# 0014 The input boundary

The components that read bytes this project did not create may not depend on the
component that can open a path, make a network call or read a clock, and the
suite refuses the dependency by name.

Status: accepted
Date: 2026-08-06
Issue: #8

The number is 0014 rather than the next one after 0002. Every number from 0001 to
0013 is named by an issue that has not delivered its record yet, so taking a free
gap would have taken one of those. Issue #124 is where this was found and moved.

## Context

This project exists to read documents somebody else made. The zip container, the
XML parts inside it, the compound file holding a macro project, the images, the
fonts a document names: every one of those is bytes from a stranger, and the code
that walks them is where a malformed file stops being a parsing problem and
becomes somebody else's problem.

The line is worth drawing while the tree is empty, because it is a line about
what a component is allowed to reach and every retrofit of such a line is a
rewrite of everything that already crossed it.

## The decision, in full

The workspace has two sides, and each member declares which one it is on, in its
own manifest:

```toml
[package.metadata.rechenblatt]
side = "pure"
```

`crates/model`, `crates/calc`, `crates/render` and `crates/macro` are on the
parsing side. `crates/host` and `crates/cli` are on the host side.
`crates/host` is new and empty, and it exists so that the capabilities have
somewhere to be that is not everywhere.

A component on the parsing side takes bytes and returns a value or a typed
error. It opens no path it was not handed, makes no network call, reads no clock,
and allocates against a ceiling it was given. It never aborts the process on
malformed input: a document that is wrong is a value the caller decides about,
not a reason for the program to stop. That contract is in `CONTRIBUTING.md`,
where somebody about to write a parser reads it, rather than only here.

A component on the parsing side may depend on another component on the parsing
side, and on nothing else. It may not depend on a host component, and it may not
depend on a crate from outside this workspace unless the workspace manifest lists
that crate in `pure-may-depend-on`. That list is empty today, which is a decision
rather than a placeholder: nothing outside the workspace has been read closely
enough to say what it can reach, so the check fails closed and the first
dependency a parser needs arrives with somebody having looked at it.

The other direction is not restricted, and that is the point of a direction. The
host side reads the model, hands a parser bytes, and takes back a value.

## Reasons, in the order they carry weight

A parser that takes bytes is a fuzz target with a wrapper around it. One that
takes a path and logs as it goes is not fuzzable without a filesystem and a log
sink, and discovering that after it is written costs the rewrite. Milestone 11
attaches a fuzzing gate to every surface that takes bytes from strangers, and
this record is what decides where that gate can attach.

A capability that is available everywhere is a capability nobody can reason
about. The interesting question about a malformed document is what the worst
thing that can happen is, and that question is only answerable if the set of
things the parsing code can do is small enough to write down. Here it is: it can
compute, and it can allocate.

Handing capabilities in rather than reaching for them makes a parser testable
without a fixture on disk. The bytes are a literal in the test, and a leg that
needs a hostile document does not need that document to exist as a file.

The rule is worth having as a refusal rather than as a paragraph because it
erodes silently. Nobody adds a filesystem dependency to a parser on purpose. It
arrives as a convenience inside something else, and by the time it is noticed the
code that relies on it is written.

## What the check does, and where it stops

`crates/cli/tests/boundary.rs` reads every member manifest and refuses four
things: a member declaring no side, a member declaring a side it cannot read, a
parsing component depending on a host component, and a parsing component
depending on an outside crate the workspace has not accepted. The refusal names
the edge, so a failing run locates the cause without a second run.

Twelve legs stand behind it. Each plants exactly one mistake in a scratch
workspace and requires that refusal and no other; each has a neighbour that
changes the one thing back and requires nothing to be refused. A check that
refuses everything fails the neighbours, and one that refuses nothing fails the
first kind.

Where it stops, stated rather than left to be discovered.

It reads declared dependencies, so it judges what a component links against. It
does not read source. A parsing component calling `std::fs` directly passes this
check, because the compiler offers no lint that refuses one module to one crate
and inventing a source scanner would mean judging comments and string literals as
if they were code. That gap is real and it is not covered here.

It does not read `[dev-dependencies]`, because a test is not the component. A
parsing crate's own tests build scratch directories on disk, and that is not the
capability this line is about.

It judges the shape of the workspace rather than the shape of a function. Nothing
here refuses a parser whose entry point takes something other than bytes; there
is no parser yet, and issue #101 is where the rest of the architecture becomes
tests.

## What this costs

An indirection. Reading a workbook from a path becomes two steps in two
components rather than one call, and the first time somebody wants a quick tool
that opens a file and prints a cell, they will find that the quick way is the
disallowed way.

A crate that does nothing. `crates/host` is empty until milestone 9, and an empty
component is a thing a reader has to be told the reason for. The reason is that
the alternative was to add it later, at which point every capability already has
a home somewhere else.

A list somebody has to maintain. `pure-may-depend-on` will be edited the first
time a parser needs a real dependency, and the check will be in the way on that
day. That is the day it is worth the most.

## What would reverse this

A parsing component that cannot be written without a capability, where the
capability cannot be handed in as a value. Streaming a document larger than
memory is the case to watch, and issue #23 is where it is decided: a reader that
must pull the next chunk itself needs something that can pull, and passing a
closure that reads may turn out to be worse than admitting the dependency.

Reversing this means naming that component and saying why handing the capability
in did not work, not restating this record with the sign changed.

## Rejected alternatives

**Making the parsing crates `no_std`.** The compiler would then refuse
`std::fs`, `std::net` and the clock outright, which closes the source-level gap
this record leaves open, and it is a genuinely stronger mechanism. It was
rejected because the dependencies a document reader needs - decompression, XML,
compound file containers - largely assume `std`, so the boundary would be bought
by giving up the ecosystem that makes the reading work possible at all. It is
worth revisiting per crate rather than for the whole side.

**A review rule instead of a check.** `docs/decisions/0001-means.md` argues that
a means in which nothing can refuse a violation fails the standpoint, and a rule
about what a component may depend on is exactly the kind that erodes without
anything going red.

**One crate with an internal module boundary.** Rust's visibility rules would let
a module reach the filesystem however the comments were written, so the boundary
would exist only in the reader's discipline. Crates are the unit the build
actually enforces, which is why the line is drawn there.
