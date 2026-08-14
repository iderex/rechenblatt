# The checks that stand in front of the default branch

A check that is not required is a suggestion. The pages below sort the checks
running here into the ones that are merge conditions and the ones that stay
beside them, with the reason for each, so that a later change to the ruleset has
a written decision behind it and not a preference.

`docs/quality-parity.md` maps this repository's gate onto the standard it chose
to be held to. It states the rule for the split and stops there; the split itself
is made here.

## The rule

A check becomes a merge condition when all three hold.

It refuses something specific. A check that produces a score or a list of
findings is not refusing anything; it is reporting, and a reporting check made
into a merge condition turns every report into a blockage.

It has been shown to bite. Somewhere in the tree there is a run that plants the
mistake the check is about and requires the check to refuse it. Without that, a
required check can quietly stop refusing and the merge condition becomes a
formality that guarantees nothing.

It cannot be red for a reason outside the change. A check that reads an upstream
feed, a published advisory or a rule set somebody else maintains can redden a
change that did not touch anything, and a required check in that position stops
the queue for a reason nobody in the pull request can fix.

## The ruleset, as it was read

    gh api repos/iderex/rechenblatt/rulesets --jq '.[] | "\(.id) \(.name)"'
    20487256 gate

    gh api repos/iderex/rechenblatt/rulesets/20487256 \
      --jq '{enforcement, bypass:.bypass_actors, required:[.rules[].parameters.required_status_checks[]?.context], rules:[.rules[].type]}'
    {"bypass":[],"enforcement":"active","required":[],"rules":["deletion","non_fast_forward","pull_request"]}

Enforcement is active. A pull request is required, deletion is refused and a
non-fast-forward push is refused. No status check is required at all, so every
check that runs here is advisory today and a red one is a mark beside a change
that can still be merged.

Bypass actors are empty, and they stay empty. A required set with a bypass actor
is a required set for everybody except the person most able to break it.

This is the state at the time of writing, and a ruleset is a live property.
Re-run both commands before quoting either.

## The names, read from completed runs

A required context is matched by its literal check-run name, and a job name in a
workflow file is not always the name that arrives. So the names below come from
completed runs rather than from the files.

Two commits, because two of these checks only run on a pull request and one name
appears only there. The default branch at the time of writing:

    gh api 'repos/iderex/rechenblatt/commits/058019f83e7a103c84541046ca68586cf323ebe5/check-runs?per_page=100' \
      --jq '.check_runs[] | "\(.name)\t\(.conclusion)"' | sort -u
    Audit workflows (zizmor)	success
    Build and suite	success
    Formatting	success
    Lint	success
    Names match the document	success
    Prove the format and lint gates bite	success
    Refuse ambiguous tracked bytes	success
    Reject Trojan Source Unicode	success
    Scorecard analysis	success

The head of a merged pull request, where the rest appear:

    gh api 'repos/iderex/rechenblatt/commits/782568028ad45bdd447f32ea4aeda56d45c1dcab/check-runs?per_page=100' \
      --jq '.check_runs[] | "\(.name)\t\(.conclusion)"' | sort -u
    Audit workflows (zizmor)	success
    Build and suite	success
    DCO sign-off	success
    Dependency review	success
    Formatting	success
    Lint	success
    Names match the document	success
    Prove the format and lint gates bite	success
    Refuse ambiguous tracked bytes	success
    Reject Trojan Source Unicode	success
    zizmor	success

`Audit workflows (zizmor)` and `zizmor` are two different check runs from one
workflow, and only the first is a job. A required set naming the second would be
requiring something that does not arrive on the default branch at all, which is
the exact mistake reading the names from a file rather than from a run produces.

`Scorecard analysis` appears on the first list and not the second. It runs on the
default branch and on a schedule, so a pull request never produces it, and a
required context a pull request cannot produce is one nothing ever satisfies.

## The required set

