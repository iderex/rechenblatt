# Quality parity

The standard this project is held to is not invented here. It is the gate that
stands in front of the default branch of the public repository
`iderex/jellyfin-plugin-sso`, adapted to a document engine.

Parity is not copying a list of names. Several of that gate's checks are about a
plugin binary and a plugin catalogue and have no counterpart in a document
engine. Others have a counterpart under a different name because the language
differs. And this project needs gates that one does not, because it takes hostile
files from strangers, renders them byte for byte, and will eventually execute
code that arrived inside them.

This document is the map. Every line of the target gate is matched, replaced by a
named counterpart, or recorded as not applicable, and each carries one line of
reasoning and the issue that delivers it.

It maps. It does not list what this repository runs today, because such a list
drifts against the workflows the moment one is added. To see the current set:

    git ls-files .github/workflows
    gh api repos/iderex/rechenblatt/actions/workflows --jq '.workflows[].name'

## The target gate, read rather than remembered

    gh api repos/iderex/jellyfin-plugin-sso/rulesets --jq '.[] | "\(.id) \(.name)"'
    18802863 Protect main and 5.0

    gh api repos/iderex/jellyfin-plugin-sso/rulesets/18802863 --jq '{enforcement, bypass:.bypass_actors, required:[.rules[].parameters.required_status_checks[]?.context]}'
    {"bypass":[],"enforcement":"active","required":["build","ABI floor build","Package (JPRM) / Build package","Package (JPRM) / Generate SBOM","CodeQL","Analyze (csharp)","DCO sign-off","Deterministic PR-hygiene checks","Enforce greppable invariants","Reject Trojan Source Unicode","Audit workflows (zizmor)","prettier","dependency-review"]}

Thirteen required contexts, active enforcement, no bypass actors. Read again
before quoting: a required set is a live property of that repository and this
document is a copy of it taken when the map was written.

## The required set, mapped

| Target check | Verdict | Counterpart here | Issue |
| --- | --- | --- | --- |
| `build` | replaced | the build job of the pull-request workflow | #3, #6 |
| `ABI floor build` | replaced | the declared minimum toolchain, refused at build time | #3 |
| `Package (JPRM) / Build package` | replaced | the container image and the release route | #104, #106 |
| `Package (JPRM) / Generate SBOM` | replaced | the bill of materials beside the release artefact | #89, #99 |
| `CodeQL` | replaced | the code scanning gate for this language | #94 |
| `Analyze (csharp)` | replaced | the same scanning gate, plus a second analyser | #94, #95 |
| `DCO sign-off` | matched | the same check, under the same name | already in the tree, document in #10 |
| `Deterministic PR-hygiene checks` | no deliverer | nothing in this tracker delivers it | none |
| `Enforce greppable invariants` | matched | the same check, under the identical name | #98 |
| `Reject Trojan Source Unicode` | matched | the same check, under the same name | already in the tree |
| `Audit workflows (zizmor)` | matched | the same check, under the same name | already in the tree |
| `prettier` | replaced | the formatting gate for code, the lint for documents | #4, #100 |
| `dependency-review` | matched | the same check, under the same name | already in the tree |

The reasoning, one line each and in the same order.

`build` is a compile of a plugin assembly, so the name does not carry over, but
the thing behind it does: a change that does not compile must not merge.

`ABI floor build` exists because a plugin is loaded by a host application whose
oldest supported version has to keep working; this project is not loaded by
anything, and the nearest real obligation is the minimum toolchain #3 declares
and refuses below.

`Package (JPRM) / Build package` builds an artefact for a plugin catalogue this
project does not publish to; the artefact an operator here actually receives is
an image and a release.

`Package (JPRM) / Generate SBOM` carries over almost unchanged, and gains
something: a bill of materials here has to cover libraries the package manager
does not know about, which is why #99 states it separately.

`CodeQL` is the code scanning surface, and the language differs, so the gate is
the same idea with a different analyser.

`Analyze (csharp)` is the language-specific arm of that same scan, and this
project splits the obligation across a first analyser and a second one with a
different lens, because a renderer of hostile input is worth looking at twice.

`DCO sign-off` is already here and already refuses, and the document it points a
contributor at now exists.

`Deterministic PR-hygiene checks` has no counterpart and, more to the point, no
issue: nothing in this tracker delivers a check over the shape of a pull request.
That is a hole in the plan rather than a deviation with a reason, and it is
recorded here as one.

`Enforce greppable invariants` carries over whole, under the target's own name
rather than a synonym, since a name that differs for no reason costs a reader the
mapping. `docs/invariants.md` is where the check is argued and where the list it
reads is named.

`Reject Trojan Source Unicode` is already here, under the identical name, and the
name is worth keeping identical.

`Audit workflows (zizmor)` is already here, under the identical name.

`prettier` formats documents and configuration in a language this project does
not use, so the obligation splits: code formatting belongs to #4 and document
linting to #100.

`dependency-review` is already here and is language-independent.

## The practices the target runs without requiring them

    gh api repos/iderex/jellyfin-plugin-sso/actions/workflows --jq '.workflows[] | "\(.name)\t\(.state)"'

The set below is the workflows whose names do not appear among the required
contexts quoted above. That derivation is by name, and a workflow can produce a
check context named differently from itself, so treat the split as the shape of
the target's practice rather than as a verified one-to-one mapping. It was not
verified against the job names inside those files.

