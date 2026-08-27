# Security policy

## Reporting something exploitable

Use GitHub's private vulnerability reporting on this repository:

<https://github.com/iderex/rechenblatt/security/advisories/new>

That route is enabled, and the state is readable rather than promised:

```
gh api repos/iderex/rechenblatt/private-vulnerability-reporting --jq .enabled
true
```

If that command prints anything else, the route is closed, and this repository
publishes no second address to fall back to. Say only that you have something to
report and wait to be asked for it. Do not open a public issue for anything
exploitable, and do not describe the problem in a pull request that fixes it until
the report has been answered.

A useful report says which bytes cause it. A document, a macro, or a request that
reproduces the behaviour is worth more than a description of it. If the document
is one you may not share, a reduced file that still triggers the behaviour is the
next best thing, and a report with neither is still worth sending.

## What a reporter can expect, and by when

An acknowledgement within 5 working days, from a person rather than an automated
reply.

An assessment within 15 working days of the acknowledgement: whether the report is
accepted, what it is judged to affect, and either a fix in progress or the reason
it is not one. If the assessment cannot be made in that window, the reporter is
told why and given a new date rather than left with silence.

Credit in the advisory under whatever name the reporter asks for, or no credit if
they prefer that. This is asked rather than assumed.

Disclosure by agreement. The intent is an advisory published once a fix exists,
and a reporter who wants to publish sooner is told what is still unfixed rather
than argued with.

None of the above is an availability guarantee. This is a small project and the
dates are what one maintainer can hold, which is why they are stated in working
days rather than hours.

## Which versions are in scope

The default branch, and nothing else, because there is no release yet:

```
gh api repos/iderex/rechenblatt/releases --jq 'length'
0
```

Once releases exist, this section names which of them still receive fixes. Until
then, a report is assessed against `main` at the time it arrives.

The repository holds a workspace of Rust crates and no parser:

```
git ls-tree -r --name-only origin/main | grep -cE '\.(rs|toml)$'
23
git grep -c -E '^[[:space:]]*(pub )?fn ' origin/main -- 'crates/*/src/*.rs'
origin/main:crates/cli/src/main.rs:4
```

Twenty-three tracked source and manifest files, and every function among them
sits in the command's own binary. The five libraries hold a doc comment and the
name their component answers to, so nothing here yet takes a byte out of a
document. The eight test files in the workspace judge the tree itself: the
dependency edges, the fixture register, the paths the documents name.

So a report against this repository right now is most likely to be about the
workflows, the tracked text, or the supply chain around them, rather than about a
parser. The threat model below is written for the software as it is built, and it
is stated now because the parsers arrive under it rather than acquiring it later.

## The threat model, in plain terms

A document is untrusted input. Every byte of a workbook, including the parts
nobody looks at, arrived from somewhere and is treated as hostile. A parser takes
bytes and returns a value or a typed error. It opens no path it was not handed,
makes no network call, and never aborts the process on malformed input.

A macro is untrusted code. It is code that arrived inside a document, and it does
not run because a document asked to be opened. What a macro may reach is decided
before it runs and defaults to the workbook and nothing else.

The party being protected is an operator running this beside their own files. Not
a hosted service with an isolated tenant per document, and not a developer running
it on a sample. Someone who put this on a machine that also holds the files it was
meant to keep away from a stranger. A finding is judged by what it does to that
person.

Two things follow from that and are worth stating separately. Documents stay on
the host: nothing this software does sends a document, or anything derived from
one, anywhere else, and issues #85 to #87 are where that becomes a proven property
rather than a sentence. And a resource ceiling is a security property here, not a
performance one, because a document that exhausts the host is a document that
denied its operator their machine.

## What is not a vulnerability here

A rendering difference from another product. That is what the fidelity corpus
measures and where it belongs is the public tracker, using the fidelity difference
template. It is the most common report this project expects and it is not a
security matter.

A document this software refuses to open, or a macro construct it refuses to run
by name. Refusing loudly is the designed behaviour. A refusal that is wrong is a
bug and goes on the tracker; a refusal that is silent is a bug and may well be a
security one, so send that privately.

A missing feature, including a format this project has said it does not read.

A scanner finding with no reachable path through this code. Send the path, or send
the finding to the tracker as a hardening suggestion. A dependency advisory
against something this project actually ships is in scope and does belong here.

Anything that requires the operator to have already decided to do it. If a
capability is one the operator explicitly configured, a report that the capability
works is not a vulnerability. A report that it can be reached without that
configuration is.

## If this software leaked a secret

Rotate first. A token, a document password or a key that reached a log line, a
metric label, a crash report, a diagnostic bundle or a message shown to a user is
compromised from the moment it was written there, and nothing an advisory says
later changes that. Rotation is the one step that does not depend on finding the
copies.

Then report it, by the private route at the top of this page. What is useful in
that report: which surface the value appeared on, at what verbosity level, the
smallest input that reproduces it, and the value in a redacted form rather than
the value. A leak that only happens at the most verbose level is still a leak,
because that level is one an operator is entitled to turn on.

A fix here stops this software writing that value again, and the advisory names
the surface, the versions affected and what an operator has to check on their own
machine. Where the copies already written went is outside what this project can
see: a log has its own retention and its own access, it may already be in a bug
report or a support ticket, and finding and removing those copies is the
operator's work. That asymmetry is the reason rotation comes before the report
rather than after it.

Nothing here holds a secret today. The tree carries no code that takes a token, a
password or a key, so this section describes a route before there is anything
travelling it, and no test in this repository proves a secret stays out of the
surfaces listed above. What exists is one pattern over the tracked source, which
refuses a credential held in a type whose `Debug` and `Display` would print it:

```
git grep -n 'Id: secret-in-a-plain-string' -- .github/scripts/invariants.txt
.github/scripts/invariants.txt:44:Id: secret-in-a-plain-string
```

That reads a declaration and never a value, so it catches the shape and not the
leak. Issue #83 is where the redacting type, the accepted routes for supplying a
secret and the tests over a crash report and a diagnostic bundle are built.

## Once a report is accepted

The fix lands with a test that fails without it, on the same terms as any other
change here. The advisory names what an operator has to do, not only what changed,
and if the answer is that they have to upgrade, it says so plainly.
