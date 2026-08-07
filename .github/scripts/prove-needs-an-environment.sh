#!/usr/bin/env bash
# Prove that the impure register's checker bites, and that the runner refuses to
# run anything it was not told to run.
#
# Usage:
#   .github/scripts/prove-needs-an-environment.sh
#
# Every refusal gets two legs. The first builds a register holding exactly the
# thing the property is about and requires that property and NO other, so a
# checker that refuses everything cannot pass. The second changes the one thing
# back and requires no refusal at all, so a checker that refuses nothing cannot
# pass either.
#
# The registers are written out here in full rather than derived from a clean one
# by substitution, and rather than read from
# `.github/scripts/needs-an-environment.txt`. Reading the real register would
# prove the state of the tree on the day it ran instead of proving the checker.
# Deriving them by substitution was tried and reverted: the difference between a
# leg and its neighbour then lives inside a quoting expression rather than in the
# text, and a reviewer cannot see what a leg is about without running it.
#
# Each register is a variable passed as an argument, which is the shape
# `.github/scripts/prove-invariants.sh` already uses for the same job.
#
# The runner's own legs are at the bottom, and they are the same rule from the
# other side: a harness nobody can start by accident, whose output cannot be
# mistaken for the pure suite's.
#
# THIS FILE IS PURE. It needs no container runtime and no network. The entries it
# writes run a stand-in script and `false`, because what is under test is the
# register and the runner and never the sealed environment the real entries need.
# That is why this proof is an ordinary gate while the harness it is about is not.

set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
checker="$here/check-needs-an-environment.sh"
runner="$here/needs-an-environment.sh"
[ -f "$checker" ] || { echo "no checker at $checker" >&2; exit 2; }
[ -f "$runner" ] || { echo "no runner at $runner" >&2; exit 2; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

cases=0
failures=0

# register <name> <text>
#
# Each call also plants the one script the records name, so a leg about a missing
# `Because` is not accidentally also a leg about a missing script.
register() {
  local dir="$work/$1"
  mkdir -p "$dir/.github/scripts"
  printf '%s' "$2" > "$dir/.github/scripts/needs-an-environment.txt"
  printf '#!/usr/bin/env bash\nexit 0\n' > "$dir/.github/scripts/stand-in.sh"
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
  output=$("$checker" "$dir" 2>&1) || status=$?
  got=$(printf '%s\n' "$output" | sed -n 's/^REFUSED \([a-z-]*\):.*/\1/p' | sort | tr '\n' ' ')
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
ID='Id: stand-in
'
NEEDS='Needs: nothing at all, which is what makes this a fixture rather than an entry
'
BECAUSE='Because: a leg needs a record the checker passes, so its neighbour can change one thing.
'
RUN='Run: bash .github/scripts/stand-in.sh
'
# The three mutated lines, each written out beside the line it replaces so a
# reader sees the one character that differs. Every one of these ends in its own
# newline: a leg that spliced a bare line into the middle would insert a blank
# line, which splits one record into two and turns the leg into a leg about
# something else.
BECAUSE_EMPTY='Because:
'
RUN_MISSPELLED='Run: bash .github/scripts/stand-inn.sh
'
FIELD_THAT_DOES_NOT_EXIST='Reason: this field name does not exist
'

# ------------------------------------------------------------------ the checker

register clean "$ID$NEEDS$BECAUSE$RUN"
expect clean "" 0

register no-reason "$ID$NEEDS$RUN"
expect no-reason "entry-without-a-reason " 1

register empty-reason "$ID$NEEDS$BECAUSE_EMPTY$RUN"
expect empty-reason "entry-without-a-reason " 1

register no-need "$ID$BECAUSE$RUN"
expect no-need "entry-without-a-need " 1

register no-script "$ID$NEEDS$BECAUSE$RUN_MISSPELLED"
expect no-script "entry-names-no-script " 1

register script-spelled-right "$ID$NEEDS$BECAUSE$RUN"
expect script-spelled-right "" 0

register duplicate "$ID$NEEDS$BECAUSE$RUN
$ID$NEEDS$BECAUSE$RUN"
expect duplicate "duplicate-entry-id " 1

register not-duplicate "$ID$NEEDS$BECAUSE$RUN
Id: stand-in-again
$NEEDS$BECAUSE$RUN"
expect not-duplicate "" 0

# A register carrying two different mistakes reports both. A checker that stopped
# at the first would leave the second to be found on the run after the repair.
register two-mistakes "$ID$BECAUSE$RUN
Id: stand-in-again
$NEEDS$RUN"
expect two-mistakes "entry-without-a-need entry-without-a-reason " 1

# ------------------------------------------------ the shapes that stop the run

# expect_unjudgeable <name>
expect_unjudgeable() {
  local name=$1 status=0
  cases=$((cases + 1))
  "$checker" "$work/$name" >/dev/null 2>&1 || status=$?
  if [ "$status" -ne 2 ]; then
    echo "FAILED $name is unjudgeable"
    echo "  expected exit 2, got $status"
    failures=$((failures + 1))
    return 0
  fi
  echo "ok      $name is unjudgeable"
}

register comments-only '# nothing but this
'
expect_unjudgeable comments-only

register unreadable-field "$ID$NEEDS$FIELD_THAT_DOES_NOT_EXIST$BECAUSE$RUN"
expect_unjudgeable unreadable-field

register no-id "$NEEDS$BECAUSE$RUN"
expect_unjudgeable no-id

register no-run "$ID$NEEDS$BECAUSE"
expect_unjudgeable no-run

expect_unjudgeable no-register-at-all

# ------------------------------------------------------------------- the runner

register runner 'Id: writes-a-file
Needs: nothing, which is what lets this leg run anywhere
Because: a leg proving the runner runs the entry it was named needs one with an observable effect.
Run: bash .github/scripts/stand-in.sh && : > ran.txt

Id: fails
Needs: nothing
Because: a leg proving the runner reports a failed entry as failed needs one that fails.
Run: false
'
runner_root="$work/runner"

# note <label> yes|no
note() {
  cases=$((cases + 1))
  if [ "$2" = "yes" ]; then
    echo "ok      $1"
  else
    echo "FAILED $1"
    failures=$((failures + 1))
  fi
}

# holds <text> <needle>
holds() {
  case "$1" in
    *"$2"*) echo yes ;;
    *) echo no ;;
  esac
}

