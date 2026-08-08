#!/usr/bin/env bash
# Refuse a difference between the tests the default run excludes and the tests
# this repository says it excludes.
#
# Usage:
#   .github/scripts/check-excluded-from-the-default-run.sh [repository-root]
#
# The default run is `cargo test --locked --workspace`. A test carrying
# `#[ignore]` is not in it, and that is one line in a source with no symptom: the
# run stays green, the count drops by one, and nobody reads a count. A test can
# therefore leave the default suite and be picked up by nothing, which is not an
# exclusion but a deletion that left its source behind.
#
# So the set is written down in `.github/scripts/excluded-from-the-default-run.txt`
# and this reads both sides. It runs in the ordinary gate on every machine,
# because it reads files and needs nothing.
#
# Five refusals:
#
#   exclusion-without-a-reason      a record with no `Because`, so nothing says
#                                   why the test is out of the default run and
#                                   the register becomes the place awkward tests
#                                   are put
#   exclusion-runs-nowhere          a record whose `Runs-in` is empty or names an
#                                   entry `.github/scripts/needs-an-environment.txt`
#                                   does not hold, which is a test excluded from
#                                   the default run and from the harness both
#   record-names-no-test            a record whose `In` holds no `#[ignore]`d
#                                   function of that name, so the register has
#                                   drifted off the source it describes
#   test-excluded-without-a-record  an `#[ignore]`d function in the tree that no
#                                   record names, which is the direction that
#                                   stops the set falling behind the code
#   duplicate-exclusion             two records for one test name, where the
#                                   second says nothing the first did not and
#                                   the two can disagree
#
# Exit 0 means the two sides agree and every record carries its reason and its
# runner. Exit 1 means at least one refusal. Exit 2 means the check could not
# run, which is a failure and never a pass, and covers a missing register, a
# register holding a line no field name starts, a record naming no test or no
# source, and a missing harness register - `Runs-in` cannot be judged against a
# register that is not there, and reporting a clean set without having judged it
# is the one output this check may not produce.
#
# WHAT THIS CANNOT JUDGE. It reads `#[ignore]` and nothing else, because that is
# the mechanism this repository excludes a test with. A test that never reaches
# the runner at all - one behind a `cfg` feature nothing enables, one in a file
# no module declares, one whose harness is not registered in a manifest - is
# outside the default run and invisible here. Those are exclusions this check
# does not see, and `docs/excluded-from-the-default-run.md` carries that residual
# where a reader meets it rather than leaving it to be discovered.
#
# It also cannot judge whether a `Because` is true, or whether the entry named by
# `Runs-in` really runs the test rather than merely existing. Both are judgements
# about meaning that no reading of the tree makes; the review is where a wrong
# one is caught.
#
# bash rather than sh: the walk reads separated records out of awk, and the array
# and here-string forms below are bash.

set -euo pipefail

root=${1:-.}
cd "$root" || { echo "check-excluded-from-the-default-run: cannot enter $root" >&2; exit 2; }

register=.github/scripts/excluded-from-the-default-run.txt
harness=.github/scripts/needs-an-environment.txt

[ -f "$register" ] || {
  echo "check-excluded-from-the-default-run: no register at $root/$register" >&2
  exit 2
}

[ -f "$harness" ] || {
  echo "check-excluded-from-the-default-run: no harness register at $root/$harness" >&2
  echo "check-excluded-from-the-default-run: \`Runs-in\` cannot be judged against a register that is not there" >&2
  exit 2
}

refusals=0

refuse() {
  # refuse <property> <subject> <detail>
  echo "REFUSED $1: $2 - $3"
  refusals=$((refusals + 1))
}

