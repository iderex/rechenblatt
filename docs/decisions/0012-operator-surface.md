# 0012 The operator surface

An operator runs a command first. A service comes second and only behind an
explicit decision to expose it. A library exists as a consequence of the
workspace layout rather than as a supported product.

Status: accepted
Date: 2026-08-06
Issue: #76

## The three shapes

The command takes a document and writes an output. It is the simplest thing that
is useful, it needs no network surface at all, and it fits into whatever an
operator already has.

The service is a long-running process with an interface over the network. It is
what an integration wants, and it is also an exposed surface on a machine holding
the operator's documents.

The library is what other software links against. It is the smallest of the three
and helps only people writing code.

## The order, and why

The command first. It is fully testable headless, it has no authentication
question, and it cannot be reached from anywhere the operator did not put it.
Every one of those becomes an open question the moment a socket is opened, and
none of them should be open while the engine is still being built.

The service second. What it adds over the command is real and it is worth being
precise about, because a service that is not justified is a surface taken on for
nothing. It holds the engine warm, so a caller does not pay start-up and font
loading per document. It gives one place to bound concurrency across many
callers, which a set of independent command invocations cannot do. And it lets
software that cannot conveniently run a subprocess still reach the engine, which
is the integration case the command genuinely cannot serve.

Everything else a service appears to add, an operator can get by invoking the
command. Batch processing is a loop. Scheduling is whatever already schedules
things on that host. Neither is a reason to open a socket.

The library third, and not as a product.

## What each shape promises about stability

The command's interface is the promise. Its options, its output formats and its
exit statuses are treated as an interface, and a change to any of them is a
change an operator has to be told about.

The service's request and response shapes are the same kind of promise, from the
point the service is first offered rather than from the point it is first
written.

The library has no stability promise at all until a record says otherwise. Its
components exist because the workspace has to be split somewhere, and the split
is an architectural claim rather than an offer. Anybody linking it is linking
something that will change without notice, and that sentence is the whole
promise until a later record replaces it.

## What the service has to meet before it is offered at all

The service is not built and not offered until all of these hold. Each names the
issue that meets it, and a pull request that offers the service states which ones
and how.

It binds to a loopback address by default, and binding anywhere else is an
explicit setting that says at startup what it did. Issue #78.

Callers are authenticated, and an unauthenticated request is refused before any
part of a document is read. Issue #78.

Every request is bounded at the edge in size, time, memory, disk and concurrency,
rather than having a limit discovered downstream when something has already been
consumed. Issue #80.

Configuration is explicit, validated in one place, and fails closed, because a
wrong configuration on an exposed surface is a security event rather than an
inconvenience. Issue #79.

Health and metrics answer honestly, so an operator can see that an instance is
degraded and why, and the metrics surface is subject to the same binding rule as
the service itself. Issue #82.

The default configuration makes no outbound connection of any kind, and anything
that would send document-derived data is off, names its destination, and is
refused unless that destination is explicitly configured. Issue #86.

Interrupted work says what happened to it, leaves no partial file at an output
path, and cleans its temporary directory on the next start. Issue #84.

The list is a conjunction. Meeting six of the seven is not a partial offer of the
service, it is a service that is not offered.

## What the packaging milestone builds

The packaging milestone builds what this record names and nothing else. Concretely
that means an artefact carrying the command, and a service-shaped example only in
the sense this record allows, which is behind the conditions above.

An image that runs the command, and an example that runs the service or the
command in a service-like mode with limits set and no port exposed beyond
loopback, are both inside what this record names. An example that exposes a port
to a network, or that presents the service as the ordinary way to run this, is
not, whatever it would do for a first ten minutes.

## Rejected alternatives

Service first. It is what an integration asks for and it opens every hard
question at once: authentication, binding, limits, configuration, and a surface
that can be reached while the engine underneath it is still changing weekly. The
cost lands on the operator, who is the party this project exists to protect.

The library as a supported product from the start. It sounds free, since the
components exist anyway, but a stability promise on a workspace split freezes the
architecture at the point it is least understood, and this project's split is an
argument that is expected to move.

## What would reverse this

An integration case that the command genuinely cannot serve, named and real
rather than anticipated, arriving before the seven conditions are met. That would
not reverse the conditions, which are about safety rather than about order. It
would only move the service earlier in the queue, and it would still wait for all
seven.
