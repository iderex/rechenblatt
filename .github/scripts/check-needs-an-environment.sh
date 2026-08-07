#!/usr/bin/env bash
# Refuse an entry in the impure register that the next reader could not act on.
#
# Usage:
#   .github/scripts/check-needs-an-environment.sh [repository-root]
#
# This runs in the pure suite's own gate, on every machine, and it reads the
# register rather than running anything out of it. That split is the point: the
# entries need an environment, and the rule about the entries does not.
#
# Four refusals:
#
#   entry-without-a-need     an entry with no `Needs`, so nobody can reproduce
#                            the environment it claims a result from
#   entry-without-a-reason   an entry with no `Because`, so nothing says why it
#                            is out of the pure suite and the register turns
#                            into the place awkward tests are put
#   entry-names-no-script    an entry whose `Run` names a script that is not in
#                            the tree, which is an entry that stopped being
#                            runnable without anything going red
#   duplicate-entry-id       two entries under one id, where the runner would
#                            silently take the first and the second would never
#                            run again
#
# Exit 0 means every entry passed all four. Exit 1 means at least one refusal.
# Exit 2 means the check could not run, which is a failure and never a pass, and
# covers a missing register, a register holding a line no field name starts, a
# register with no entries in it at all, and a record with no `Id` or no `Run` -
# an entry the runner can neither name nor execute is not an entry, and judging
# the rest of the file while one of those sits in it would report a clean
# register that is not one.
#
# WHAT THIS CANNOT JUDGE, and it is the larger half. Whether an entry is here
# BECAUSE it genuinely cannot be pure is a judgement about the thing the entry
# runs, and no reading of this file makes it: a `Because` line saying something
# false passes exactly like a true one. What is checkable is that somebody wrote
# the sentence down where a reviewer meets it, and the review is where a wrong
# one is caught. docs/needs-an-environment.md says the same thing at more length.
#
# bash rather than sh: the walk reads tab-separated records out of awk, and the
# array and here-string forms below are bash.

set -euo pipefail

root=${1:-.}
cd "$root" || { echo "check-needs-an-environment: cannot enter $root" >&2; exit 2; }

register=.github/scripts/needs-an-environment.txt
[ -f "$register" ] || {
  echo "check-needs-an-environment: no register at $root/$register" >&2
  exit 2
}

refusals=0
entries=0

refuse() {
  # refuse <property> <subject> <detail>
  echo "REFUSED $1: $2 - $3"
  refusals=$((refusals + 1))
}

# The register as one line per record, with the record's first line number in
# front so a refusal can name where to go. A field that is absent arrives empty,
# which is what the four legs below are about.
#
# The fields are separated by a unit separator rather than by a tab, and that is
# not decoration. A tab is an IFS WHITESPACE character, so `read` collapses a run
# of them into one delimiter and drops the empty field between: a record with an
# empty `Because` would arrive with its `Run` shifted one field left, and the
# checker would report the entry as having no command when what it has no
# reason. That is the near-miss this separator is chosen against, and it was
# found by a leg below failing for the wrong cause.
table=$(awk '
  function emit() {
    if (started) printf "%s\037%s\037%s\037%s\037%s\n", line, id, needs, because, run
    started = 0; line = 0; id = ""; needs = ""; because = ""; run = ""
  }
  /^#/ { next }
  /^[[:space:]]*$/ { emit(); next }
  {
    if (!started) { started = 1; line = NR }
    value = $0
  }
  # The name, then the colon, then at most one space. A field written with its
  # colon and nothing after it has to reach the legs below as an empty value
  # rather than as a line no field name starts, because present-and-empty is
  # exactly what two of those legs are about.
  /^Id:/      { sub(/^Id:[ ]?/, "", value);      id = value;      next }
  /^Needs:/   { sub(/^Needs:[ ]?/, "", value);   needs = value;   next }
  /^Because:/ { sub(/^Because:[ ]?/, "", value); because = value; next }
  /^Run:/     { sub(/^Run:[ ]?/, "", value);     run = value;     next }
  { printf "UNREADABLE\t%s\t%s\n", NR, $0 }
  END { emit() }
' "$register")

if printf '%s\n' "$table" | grep -q '^UNREADABLE'; then
  echo "check-needs-an-environment: $register holds a line no field name starts:" >&2
  printf '%s\n' "$table" | sed -n 's/^UNREADABLE\t/  line /p' >&2
  echo "check-needs-an-environment: refusing to judge a register it could not read" >&2
  exit 2
fi

[ -n "$table" ] || {
  echo "check-needs-an-environment: read no entries out of $register; refusing to report a clean register" >&2
  exit 2
}

seen=""

while IFS=$'\037' read -r line id needs because run; do
  [ -n "$line" ] || continue
  entries=$((entries + 1))

  if [ -z "$id" ]; then
    echo "check-needs-an-environment: the record at $register:$line declares no Id" >&2
    echo "check-needs-an-environment: an entry the runner cannot name is not an entry" >&2
    exit 2
  fi

  if [ -z "$run" ]; then
    echo "check-needs-an-environment: \`$id\` at $register:$line declares no Run" >&2
    echo "check-needs-an-environment: an entry the runner cannot execute is not an entry" >&2
    exit 2
  fi

  case " $seen " in
    *" $id "*)
      refuse duplicate-entry-id "$register:$line" \
        "\`$id\` is declared twice, and the runner takes the first, so the second never runs again"
      ;;
    *) seen="$seen $id" ;;
  esac

  [ -n "$needs" ] || refuse entry-without-a-need "$register:$line" \
    "\`$id\` says nothing about the environment it needs, so a result from it cannot be reproduced"

  [ -n "$because" ] || refuse entry-without-a-reason "$register:$line" \
    "\`$id\` does not say why it cannot be proved in the pure suite"

  # Every path-shaped token in the command that looks like something in this
  # tree. A command is words; the ones with a directory separator are the ones
  # this repository is the authority for, and a bare word is a program name that
  # comes from the environment the entry declares rather than from the tree.
  for word in $run; do
    case "$word" in
      */*)
        [ -e "$word" ] || refuse entry-names-no-script "$register:$line" \
          "\`$id\` runs \`$word\`, which is not in the tree"
        ;;
    esac
  done
done <<< "$table"

# Say what was examined, so a run that read one entry cannot be read as one that
# read the register and found nothing wrong.
echo "read $entries entr(y|ies) from $register"

if [ "$refusals" -ne 0 ]; then
  echo "check-needs-an-environment: $refusals refusal(s). docs/needs-an-environment.md says what each one is for."
  exit 1
fi

echo "check-needs-an-environment: every entry names what it needs, says why, and runs something that is there"