| Target practice | Verdict | Counterpart here | Issue |
| --- | --- | --- | --- |
| Stryker mutation testing | matched | mutation testing over the modules where correctness is not obvious | #97 |
| Fuzz (SharpFuzz) | matched, and widened | every surface that takes bytes from a stranger | #96 |
| E2E Login Harness | replaced | the harness for what cannot be pure, named for what it needs | #103 |
| Wiki Lint | replaced | the documentation lint, which this document is a subject of | #100 |
| Scorecard supply-chain security | matched | the same check, already in the tree | #99 for the triage |
| Manifest freshness, Regenerate manifest | not applicable | there is no catalogue manifest here | none |
| Nightly betas, Publish Beta, Publish Release | replaced | the release route and the upgrade procedure | #106, #107 |
| Publish failure alert | not applicable | it watches a publish route that does not exist here yet | none |

Mutation testing carries over for the same reason it exists there, that coverage
measures which lines ran and not whether a test would have noticed them changing.

Fuzzing is the one practice this project needs more of than the target, not less:
there, the fuzzed surface is small; here it is every reader of a document
somebody was sent by email.

The end-to-end harness there logs into a running server. The impure surface here
is different in kind, so the counterpart is not a login harness but the general
rule that anything needing a real environment lives in a separate named harness.

The wiki lint reads documentation that lives outside the tree. This project's
documentation is in the tree, so the counterpart is a lint over `docs/` rather
than over a wiki, which is why the verdict is replaced rather than matched.

The supply-chain self-audit already runs here. What #99 adds is the triage, so a
finding is either fixed or recorded as accepted with a reason rather than
accumulating.

A catalogue manifest is a plugin-distribution artefact with nothing behind it in
this project.

Publishing there produces plugin builds for a catalogue; here it produces an
image and a release an operator installs and upgrades.

The second analyser this project plans, #95, has no row of its own above because
its counterpart at the target sits inside the required set rather than beside it.
It is mapped under `Analyze (csharp)` and `Enforce greppable invariants`.

## What this project adds beyond the target

These exist because of what this software does, not because the target has them.

| Addition | Why it is needed here | Issue |
| --- | --- | --- |
| Input fuzz gate | Every reader in this project takes bytes from a stranger, which the target has almost none of. | #96 |
| Headless conformance gate | A renderer is the software most likely to acquire a hidden dependency on a display, a host font directory or a temporary path. | #102 |
| Rendering determinism gate | The fidelity comparison is a byte comparison, so a renderer that wobbles makes every other number here meaningless. | #42 |
| Fidelity baseline gate | A score nobody can fail is decoration; the baseline turns the measurement into a refusal. | #30 |
| Input boundary check | The rule that parsing code reaches no capability is worth nothing unless the build refuses the edge. | #8 |
| Corpus provenance check | A corpus of real documents is a place personal data and unclear licences arrive, and neither is caught by reading a diff. | #26, #90 |
| Architecture rules as tests | The dependency directions this project is built on are claims, and a claim that no test reads decays. | #101 |

## Rules named for the greppable gate that it does not refuse

Two rules were named as invariants for `Enforce greppable invariants` and neither
ended up in the list it reads. They are written here as the rules they actually
are, so that neither is left looking like something a check refuses.

Nothing here judges whether document content reaches a log format string.
Whether an interpolated value came from a document is a property of where that
value came from, and no reading of the text decides it: the same format string is
correct or a disclosure depending on what was bound to the variable three
functions earlier. The enforceable neighbour is a format string that is not a
literal, which is a different rule and is not substituted for this one. #81 is
where what a log may contain is decided.

A fixture without a provenance record is refused, and not by that gate.
`crates/model/tests/fixture_registry.rs` refuses it in the suite and
`tests/fixtures/README.md` carries the shape of a record. A second refusal
beside it would give one rule two places to be repaired and two places to
disagree, so the greppable list does not carry it. `docs/invariants.md` says the
same thing where somebody reading that gate will meet it.

## Which of these are merge conditions

None of them, today. Read rather than assumed:

    gh api repos/iderex/rechenblatt/rulesets --jq '.[] | "\(.id) \(.name)"'
    20487256 gate

    gh api repos/iderex/rechenblatt/rulesets/20487256 --jq '{enforcement, bypass:.bypass_actors, required:[.rules[].parameters.required_status_checks[]?.context]}'
    {"bypass":[],"enforcement":"active","required":[]}

The ruleset on the default branch requires a pull request, refuses deletion and
refuses a non-fast-forward push. It carries no required status check at all, so
every check that runs here is advisory and a red one is a mark beside a change
that can still be merged. That is the state this map is written against, and it
is the single largest deviation from the target, which requires thirteen.

Which checks become merge conditions is #93, and the split belongs there rather
than here. What this document commits to is the reasoning for the split: a check
becomes a merge condition when it refuses something specific, has been shown to
bite, and cannot be red for a reason outside the change. A check stays advisory
while it is a score, a scan whose findings need triage rather than a verdict, or
a run whose cost or duration means it happens on a schedule.

By that rule the gates that read the tree and refuse a named property belong in
the required set, and the ones that produce a number or a finding list belong
beside it. The fidelity baseline is the interesting case: it is a score, and #30
turns it into a refusal by fixing the comparison to a tracked baseline rather
than to a judgement, which is what makes it eligible.

## What this document is not

It is not a promise that the mapped checks exist. Every entry above naming an
issue is an entry naming work that has not been done, and the entries saying
`already in the tree` are the only ones that describe something running.

It is not a claim that the target gate is the right gate. It is the standard this
project chose to be held to, and a deviation from it is recorded here with a
reason rather than argued away.
