#!/usr/bin/env bash
# Prove that the code scanning gate bites, that the tracked register is what
# makes it bite, and that the declared threshold is what decides which findings
# fail rather than merely being reported.
#
# Usage:
#   .github/scripts/prove-code-scanning.sh
#
# Each property gets a leg that plants exactly the construct it is about and
# requires a refusal naming the file and the lint, and a neighbouring leg that
# changes the one thing back and requires a clean run. A gate that refuses
# everything fails the second kind; one that refuses nothing fails the first.
#
# The third kind of leg is the one worth arguing for, and it is the same
# argument `.github/scripts/prove-format-and-lint.sh` makes: it plants the
# construct and takes the record out of the register, or moves the threshold in
# the parity document, and requires the verdict to move with it. Without those,
# a leg that passes proves only that some default somewhere refuses the
# construct, which is not what the tracked lines claim.
#
# The subject is HEAD, not the working tree, so the legs judge the commit a
# reader will have. Commit before running this.

set -euo pipefail

repo=$(cd "$(dirname "$0")/../.." && pwd)

command -v cargo >/dev/null 2>&1 || { echo "no cargo on PATH" >&2; exit 2; }
git -C "$repo" rev-parse --verify HEAD >/dev/null 2>&1 || { echo "no HEAD in $repo" >&2; exit 2; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# One target directory for every leg. The workspace has no dependencies outside
# itself, so this costs nothing in correctness and saves a rebuild per leg.
export CARGO_TARGET_DIR="$work/target"

cases=0
failures=0

# tree <name>: HEAD, unpacked. The copy carries this repository's own register,
# its own parity document and its own manifests, because those are the things
# under test. A fixture writing its own would prove the fixture.
tree() {
  local name=$1
  local dir="$work/$name"
  mkdir -p "$dir"
  git -C "$repo" archive --format=tar HEAD | tar -xf - -C "$dir"
  printf '%s' "$dir"
}

# plant <dir> <body>: a public module in a library on the parsing side, so the
# construct is inside `--lib` and any diagnostic about it names the file.
plant() {
  local dir=$1 body=$2
  printf '%s' "$body" > "$dir/crates/calc/src/probe.rs"
  printf '\npub mod probe;\n' >> "$dir/crates/calc/src/lib.rs"
}

# plant_test <dir> <body>: the same construct in an integration test instead,
# which is outside what this gate reads and is the bound the check discloses.
plant_test() {
  local dir=$1 body=$2
  mkdir -p "$dir/crates/calc/tests"
  printf '%s' "$body" > "$dir/crates/calc/tests/probe.rs"
}

# unregister <dir> <lint>: remove that lint's whole record from the register, so
# a leg can ask what the record was doing.
unregister() {
  local dir=$1 lint=$2
  local file="$dir/.github/scripts/code-scanning-lints.txt"
  grep -q "^Lint: $lint\$" "$file" || {
    echo "FAIL  the register carries no record for $lint; this proof is out of date" >&2
    failures=$((failures + 1))
    return
  }
  awk -v lint="$lint" '
    function emit() { if (buf != "" && !drop) printf "%s", buf; buf = ""; drop = 0 }
    /^[[:space:]]*$/ { buf = buf $0 "\n"; emit(); next }
    { buf = buf $0 "\n"; if ($0 == "Lint: " lint) drop = 1 }
    END { emit() }
  ' "$file" > "$file.next"
  mv "$file.next" "$file"
}

# retune <dir> <severity>: rewrite the threshold the parity document declares, so
# a leg can ask whether the threshold is deciding anything.
retune() {
  local dir=$1 severity=$2
  local file="$dir/docs/quality-parity.md"
  grep -q '^Code scanning threshold: ' "$file" || {
    echo "FAIL  the parity document declares no threshold; this proof is out of date" >&2
    failures=$((failures + 1))
    return
  }
  sed "s/^Code scanning threshold: .*/Code scanning threshold: $severity/" "$file" > "$file.next"
  mv "$file.next" "$file"
}

# untune <dir>: take the threshold line out entirely.
untune() {
  local dir=$1
  local file="$dir/docs/quality-parity.md"
  grep -v '^Code scanning threshold: ' "$file" > "$file.next"
  mv "$file.next" "$file"
}

# expect <dir> <label> <clean|refused|unrunnable> [<substring>...]
expect() {
  local dir=$1 label=$2 want=$3
  shift 3
  local out status
  cases=$((cases + 1))
  set +e
  out=$(cd "$dir" && bash .github/scripts/check-code-scanning.sh . 2>&1)
  status=$?
  set -e

  local wanted_status
  case "$want" in
    clean) wanted_status=0 ;;
    refused) wanted_status=1 ;;
    unrunnable) wanted_status=2 ;;
    *) echo "unknown expectation $want" >&2; exit 2 ;;
  esac

  local missing=""
  local needle
  for needle in "$@"; do
    printf '%s\n' "$out" | grep -qF -- "$needle" || missing="$missing [$needle]"
  done

  if [ "$status" -eq "$wanted_status" ] && [ -z "$missing" ]; then
    if [ "$#" -eq 0 ]; then
      echo "ok    $label is $want"
    else
      echo "ok    $label is $want, naming$(printf ' [%s]' "$@")"
    fi
    return
  fi

  if [ "$status" -ne "$wanted_status" ]; then
    echo "FAIL  $label should be $want (exit $wanted_status), got exit $status"
  else
    echo "FAIL  $label is $want but does not name:$missing"
  fi
  printf '%s\n' "$out" | sed 's/^/      /'
  failures=$((failures + 1))
}

