# 0004 Input formats

The first release reads one format, the SpreadsheetML package the incumbent
suite writes today, and refuses everything else by name, from the bytes rather
than from the file name. The older binary workbook is refused as not yet
supported and reconsidered in a later milestone; the binary SpreadsheetML
workbook and the OpenDocument spreadsheet are refused as undecided.

Status: accepted
Date: 2026-09-03
Issue: #15

## Context

A fidelity project that accepts everything measures nothing, because a
difference could always be blamed on a format nobody meant to support. So the
input surface is decided before the parser exists, and it is decided against
what an operator actually has rather than against what would be pleasant to
support.

Three formats were candidates. The current format of the incumbent suite,
which is where the documents an operator makes today come from. The older
binary format of the same suite, which is where the documents a public body
inherited come from, and which is a different container with its own parser,
its own quirks and its own security history. And the format the two open suites
use natively.

The older binary format was the fourth open question of #111, and the decision
that this record depends on was taken on #15 on 2026-08-31: out of the first
release, deferred rather than refused for good, with the refusal saying the
format is not supported yet rather than implying it never will be. This record
does not re-argue that; it carries it out and adds the two formats that entry
did not cover.

## The decision, in full

### Accepted

**The SpreadsheetML package.** A zip archive of parts under the Open Packaging
Conventions, whose main part is a SpreadsheetML workbook. The ordinary
workbook, the macro-enabled workbook and the template forms of both are one
package format and are accepted as one thing: they differ in the content type of
the main part and in whether a macro project part is present, not in how the
package is read. Whether a macro that is present may run is the macro track's
question and is not decided by accepting the package that carries it.

It is read against ECMA-376, Office Open XML File Formats, 5th edition: Part 1,
Fundamentals and Markup Language Reference (December 2016), for the markup;
Part 2, Open Packaging Conventions (December 2021), for the container; Part 3,
Markup Compatibility and Extensibility (December 2015), for the extension
mechanism a document uses to carry what a later version added; and Part 4,
Transitional Migration Features (December 2016). The conformance class read
against is Transitional, because that is what the incumbent writes by default.
That last sentence is a claim rather than a measurement: no document in this
repository has been read to confirm it, and the corpus #26 builds is where a
document would show which class it declares.

The macro project part inside the package is a compound file with its own
specification. Naming it here would be naming a specification for a part this
record does not decide the reading of, so it is named in the macro track where
that part is first opened.

### Refused, deferred

**The older binary workbook.** A compound file holding streams of records. It
is refused, and the refusal says the format is not supported yet. #169 is the
issue that reconsiders it, in the milestone created for that purpose, and the
section below on adding a format is the condition it has to meet.

When it is read, it is read against MS-XLS, Excel Binary File Format (.xls)
Structure, at whatever revision stands on the day the decision moves; revision
12.2 of 2025-08-19 is the one current at the date of this record. The revision
is written here so a reader can tell whether the specification moved under the
decision, and it is not a commitment to that revision.

### Refused, undecided

**The binary SpreadsheetML workbook.** The same package format as the accepted
one, with its workbook and sheet parts written as binary records instead of
markup. It is the incumbent's own answer to a large workbook, so it arrives
from the same population the accepted format does. No decision is taken here,
because the format was named in no entry of #111 and deciding it in a record
that carries out a different decision would be deciding it by accident. It is
refused with a message saying so, and whoever decides it does so on the same
evidence the section below asks for.

**The OpenDocument spreadsheet.** The native format of the two open suites this
project is measured against. Those suites render it natively and this project
exists to close a fidelity gap on documents from the incumbent, so there is no
gap here to close. That is the argument for refusing it for good, and it is not
taken here: it is an argument from the project's premise rather than from a
count of what operators hold, and the section below says what a count would
have to show. Refused as undecided.

### What a refusal says

A refused document produces one message from the detector, and it says four
things: what the document is, in words; what the decision about that format
is, in the same words this record uses; the byte in the document that decided
it; and the name of this record. It says them before anything has tried to
parse the document, so a refusal is never a parse failure deep inside.

The message carries no path. The detector is in `crates/model`, on the parsing
side of the boundary `docs/decisions/0014-input-boundary.md` draws, so it takes
bytes and has no parameter for a name. The caller that opened the path writes
it in front of the message, and #77 is where that caller is built.

### Detection is by content

The format is read from the bytes and the name the file had plays no part,
because there is no parameter for it. The first bytes decide between the two
containers: the compound file signature and the zip local header signature are
both fixed. Inside a zip archive the central directory at the end of the file
names every part, and the names are enough: a workbook part, a binary workbook
part, a macro project part, a word-processing or presentation part, or the
`mimetype` entry an OpenDocument package carries first and stores uncompressed,
whose content is the one part content the detector reads.

Not one byte of any part is inflated. A detector that decompressed would carry
the attack surface #16 exists to bound, before #16 has bounded it. What the
detector reads is bounded by the length of the bytes it was handed, it
allocates nothing, and every offset it follows out of the archive's own
records is checked against the bytes before it is used, so a record pointing
outside the file is a damaged package rather than a panic.

`crates/model/tests/input_format.rs` proves it. Each leg builds a package in
memory with the parts its format carries, and the renamed-file legs write one
set of bytes under several names in a directory the test made, read each back
through its path, and require the same answer from all of them: a workbook
package called `.xls`, `.ods`, `.txt` or nothing at all is a workbook package,
and a compound file called `.xlsx` is the older binary workbook. The packages
are built by the test rather than committed, because a fixture under
`tests/fixtures/` is outside what #15 declared it would touch, and because
these bytes are a literal in the sense `docs/decisions/0014-input-boundary.md`
means: a leg that needs a document does not need it to exist as a file.