ran="$runner_root/ran.txt"

# No argument. This is the near-miss the whole file is aimed at: a runner that
# did something sensible with no argument is one a contributor runs by accident.
rm -f "$ran"
bare_status=0
bare=$("$runner" "" "$runner_root" 2>&1) || bare_status=$?
note "no argument exits 2" "$([ "$bare_status" -eq 2 ] && echo yes || echo no)"
note "no argument runs nothing" "$([ -e "$ran" ] && echo no || echo yes)"
note "no argument prints the register" "$(holds "$bare" writes-a-file)"

unknown_status=0
unknown=$("$runner" not-an-entry "$runner_root" 2>&1) || unknown_status=$?
note "an unknown id exits 2" "$([ "$unknown_status" -eq 2 ] && echo yes || echo no)"
note "an unknown id runs nothing" "$([ -e "$ran" ] && echo no || echo yes)"
note "an unknown id prints the ids that do exist" "$(holds "$unknown" writes-a-file)"

list_status=0
listed=$("$runner" list "$runner_root" 2>&1) || list_status=$?
note "list exits 0" "$([ "$list_status" -eq 0 ] && echo yes || echo no)"
note "list runs nothing" "$([ -e "$ran" ] && echo no || echo yes)"
note "list prints what each entry needs" "$(holds "$listed" 'needs   nothing')"

passed_status=0
passed=$("$runner" writes-a-file "$runner_root" 2>&1) || passed_status=$?
note "a named entry exits 0 when its command succeeds" \
  "$([ "$passed_status" -eq 0 ] && echo yes || echo no)"
note "the named entry actually ran" "$([ -e "$ran" ] && echo yes || echo no)"

# The record. A result with no marker and no environment is one somebody can
# paste beside a pure-suite number, which is the failure this half is about.
for wanted in 'needs-an-environment:' 'result record begins' 'commit ' 'host ' \
  'may not be quoted as one' 'writes-a-file PASSED'; do
  note "the result record carries \`$wanted\`" "$(holds "$passed" "$wanted")"
done

failed_status=0
failed=$("$runner" fails "$runner_root" 2>&1) || failed_status=$?
note "a named entry whose command fails exits 1" \
  "$([ "$failed_status" -eq 1 ] && echo yes || echo no)"
note "a failed entry says so in its record" "$(holds "$failed" 'fails FAILED')"

echo "$cases case(s), $failures failure(s)"
[ "$failures" -eq 0 ] || exit 1
