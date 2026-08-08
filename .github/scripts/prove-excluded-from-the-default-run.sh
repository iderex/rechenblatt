#!/usr/bin/env bash
# Prove that the exclusion register's checker bites, and bites for the reason it
# names.
#
# Usage:
#   .github/scripts/prove-excluded-from-the-default-run.sh
#
# Every refusal gets two legs. The first builds a tree holding exactly the thing
# the property is about and requires that property and NO other, so a checker
# that refuses everything cannot pass. The second changes the one thing back and
# requires no refusal at all, so a checker that refuses nothing cannot pass
# either.
#
# The trees are written out here in full rather than derived from a clean one by
# substitution, and rather than read from the repository's own register and
# sources. Reading those would prove the state of the tree on the day it ran
# instead of proving the checker, which is the same reason
# `.github/scripts/prove-needs-an-environment.sh` gives for the same choice.
#
# Two legs are worth naming before they are read, because they are the near-miss
# rather than the obvious mistake. `attribute-removed` is a test that came BACK
# into the default run and left its record behind, which is the failure a
# register that only reads itself cannot see. And `nothing-excluded` is an empty
# register over a tree that excludes nothing, which has to pass: a check that
# treated an empty register as unjudgeable would make the honest state the red
# one.
#
# THIS FILE IS PURE. It compiles nothing, runs no cargo and needs no container.
# What is under test is a checker that reads two files and a directory of
# sources, so the sources it writes are four-line stand-ins and never anything
# that has to build.

set -euo pipefail