| Check | Refuses | Shown to bite by | Delivered by |
| --- | --- | --- | --- |
| `Formatting` | source the pinned formatter would rewrite | `.github/scripts/prove-format-and-lint.sh` | #4 |
| `Lint` | a lint the tracked denial names | `.github/scripts/prove-format-and-lint.sh` | #4 |
| `Prove the format and lint gates bite` | a formatting or lint gate that has stopped refusing | it is that run | #4 |
| `Build and suite` | a change that does not compile, a red suite, coverage under the declared floor | the legs inside the suite, and a run that cannot read the floor fails rather than assuming one | #3, #5, #6 |
| `Names match the document` | a check name only the workflow or only `docs/checks.md` knows about | `.github/scripts/prove-check-names.sh` | #6 |
| `Refuse ambiguous tracked bytes` | a tracked path the attributes cannot reach | `.github/scripts/prove-tracked-bytes.sh` | #9 |

Six, all of them first-party, all of them reading this repository and nothing
else. Every one has a proof beside it that plants its own mistake and requires
the refusal, which is what makes requiring it worth anything.

## What stays beside it, and why

`DCO sign-off` refuses something specific, is first-party, fails closed, and
cannot be red for an outside reason. It fails only the second condition: nothing
in this tree plants a commit without a sign-off and requires the check to refuse
it.

`Reject Trojan Source Unicode` is in the same position for the same reason. It
refuses a named byte class and fails closed on a scanner error, and no run here
plants one of those characters and requires the refusal.

Both are one proof away from the table above, and no issue in the tracker
delivers either proof. That is a hole in the plan rather than a deviation with a
reason, and it is recorded here as one. Read rather than remembered:

    gh issue list --repo iderex/rechenblatt --state open --limit 200 \
      --json number,title,body \
      --jq '.[] | select((.title + " " + .body) | test("DCO|sign-off|Trojan"; "i")) | "#\(.number) \(.title)"'
    #88 Add the licence file and the source file headers
    #37 Shape and place text the way the document asks

Two matches, neither about a proof that either check bites.

`Audit workflows (zizmor)` and `zizmor` report findings against rules maintained
elsewhere. A new rule upstream reddens a workflow file nobody touched.

`Dependency review` reads published advisories. An advisory published between one
run and the next reddens an unchanged change, which is the third condition
failing exactly as it is written.

`Scorecard analysis` is a score, and it never appears on a pull request, so it
could not be required even if it were eligible.

`Declare what needs an environment` reads the register behind the harness in
`docs/needs-an-environment.md` and refuses an entry naming no environment, no
reason or no script. It meets all three conditions: it refuses named things, its
proof plants each of those mistakes and requires the refusal, and it reads one
tracked file and nothing outside this repository. It is outside the table for the
same reason the headless gate is, which is that the table quotes names read from
completed runs and it has none yet. It joins the table in the same change
that adds it to the ruleset.

The harness that check reads the register of is a different thing and is not a
candidate at all. Every entry in it needs an environment, which is the third
condition failing by construction, and `docs/quality-parity.md` records that
where the parity map is argued.

The headless gate issue #7 delivers meets the first and third conditions and
carries its own proof, so it is eligible on the second. Its check-run name is
absent from the table for the same reason: the table quotes names read from
completed runs, and it has none yet. It joins the table above in the same change that adds it to the
ruleset, once it has one.

## Changing the set

Adding a check to the required set changes this page and the ruleset together,
in one pull request, and the page says which of the three conditions the check
now meets that it did not meet before. Removing one is the
same in reverse and says what stopped being true.

Nothing enforces that coupling. No job reads the ruleset and compares it against
this table, so a required set changed without a change to this file leaves no
trace, and the commands above are how a reader checks it instead of trusting
it.

## The claims not being made here

The ruleset is unchanged by the pull request that added this file. What is
written here is the decision and the reasoning; the required set is still empty
when read, and it stays empty until somebody with the rights over that setting
applies it.

Nor does it claim the six are enough. They are the ones that meet the rule today.
A check that meets the rule tomorrow belongs in the table the day its proof
lands, and the largest single gap is that two checks which refuse real things
sit outside the set only because nobody has planted the mistake yet.

One thing measured rather than supposed: on the day this was written, seven of
these checks went red on one commit without executing a step, because the action
service could not resolve a download. That is the third condition failing for
every check at once rather than for any one of them, and it is the argument for
re-running a red required check before believing it, not an argument against
requiring any of these six.