### Where detection stops

It reads names, not the relationship that formally identifies a package's
main part, because reading that relationship means inflating it. A package
whose workbook part is not at the conventional name is reported as a package
this project cannot name, which is a refusal and not a wrong acceptance. The
reader #16 builds inflates parts behind ceilings, and the formal
identification through the package relationships belongs there.

It tells a macro-enabled package from an ordinary one by the presence of the
macro project part rather than by the content type of the main part, for the
same reason. A package declaring itself macro-enabled and carrying no project
is detected as an ordinary workbook, and a package declaring itself ordinary
and carrying a project is detected as macro-enabled. The second is the one
that matters and it is the one this reading gets right.

The packages the tests build are not documents an application wrote. They
carry the parts a workbook carries and no more, so they prove the detector and
not the reader. A document from an application arrives with the corpus, and
the first one is where this detector meets a package it did not build.

## Reasons, in the order they carry weight

**One format is what a first release can measure.** Every fidelity number this
project publishes is a number about the documents it read. Two formats in the
first release are two populations, two reference producers and two sets of
quirks under one number, and the number stops meaning anything.

**The accepted format is where the documents are.** An operator's own documents
are in the current format; the archive case is real and it is the case that
arrives least often first. The decision on #15 weighed that and this record
carries the result.

**Content beats the name because the name is the cheapest thing to lie about.**
A document arrives by email with the extension somebody chose, and a reader
that trusts the extension parses a compound file as a zip archive and reports
the failure from somewhere deep inside the wrong parser. Reading the signature
costs eight bytes and makes the refusal say what the thing actually is.

**Refusing by name is what makes the corpus honest.** A format nobody meant to
support, silently accepted, is a difference nobody can attribute. A format
refused with its name is a document the corpus does not count.

**Undecided is a state and it is written down as one.** Two formats are refused
here without a decision being taken about them, and the refusal says so rather
than dressing the absence of a decision as one. A reader of the message and a
reader of this record get the same word.

## What this costs

**Everything an operator inherited is refused.** Public bodies with twenty
years of archives get a refusal on every one of those files, with a message
saying not yet. That is the direct cost of the deferral and it is not softened
here; #169 is the issue that pays it.

**The incumbent's own large-workbook format is refused.** An operator whose
workbooks are big enough that the incumbent saved them in the binary form gets
a refusal, and that operator is more likely than most to be the one this
project is for.

**Detection carries names the reader will carry again.** The part names the
detector compares against live in the model crate, which is the one component
allowed to name them, and #16 will name them a second time when it opens the
parts. Two places is the price of a detector that does not inflate, and the
model ownership check keeps both inside one component.

**A reading by convention rather than by relationship.** A conforming package
with an unconventional workbook part name is refused. No document from the
incumbent is known to be shaped that way, which is a claim, and a reader that
finds one has the refusal message to quote.

## What would have to be true before another format is added

Three things, all written down, before the record moves a format from refused
to accepted.

**A measurement of need over the corpus, not an impression.** How often
documents in that format arrive, counted over the population #26 assembles and
published by #33, with the command that produced the count. The fourth entry of
#111 said the need for the older format was unmeasured, and it stays unmeasured
until something counts it.

**A reading of the format's specification that names what the parser has to
reach.** The specification by title and revision, and a sentence saying what a
document in that format can make a reader do: which container, which
decompression, which embedded objects. For a compound file that is a different
container from the accepted one with its own history of hostile documents, and
`docs/decisions/0003-workbook-model.md` already names that the document part
markers cover nothing of it until somebody adds its stream names.

**A place for its refusals to live.** Every ceiling and every refusal #16
builds for the accepted container has a counterpart for the new one, or the
record says which have none and why. A second container that arrives without
its bounds is the accepted container's attack surface doubled.

Adding a format is then an amendment to this record naming the evidence, the
specification and the bounds, and a change to the detector that stops refusing
it. The message the detector prints for that format today already tells a
reader that this section is where the answer is.

## What would reverse this

A count showing that the population this project is aimed at holds mostly
documents in a refused format. If the corpus, once assembled from the sources
#111's third entry allows, shows the older binary workbook or the binary
SpreadsheetML workbook to be the majority of what public bodies actually hold,
then the first release is reading the wrong format and this record is replaced
rather than amended.

Reversing it means citing that count. It does not mean restating this record
with a different format in the first section.

## Rejected alternatives

**Accept the older binary format in the first release.** It serves the archive
case and roughly doubles the reading work before anything renders, for the
case that arrives least often first. Rejected on #15, with the reasoning
recorded there and carried here.

**Refuse the older binary format for good.** It keeps the project focused and
tells a large part of the intended audience that their problem is not this
project's problem, on the strength of a need nobody measured. Rejected on #15
for that reason; the deferral keeps the refusal honest and the reconsideration
visible.

**Detect by extension and confirm by content.** Cheaper to write and it makes
the extension part of the answer, so a renamed file produces a message about
the name rather than about the document. The whole point of reading the
content is that the name is not evidence.

**Detect by inflating the content types part.** The formally correct way to
identify a package, and it means running a decompressor on a stranger's bytes
before any ceiling exists. Rejected until #16 puts the ceilings there; the
detector reads names from the central directory instead and says so.

**Decide the OpenDocument spreadsheet now, either way.** Refusing it for good
is well argued from the premise and poorly argued from evidence. Accepting it
would add a format the two open suites already serve. Neither was in the
question this record answers, so both are left where they can be taken with
the evidence the section above asks for.