# Invoked as `bash <path>` rather than executed, because every script in this
# directory is tracked without an executable bit and every caller including the
# workflow reaches them the same way. Executing one directly works on a
# filesystem that ignores the mode and exits 126 on one that does not, which is
# a failure that appears only on the gate.
here=$(cd "$(dirname "$0")" && pwd)
checker="$here/check-excluded-from-the-default-run.sh"
[ -f "$checker" ] || { echo "no checker at $checker" >&2; exit 2; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

cases=0
failures=0

# The harness register every leg is judged against, so a leg about a missing
# `Because` is not accidentally also a leg about a `Runs-in` that resolves
# nowhere. It holds one id and needs nothing to be true of the world.
HARNESS='Id: stand-in
Needs: nothing at all, which is what makes this a fixture rather than an entry
Because: a leg needs an entry a record can name, so its neighbour can change one thing.
Run: bash .github/scripts/stand-in.sh
'

# The sources. Each is a whole file rather than a fragment, because what the
# checker walks is files.
SOURCE_ONE_IGNORED='#[test]
#[ignore = "a stand-in, so a leg has something to be about"]
fn probe_stands_in() {}
'
SOURCE_ONE_RUNNING='#[test]
fn probe_stands_in() {}
'
SOURCE_TWO_IGNORED='#[test]
#[ignore = "a stand-in, so a leg has something to be about"]
fn probe_stands_in() {}

#[test]
#[ignore = "a second stand-in, for the legs that need two"]
fn probe_stands_in_too() {}
'
SOURCE_NONE_IGNORED='#[test]
fn a_test_the_default_run_does_run() {}
'

# tree <name> <register-text> [<source-text>] [<harness-text>]
#
# An absent source means the tree has no Rust in it at all; an absent harness
# means the harness register is written with the one stand-in entry above.
tree() {
  local dir="$work/$1"
  mkdir -p "$dir/.github/scripts" "$dir/crates/model/tests"
  printf '%s' "$2" > "$dir/.github/scripts/excluded-from-the-default-run.txt"
  printf '%s' "${4-$HARNESS}" > "$dir/.github/scripts/needs-an-environment.txt"
  printf '#!/usr/bin/env bash\nexit 0\n' > "$dir/.github/scripts/stand-in.sh"
  if [ -n "${3-}" ]; then
    printf '%s' "$3" > "$dir/crates/model/tests/environment.rs"
  fi
}

# expect <name> <expected-properties> <expected-exit>
#
# The properties are compared as a whole sorted set rather than searched for, so
# a checker that refuses everything fails the clean legs, and one that refuses
# every mistake under a single name fails the exact ones.
expect() {
  local name=$1 want=$2 want_exit=$3
  local dir="$work/$name" output status=0 got
  cases=$((cases + 1))
  output=$(bash "$checker" "$dir" 2>&1) || status=$?
  got=$(printf '%s\n' "$output" | sed -n 's/^REFUSED \([a-z-]*\):.*/\1/p' | sort -u | tr '\n' ' ')
  if [ "$got" != "$want" ] || [ "$status" -ne "$want_exit" ]; then
    echo "FAILED $name"
    echo "  expected refusals [$want] and exit $want_exit"
    echo "  got      refusals [$got] and exit $status"
    printf '%s\n' "$output" | sed 's/^/    /'
    failures=$((failures + 1))
    return 0
  fi
  echo "ok      $name"
}

# The four field lines, so a leg below reads as the one thing it changes.
TEST='Test: probe_stands_in
'
IN='In: crates/model/tests/environment.rs
'
BECAUSE='Because: a leg needs a record the checker passes, so its neighbour can change one thing.
'
RUNS_IN='Runs-in: stand-in
'

# The mutated lines, each written out beside the line it replaces so a reader
# sees the one thing that differs. Every one ends in its own newline: a leg that
# spliced a bare line into the middle would insert a blank line, which splits one
# record into two and turns the leg into a leg about something else.
BECAUSE_EMPTY='Because:
'
RUNS_IN_EMPTY='Runs-in:
'
RUNS_IN_ABSENT='Runs-in: an-entry-the-harness-does-not-hold
'
IN_MISSPELLED='In: crates/model/tests/environmnet.rs
'
FIELD_THAT_DOES_NOT_EXIST='Reason: this field name does not exist
'

# The second record, for the legs that need two.
SECOND='Test: probe_stands_in_too
In: crates/model/tests/environment.rs
Because: a second record, so a leg can hold two mistakes or two tests.
Runs-in: stand-in
'

# --------------------------------------------------------- the reason and the runner

tree clean "$TEST$IN$BECAUSE$RUNS_IN" "$SOURCE_ONE_IGNORED"
expect clean "" 0

tree no-reason "$TEST$IN$RUNS_IN" "$SOURCE_ONE_IGNORED"
expect no-reason "exclusion-without-a-reason " 1

tree empty-reason "$TEST$IN$BECAUSE_EMPTY$RUNS_IN" "$SOURCE_ONE_IGNORED"
expect empty-reason "exclusion-without-a-reason " 1

tree reason-back "$TEST$IN$BECAUSE$RUNS_IN" "$SOURCE_ONE_IGNORED"
expect reason-back "" 0

tree no-runner "$TEST$IN$BECAUSE" "$SOURCE_ONE_IGNORED"
expect no-runner "exclusion-runs-nowhere " 1

tree empty-runner "$TEST$IN$BECAUSE$RUNS_IN_EMPTY" "$SOURCE_ONE_IGNORED"
expect empty-runner "exclusion-runs-nowhere " 1

# The near-miss on this half: a `Runs-in` that is spelled, is plausible, and
# names an entry the harness register does not hold. The test is then out of the
# default run and out of the harness both, which is the shape this whole file
# exists against.
tree runner-not-in-the-harness "$TEST$IN$BECAUSE$RUNS_IN_ABSENT" "$SOURCE_ONE_IGNORED"
expect runner-not-in-the-harness "exclusion-runs-nowhere " 1

tree runner-back "$TEST$IN$BECAUSE$RUNS_IN" "$SOURCE_ONE_IGNORED"
expect runner-back "" 0

# ------------------------------------------------------- the two sides agreeing

# One character in a path. The record still names a test that exists and is
# still recorded, so this is the record having drifted off its source and
# nothing else.
tree wrong-source "$TEST$IN_MISSPELLED$BECAUSE$RUNS_IN" "$SOURCE_ONE_IGNORED"
expect wrong-source "record-names-no-test " 1

tree source-back "$TEST$IN$BECAUSE$RUNS_IN" "$SOURCE_ONE_IGNORED"
expect source-back "" 0

# The near-miss the register cannot see by reading itself: the attribute was
# taken off, so the test is back in the default run and its record is describing
# an exclusion that no longer exists.
tree attribute-removed "$TEST$IN$BECAUSE$RUNS_IN" "$SOURCE_ONE_RUNNING"
expect attribute-removed "record-names-no-test " 1

# The other direction, and the one that stops the set falling behind the code: a
# second test left the default run and nobody wrote it down.
tree unrecorded "$TEST$IN$BECAUSE$RUNS_IN" "$SOURCE_TWO_IGNORED"
expect unrecorded "test-excluded-without-a-record " 1

tree both-recorded "$TEST$IN$BECAUSE$RUNS_IN
$SECOND" "$SOURCE_TWO_IGNORED"
expect both-recorded "" 0

# A tree that excludes nothing, with a register holding nothing. This has to
# pass. A checker that treated an empty register as unjudgeable would make the
# honest state the red one and would teach somebody to write a record for a test
# that does not exist.
tree nothing-excluded '# nothing is excluded from the default run in this tree
' "$SOURCE_NONE_IGNORED"
expect nothing-excluded "" 0

# ------------------------------------------------------------------ duplicates

tree duplicate "$TEST$IN$BECAUSE$RUNS_IN
$TEST$IN$BECAUSE$RUNS_IN" "$SOURCE_ONE_IGNORED"
expect duplicate "duplicate-exclusion " 1

tree not-duplicate "$TEST$IN$BECAUSE$RUNS_IN
$SECOND" "$SOURCE_TWO_IGNORED"
expect not-duplicate "" 0

# A register carrying two different mistakes reports both. A checker that
# stopped at the first would leave the second to be found on the run after the
# repair.
tree two-mistakes "$TEST$IN$RUNS_IN
Test: probe_stands_in_too
In: crates/model/tests/environment.rs
Because: the second record keeps its reason and loses its runner.
" "$SOURCE_TWO_IGNORED"
expect two-mistakes "exclusion-runs-nowhere exclusion-without-a-reason " 1

# ------------------------------------------------ the shapes that stop the run

# expect_unjudgeable <name>
expect_unjudgeable() {
  local name=$1 status=0
  cases=$((cases + 1))
  bash "$checker" "$work/$name" >/dev/null 2>&1 || status=$?
  if [ "$status" -ne 2 ]; then
    echo "FAILED $name is unjudgeable"
    echo "  expected exit 2, got $status"
    failures=$((failures + 1))
    return 0
  fi
  echo "ok      $name is unjudgeable"
}

tree unreadable-field "$TEST$IN$FIELD_THAT_DOES_NOT_EXIST$BECAUSE$RUNS_IN" "$SOURCE_ONE_IGNORED"
expect_unjudgeable unreadable-field

tree no-test "$IN$BECAUSE$RUNS_IN" "$SOURCE_ONE_IGNORED"
expect_unjudgeable no-test

tree no-source "$TEST$BECAUSE$RUNS_IN" "$SOURCE_ONE_IGNORED"
expect_unjudgeable no-source

# No harness register at all. `Runs-in` cannot be judged against a register that
# is not there, and a clean report from a check that judged half of what it
# names is the one output this must not produce.
tree no-harness "$TEST$IN$BECAUSE$RUNS_IN" "$SOURCE_ONE_IGNORED" ''
rm -f "$work/no-harness/.github/scripts/needs-an-environment.txt"
expect_unjudgeable no-harness

mkdir -p "$work/no-register-at-all"
expect_unjudgeable no-register-at-all

echo "$cases case(s), $failures failure(s)"
[ "$failures" -eq 0 ] || exit 1