# The register as one line per record, with the record's first line number in
# front so a refusal can name where to go.
#
# The fields are separated by a unit separator rather than by a tab for the
# reason `.github/scripts/check-needs-an-environment.sh` gives at more length: a
# tab is an IFS whitespace character, so `read` collapses a run of them and drops
# the empty field between, and a record with an empty `Because` would arrive with
# its `Runs-in` shifted one field left. The checker would then report a record as
# running nowhere when what it has no reason.
table=$(awk '
  function emit() {
    if (started) printf "%s\037%s\037%s\037%s\037%s\n", line, test, in_file, because, runs_in
    started = 0; line = 0; test = ""; in_file = ""; because = ""; runs_in = ""
  }
  /^#/ { next }
  /^[[:space:]]*$/ { emit(); next }
  {
    if (!started) { started = 1; line = NR }
    value = $0
  }
  # The name, then the colon, then at most one space. A field written with its
  # colon and nothing after it reaches the legs below as an empty value rather
  # than as a line no field name starts, because present-and-empty is exactly
  # what two of those legs are about.
  /^Test:/    { sub(/^Test:[ ]?/, "", value);    test = value;    next }
  /^In:/      { sub(/^In:[ ]?/, "", value);      in_file = value; next }
  /^Because:/ { sub(/^Because:[ ]?/, "", value); because = value; next }
  /^Runs-in:/ { sub(/^Runs-in:[ ]?/, "", value); runs_in = value; next }
  { printf "UNREADABLE\t%s\t%s\n", NR, $0 }
  END { emit() }
' "$register")

if printf '%s\n' "$table" | grep -q '^UNREADABLE'; then
  echo "check-excluded-from-the-default-run: $register holds a line no field name starts:" >&2
  printf '%s\n' "$table" | sed -n 's/^UNREADABLE\t/  line /p' >&2
  echo "check-excluded-from-the-default-run: refusing to judge a register it could not read" >&2
  exit 2
fi

# Every id the harness register declares, one per line. A `Runs-in` is judged
# against this and against nothing else.
harness_ids=$(sed -n 's/^Id:[ ]\{0,1\}//p' "$harness")

# Every `#[ignore]`d function in the tree, as `<path><TAB><name>`, in a stable
# order.
#
# The attribute may be followed by more attributes and by blank lines before the
# function it is about, and `#[ignore = "..."]` and a bare `#[ignore]` are the
# same exclusion, so both forms are read. Build output is derived from the
# sources beside it and is not tracked, so scanning it would report the same
# function twice or one nobody wrote.
ignored_tests() {
  find . -name target -prune -o -name '*.rs' -print 2>/dev/null | LC_ALL=C sort | while IFS= read -r file; do
    awk -v file="${file#./}" '
      /^[[:space:]]*#\[ignore[]( =]/ { pending = 1; next }
      pending && /^[[:space:]]*#\[/ { next }
      pending && /^[[:space:]]*(\/\/|$)/ { next }
      pending && /^[[:space:]]*(pub[[:space:]]+)?(async[[:space:]]+)?fn[[:space:]]+[A-Za-z0-9_]+/ {
        name = $0
        sub(/^[[:space:]]*/, "", name)
        sub(/^pub[[:space:]]+/, "", name)
        sub(/^async[[:space:]]+/, "", name)
        sub(/^fn[[:space:]]+/, "", name)
        sub(/[^A-Za-z0-9_].*$/, "", name)
        printf "%s\t%s\n", file, name
        pending = 0
        next
      }
      pending { pending = 0 }
    ' "$file"
  done
}

in_tree=$(ignored_tests)

records=0
recorded_names=""

# What the default run does not run, printed before any verdict, so a run that
# read one record cannot be read as one that read the set. This is the command
# the set is printed by.
echo "excluded from \`cargo test --locked --workspace\`:"

while IFS=$'\037' read -r line test in_file because runs_in; do
  [ -n "$line" ] || continue
  records=$((records + 1))

  if [ -z "$test" ]; then
    echo "check-excluded-from-the-default-run: the record at $register:$line names no Test" >&2
    echo "check-excluded-from-the-default-run: a record naming no test is not a record" >&2
    exit 2
  fi

  if [ -z "$in_file" ]; then
    echo "check-excluded-from-the-default-run: \`$test\` at $register:$line names no In" >&2
    echo "check-excluded-from-the-default-run: a record that does not say where the test is cannot be checked against it" >&2
    exit 2
  fi

  echo "  $test  in $in_file  run by ${runs_in:-<nothing>}"

  case " $recorded_names " in
    *" $test "*)
      refuse duplicate-exclusion "$register:$line" \
        "\`$test\` is recorded twice, and a second record says nothing the first did not while being able to disagree with it"
      ;;
    *) recorded_names="$recorded_names $test" ;;
  esac

  [ -n "$because" ] || refuse exclusion-without-a-reason "$register:$line" \
    "\`$test\` does not say why it is out of the default run"

  if [ -z "$runs_in" ]; then
    refuse exclusion-runs-nowhere "$register:$line" \
      "\`$test\` names no harness entry, so nothing in this repository runs it"
  elif ! printf '%s\n' "$harness_ids" | grep -qxF "$runs_in"; then
    refuse exclusion-runs-nowhere "$register:$line" \
      "\`$test\` says it is run by \`$runs_in\`, which $harness declares no entry for, so it is out of the default run and out of the harness both"
  fi

  if ! printf '%s\n' "$in_tree" | grep -qxF "$in_file$(printf '\t')$test"; then
    refuse record-names-no-test "$register:$line" \
      "$in_file holds no \`#[ignore]\`d function called \`$test\`, so this record describes a source that has moved under it"
  fi
done <<< "$table"

[ "$records" -ne 0 ] || echo "  nothing"

# The other direction. A test that left the default run and never got a record
# is the failure the register cannot catch by reading itself.
while IFS=$'\t' read -r file name; do
  [ -n "$name" ] || continue
  case " $recorded_names " in
    *" $name "*) ;;
    *)
      refuse test-excluded-without-a-record "$file" \
        "\`$name\` carries \`#[ignore]\`, so the default run does not run it, and $register names no record for it"
      ;;
  esac
done <<< "$in_tree"

# Say what was examined on both sides, so a run that read an empty tree cannot be
# read as one that read the tree and found nothing wrong.
tree_count=$(printf '%s' "$in_tree" | grep -c . || true)
echo "read $records record(s) from $register against $tree_count \`#[ignore]\`d function(s) in the tree"

if [ "$refusals" -ne 0 ]; then
  echo "check-excluded-from-the-default-run: $refusals refusal(s). docs/excluded-from-the-default-run.md says what each one is for."
  exit 1
fi

echo "check-excluded-from-the-default-run: every excluded test says why it is out and names an entry that runs it"
