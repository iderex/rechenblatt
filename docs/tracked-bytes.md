# The bytes in this tree

Two guards read the tracked tree and neither one covers what the other does. This
note says which refuses what, because the overlap is easy to assume and there is
almost none.

## What the tracked-bytes gate refuses

`.github/scripts/check-tracked-bytes.sh`, run by `.github/workflows/tracked-bytes.yml`.

Its subject is the bytes in git, read with `git cat-file blob :path`. Not the
working tree: a clone with `core.autocrlf=true` holds CRLF in its working copy for
files stored with LF, so a check reading the working tree would refuse a clean
repository on one platform and pass it on another.

Five properties, one refusal each.

`cr-in-tracked-text` is a carriage return in a file git treats as text. This is
the one people assume `.gitattributes` already handles. It does not: git's
conversion rewrites CRLF into LF on the way into the index and leaves a lone CR
exactly where it was. A lone CR is also the byte most likely to survive an editor,
a paste and a review unnoticed.

`bom-in-tracked-text` is a UTF-8 byte order mark at the start of such a file. Git
has no opinion about it at all.

`text-is-not-utf8` is such a file that is not valid UTF-8. Git has no opinion
about that either. A latin-1 file read as UTF-8 on one machine and as latin-1 on
another produces two results from one commit, which is exactly what a project
comparing renderings cannot have.

`declared-text-holds-nul` and `declared-binary-is-text` are the two directions of
a tracked file whose declared type does not match its content. They are separate
properties because the repairs are opposite: the first says declare the file
binary, the second says stop declaring it binary. A text file wearing a binary
declaration is the quieter of the two, because it is invisible to every check that
reads tracked text, including the Unicode guard below.

## What the Unicode guard refuses

`.github/workflows/unicode-guard.yml`.

Bidirectional overrides, isolates and marks, and zero-width characters, in files
`git grep -I` reads as text. That is the Trojan Source class, CVE-2021-42574,
where source renders differently from how it executes.

It says nothing about line endings, nothing about encoding validity, and nothing
about whether a file's declared type matches its content.

## Where the two touch

At one codepoint, and they disagree on purpose.

U+FEFF, the byte order mark, is deliberately outside the Unicode guard's set: its
own comment says a leading BOM is legitimate and banning it there would produce
false positives. The tracked-bytes gate refuses one at the start of a tracked text
file. That is not a contradiction. The Unicode guard is about a character
appearing anywhere in a line and deceiving a reader; the tracked-bytes gate is
about the first three bytes of a file and what a parser does with them.

Everything else is disjoint. Neither guard subsumes the other and removing either
leaves a class of byte nobody refuses.

## Exceptions

A fixture that exists to prove this project handles a carriage return, or a byte
order mark, is text carrying the byte the rules refuse. It stays by being declared
in `.gitattributes`:

    path/to/fixture.txt allow-cr
    path/to/other.txt   allow-bom

The check prints every exception it honoured and the count of paths it examined,
so a green run cannot be read as a tree with no exceptions in it. There are none
today, and the run says so rather than being silent about it.

There is no exception for the other three properties. A file that is not UTF-8, a
text file holding a NUL, or a binary declaration over readable text are all fixed
by changing the declaration or the file, and an escape hatch for them would be a
way to keep the ambiguity this whole gate exists to remove.

## Running both locally

    bash .github/scripts/check-tracked-bytes.sh .

Exit 0 is the pass. Exit 1 names each path and the property it broke. Exit 2 means
the check could not run, which is a failure and never a pass.

    bash .github/scripts/prove-tracked-bytes.sh

Twelve legs. Each property is refused by a repository holding exactly the byte it
is about and by nothing else, and the same repository with that byte changed back
refuses nothing at all. A checker that refuses everything fails the second kind of
leg; one that refuses nothing fails the first.

The declarations those legs are judged against are written inside the proof rather
than read from this repository's `.gitattributes`. A proof reading the real
declarations would prove the state of the tree on the day it ran. This proves the
checker.

The workflow runs the proof before the scan, so a checker that has stopped biting
reds the run instead of reporting a clean tree.

The Unicode guard's local form is in `CONTRIBUTING.md`, next to the other commands
a contributor runs before pushing.
