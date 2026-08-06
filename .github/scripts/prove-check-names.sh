#!/usr/bin/env bash
# Prove that check-check-names.sh bites, once per property it names.
#
# Usage:
#   .github/scripts/prove-check-names.sh
#
# Each property gets two legs. The first builds a tree holding exactly the
# mistake the property is about and requires the checker to refuse THAT property
# and no other, so a checker refusing everything cannot pass this. The second
# changes the one thing back and requires the checker to refuse nothing, so a
# checker refusing nothing cannot pass either.
#
# The workflows and the document these legs are judged against are written here
# rather than copied from the real ones. A proof that reads the real tree proves
# the state of the tree on the day it ran; this proves the checker.

set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
checker="$here/check-check-names.sh"
[ -f "$checker" ] || { echo "no checker at $checker" >&2; exit 2; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

cases=0
failures=0

# make_tree <name>: an empty root with the two directories the checker reads.
make_tree() {
  local name=$1
  local dir="$work/$name"
  mkdir -p "$dir/.github/workflows" "$dir/docs"
  printf '%s' "$dir"
}

# workflow <dir> <file>; body on stdin.
workflow() {
  local dir=$1 file=$2
  cat > "$dir/.github/workflows/$file"
}

# document <dir>; the rows, one name per argument.
document() {
  local dir=$1
  shift
  {
    printf '# The checks\n\n'
    printf '| Check | What reproduces it here |\n'
    printf '| --- | --- |\n'
    local name
    for name in "$@"; do
      printf '| `%s` | something |\n' "$name"
    done
  } > "$dir/docs/checks.md"
}

# expect <dir> <label> <property|clean>
expect() {
  local dir=$1 label=$2 want=$3
  local out status refusals
  cases=$((cases + 1))
  set +e
  out=$(bash "$checker" "$dir" 2>&1)
  status=$?
  set -e
  refusals=$(printf '%s\n' "$out" | grep '^REFUSED ' || true)

  if [ "$want" = "clean" ]; then
    if [ "$status" -eq 0 ] && [ -z "$refusals" ]; then
      echo "ok    $label refuses nothing"
      return
    fi
    echo "FAIL  $label should refuse nothing, got status $status"
    printf '%s\n' "$out" | sed 's/^/      /'
    failures=$((failures + 1))
    return
  fi

  local wanted_lines other
  wanted_lines=$(printf '%s\n' "$refusals" | grep -c "^REFUSED $want: " || true)
  other=$(printf '%s\n' "$refusals" | grep -v "^REFUSED $want: " | grep '^REFUSED ' || true)
  if [ "$status" -eq 1 ] && [ "$wanted_lines" -ge 1 ] && [ -z "$other" ]; then
    echo "ok    $label refuses exactly $want"
    return
  fi
  echo "FAIL  $label should refuse exactly $want, got status $status"
  printf '%s\n' "$out" | sed 's/^/      /'
  failures=$((failures + 1))
}

named_job='name: example

on:
  pull_request:

jobs:
  build:
    name: Build the thing
    runs-on: ubuntu-latest
    steps:
      - name: A step whose name is not a check name
        run: true
'

unnamed_job='name: example

on:
  pull_request:

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: A step whose name is not a check name
        run: true
'

# Every leg below carries a second workflow that is correct and documented, so
# the tree under test differs from a clean one in exactly one thing. A fixture
# holding a single job cannot do that: taking its name away also leaves the row
# describing it unproduced, and the leg would then pass on two refusals without
# saying which one it meant.

# --- job-without-a-name -------------------------------------------------------
# The one-character mistake somebody actually makes: the `name:` line is dropped
# in a tidy-up and the check quietly starts answering to the job id.
d=$(make_tree noname-bad)
printf '%s' "$named_job" | workflow "$d" one.yml
printf '%s' "$unnamed_job" | workflow "$d" two.yml
document "$d" "Build the thing"
expect "$d" "a job with no name of its own" job-without-a-name

d=$(make_tree noname-good)
printf '%s' "$named_job" | workflow "$d" one.yml
printf '%s' "${named_job/Build the thing/Test the thing}" | workflow "$d" two.yml
document "$d" "Build the thing" "Test the thing"
expect "$d" "the same job with a name back" clean

# --- check-not-documented -----------------------------------------------------
# A job renamed in the workflow and nowhere else. This is the failure the whole
# checker exists for.
d=$(make_tree undocumented-bad)
printf '%s' "$named_job" | workflow "$d" one.yml
printf '%s' "${named_job/Build the thing/Test the thing}" | workflow "$d" two.yml
document "$d" "Build the thing"
expect "$d" "a second job the document does not carry" check-not-documented

d=$(make_tree undocumented-good)
printf '%s' "$named_job" | workflow "$d" one.yml
printf '%s' "${named_job/Build the thing/Test the thing}" | workflow "$d" two.yml
document "$d" "Build the thing" "Test the thing"
expect "$d" "the same pair with both rows present" clean

# --- documented-check-is-absent -----------------------------------------------
# The other direction: a row describing a gate that is not running. A reader
# takes that row as an assurance, and there is nothing behind it.
d=$(make_tree absent-bad)
printf '%s' "$named_job" | workflow "$d" one.yml
document "$d" "Build the thing" "A gate nobody runs"
expect "$d" "a documented name no job produces" documented-check-is-absent

d=$(make_tree absent-good)
printf '%s' "$named_job" | workflow "$d" one.yml
document "$d" "Build the thing"
expect "$d" "the same document without that row" clean

# --- the parse is not fooled by a step ----------------------------------------
# A step's `name:` sits deeper than a job's, and reading one as a check name
# would refuse a tree that is correct. Every leg above already carries a named
# step; this one names the step something no row carries, so a parse that read
# it as a job would go red here.
d=$(make_tree step-name)
printf '%s' "${named_job/A step whose name is not a check name/Nothing documents me}" | workflow "$d" one.yml
document "$d" "Build the thing"
expect "$d" "a step name that is not a check name" clean

echo
echo "$cases case(s), $failures failure(s)"
[ "$failures" -eq 0 ] || exit 1
