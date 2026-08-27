# 0015 Locale

The document decides how its own values are shown. An explicit setting supplied
to the run overrides it where the document is silent. The host environment is
read for none of it, ever.

Status: accepted
Date: 2026-08-27
Issue: #19

## Context

What a cell contains and what a cell shows are different things, and the
difference is a format string. The same stored number is `1,234.50` under one
convention and `1.234,50` under another, a month name is one word in one language
and a different word in another, and a date puts its day and its month in one
order or the other. Three parties have an opinion about which: the document,
whoever is running this software, and the machine it happens to be running on.

Nothing in this tree decides between them:

    git grep -in 'locale' -- docs/
    (no output, exit 1)

That absence is the reason for this record rather than a paragraph in a source
comment. The rule binds the number formats in issue #19, the renderer that draws
what they produce, the formula parser in issue #45 whose argument separator is
the same question in different clothes, and eventually the macro object model in
issue #72. A rule reached independently by four components is four rules.

The usual answer in this class of software is the host: read the machine's
regional settings and use them where the document says nothing. It is the answer
this record is written against, and the reason is in the next section rather than
in a preference.

## The decision, in full

Four sources, in this order, highest first. The first one that answers a question
answers it.

**1. What the format string itself declares.** A format string may carry its own
locale identifier, so that one column in a workbook can be shown under a
convention different from the rest of it. Where it does, that wins over
everything below, because it is the document being specific about this value
rather than about documents in general. Whether a given format string carries one
is read by the parser issue #19 builds; this record states the precedence and not
the grammar.

**2. What the workbook declares.** The language and convention the document
carries for itself, applied to every format string that named none.

**3. An explicit setting supplied to the run.** Where the document has said
nothing at either level, the operator may say it. This is the only place an
operator's opinion enters, and it enters below the document rather than above it.

**4. A fallback fixed in this project and written here.** Where none of the three
above has answered: the decimal separator is `.`, the group separator is `,`,
month and day names are English, and dates are shown in the order the format
string's own field sequence gives rather than reordered.

**The host environment is not in that list, and is not a fifth entry below it.**
No regional setting, no environment variable, no system language, no user
profile. A document that reaches step 4 gets the fallback above and not the
machine's convention.

### What this rule does not cover

**The epoch base is not a locale question.** Which of the two epoch bases a
workbook uses, and the leap-year behaviour of the older one, is a fact the
document declares about its own stored numbers. It is decided by the document
always, it has no override, and it never enters the precedence above. Issue #19's
third condition is where it is implemented.

**The formula separator is the same rule, stated where it is used.** Issue #45
requires the argument separator to be taken from the workbook rather than from
the host locale, which is steps 2 and 3 of this record reached independently. It
is named here so the two cannot drift apart, and not restated.

**Which locale data this project carries.** Month names in a language mean data
from somewhere, and that is a dependency. `docs/decisions/0001-means.md` sets the
rule that a dependency arrives in the commit that needs it, with its licence read
at that point, and this record does not pre-empt it.

### Every fallback is visible in the run

A value shown under step 4 is a value this project chose a convention for on the
document's behalf. The run says so: which locale each such value was shown under,
and that it came from the fallback rather than from the document.

That is the difference between a rendering that is wrong and a rendering that is
wrong and says nothing. It also makes the reversal condition below measurable,
because how often step 4 is reached over a real corpus is then a count rather
than an impression.

## Reasons, in the order they carry weight

**Two renders of one document have to produce identical bytes.** Issue #42
requires it, and a locale read from the host makes it a property of the machine.
The same document on two hosts would render differently, correctly by that rule,
and no test could tell that from a bug.

**A fidelity difference has to be attributable.** The corpus measures this
project against a reference. A difference caused by the host's regional settings
is a difference nobody can attribute to a stage, to a commit or to a decision,
and it would move when the machine did.

**The party being protected is an operator rendering somebody else's documents.**
`SECURITY.md` names them: someone who put this on a machine that also holds their
own files. Their machine's regional settings are a fact about them, and letting
those decide how a stranger's spreadsheet is displayed is the software making the
operator's environment part of the document's meaning.

**An operator who wants their own convention can still have it, and has to ask.**
Step 3 is that route. It costs one setting and it is a statement rather than an
accident, which is the whole difference between this rule and the host rule.

## What this costs

**A document that declares nothing looks foreign to whoever opens it.** A
workbook authored under a European convention that carries no declaration renders
with a point as its decimal separator, on the machine of somebody for whom that
is wrong. That is a real cost and the remedy is step 3, not the host.

**The fallback is this project deciding for a document that did not.** It is
written here so it can be argued with rather than found in the source, and the
run discloses every value it was applied to, but it is still a choice made on
somebody else's behalf.

**A setting has to exist before step 3 does anything.** Issue #79 is where
configuration is validated and issue #77 is the command that would carry it.
Until then the chain is three long and the fallback is reached more often than it
will be later.

**Locale data is weight.** Month and day names in the languages a corpus needs
are data this project has to carry and keep current, and that arrives as a
dependency with its own licence and its own notices.

## What would reverse this

The fallback moving is not a reversal of this record and is expected. What would
reverse it is a measurement, over the corpus issues #25 to #27 build, showing
that documents in the wild overwhelmingly declare no locale at any level, so that
step 4 decides the appearance of most values rather than a few. A rule whose last
resort is doing most of the work is a rule that has not been read correctly, and
the answer then is a different chain rather than a different fallback.

The host does not come back from that measurement. It would come back only from a
case where a document's appearance is genuinely a property of who is looking, and
this project's premise is the opposite: a document renders the same everywhere or
the number measuring it means nothing.

## Rejected alternatives

**The host as the last resort, below the document and the setting.** What most
software does, and what a reader of this record most expects to find. It is
rejected for the first two reasons above rather than for being wrong in spirit:
it makes a rendering a property of the machine, and it makes a class of fidelity
difference unattributable. It is also the most expensive alternative to reverse,
because by the time anybody notices, the reports are about machines rather than
about documents.

**An explicit setting above the document.** It reads as respecting the operator
and it overrules the document about its own content: a format string saying
exactly how this number is to be shown would be ignored because the operator set
a preference for something else entirely. The operator's opinion belongs where
the document has no opinion, which is where this record puts it.

**Refuse a document that declares no locale.** Consistent with this project's
habit of refusing loudly rather than guessing, and useless here. Declaration is
partial far more often than it is absent, so the refusal would fire on documents
that are almost fully specified, and there is no answer an operator could give at
that point that step 3 does not already give them earlier.

**No fallback: leave such a value unformatted.** Honest in the same way and worse
to look at. A page of unformatted serial numbers is not a rendering of the
document, and the disclosure above gets the same information to a reader without
destroying the page.