# The constructs. Each is the smallest thing that trips exactly one lint.

aborts='pub fn probe(v: Option<i32>) -> i32 {
    v.unwrap()
}
'

does_not_abort='pub fn probe(v: Option<i32>) -> i32 {
    match v {
        Some(n) => n,
        None => 0,
    }
}
'

# A width losing its high bits, which the register places below the threshold.
# The construct has to trip exactly one lint for the two legs below to say what
# they claim: an integer division, for instance, is also arithmetic that can
# abort, so it would be refused for a reason neither leg is about.
truncates='pub fn probe(width: u64) -> u32 {
    width as u32
}
'

silenced_without_a_reason='#[allow(clippy::unwrap_used)]
pub fn probe(v: Option<i32>) -> i32 {
    v.unwrap()
}
'

silenced_with_a_reason='#[allow(
    clippy::unwrap_used,
    reason = "the leg beside this one is what says the reason is what makes the difference"
)]
pub fn probe(v: Option<i32>) -> i32 {
    v.unwrap()
}
'

# --- the tree as it stands ----------------------------------------------------
d=$(tree as-committed)
expect "$d" "the tree with nothing planted in it" clean \
  "the shipped code carries nothing at or above"

# --- a finding at or above the threshold --------------------------------------
d=$(tree aborts); plant "$d" "$aborts"
expect "$d" "an abort on a value a document controls" refused \
  "REFUSED finding-at-or-above-the-threshold" "probe.rs" "clippy::unwrap_used"

d=$(tree aborts-repaired); plant "$d" "$does_not_abort"
expect "$d" "the same function handling the missing value" clean

# The register is what refuses, rather than a default of the analyser. With the
# record gone the same construct has to pass, and a red run here would mean the
# tracked line was decorative.
d=$(tree aborts-unregistered); plant "$d" "$aborts"; unregister "$d" "clippy::unwrap_used"
expect "$d" "the same abort with its record taken out of the register" clean

# --- a finding below the threshold --------------------------------------------
# Reported and uploaded, and not a refusal. This is what the threshold buys, and
# a gate that refused this would be a gate with no threshold in it.
d=$(tree below); plant "$d" "$truncates"
expect "$d" "a width losing its high bits, which the register puts below the threshold" clean \
  "reported below the threshold" "clippy::cast_possible_truncation"

# The threshold is what decides. The same construct, with the parity document
# lowered by one rung, has to be refused.
d=$(tree below-retuned); plant "$d" "$truncates"; retune "$d" low
expect "$d" "the same conversion with the parity document declaring low" refused \
  "REFUSED finding-at-or-above-the-threshold" "clippy::cast_possible_truncation"

# --- what the gate does not read ----------------------------------------------
# The same abort in a test passes, because the subject is the shipped code. This
# leg exists so the bound is proved rather than asserted in a comment.
d=$(tree aborts-in-a-test); plant_test "$d" "$aborts"
expect "$d" "the same abort written in an integration test" clean

# --- suppressions -------------------------------------------------------------
d=$(tree silenced); plant "$d" "$silenced_without_a_reason"
expect "$d" "a lint silenced with no reason beside it" refused \
  "REFUSED suppression-without-a-reason" "probe.rs"

d=$(tree silenced-with-a-reason); plant "$d" "$silenced_with_a_reason"
expect "$d" "the same suppression carrying its reason" clean

# --- the check stopping rather than judging -----------------------------------
# A gate that cannot read its own inputs must not report a clean tree. Each of
# these is a failure and never a pass.
d=$(tree no-threshold); untune "$d"
expect "$d" "a parity document declaring no threshold" unrunnable \
  "declares no threshold this gate can read"

d=$(tree bad-threshold); retune "$d" critical
expect "$d" "a parity document declaring a severity outside the vocabulary" unrunnable \
  "which is not one of high, medium or low"

d=$(tree bad-severity)
sed 's/^Severity: high$/Severity: catastrophic/' "$d/.github/scripts/code-scanning-lints.txt" > "$d/reg.next"
mv "$d/reg.next" "$d/.github/scripts/code-scanning-lints.txt"
expect "$d" "a register record carrying a severity outside the vocabulary" unrunnable \
  "cannot be placed against the threshold"

d=$(tree unreadable-register)
printf '\nthis line starts with no field name\n' >> "$d/.github/scripts/code-scanning-lints.txt"
expect "$d" "a register holding a line no field name starts" unrunnable \
  "refusing to judge a tree against a register it could not read"

d=$(tree no-register)
rm -f "$d/.github/scripts/code-scanning-lints.txt"
expect "$d" "a tree with no register at all" unrunnable "no register at"

echo
echo "$cases case(s), $failures failure(s)"
[ "$failures" -eq 0 ] || exit 1
