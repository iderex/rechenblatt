#!/usr/bin/env bash
# Run one thing that cannot be proved in the pure suite, and record what it ran in.
#
# Usage:
#   .github/scripts/needs-an-environment.sh list [root]
#   .github/scripts/needs-an-environment.sh <id> [root]
#
# The file is named for what it needs rather than for what it does, so that
# somebody scanning the scripts directory meets the requirement before the
# feature. Nothing here is part of `cargo test --locked --workspace`, and nothing
# here runs without being told which entry to run: an invocation with no argument
# prints the register and exits 2. There is no `all`, and that is deliberate. An
# entry needs a different environment from its neighbour, and a runner that took
# them together would report one verdict over two requirements.
#
# `.github/scripts/needs-an-environment.txt` is the register and this file holds
# no entry of its own. `docs/needs-an-environment.md` argues why.
#
# Exit 0 means the entry's command succeeded in the environment recorded above
# the result. Exit 1 means it failed there. Exit 2 means it did not run, which
# covers an unknown id, an unreadable register, and a missing argument, and is a
# failure and never a pass.
#
# What a result is, and what it is not. Every line this file prints begins with
# `needs-an-environment:`, and the block around a run carries the entry, what it
# needed, why it is here, the commit and the host. That marker is the whole point
# of it: a number out of this runner carries its environment or it is not a
# number anybody can use, and it may never be quoted as a result of the pure
# suite, which ran somewhere else under different rules.

set -euo pipefail

mode=${1:-}
root=${2:-}

say() {
  echo "needs-an-environment: $1"
}

if [ -z "$root" ]; then
  root=$(git rev-parse --show-toplevel 2>/dev/null || echo .)
fi
cd "$root" || { say "cannot enter $root" >&2; exit 2; }

register=.github/scripts/needs-an-environment.txt
[ -f "$register" ] || { say "no register at $root/$register" >&2; exit 2; }

# The register, flattened to one line per entry. The loader is deliberately
# dumber than the checker beside it: this file has to be able to run an entry out
# of a register the checker would refuse, or a red checker would take the harness
# down with it.
#
# A unit separator between the fields rather than a tab. A tab is an IFS
# WHITESPACE character, so `read` collapses a run of them and drops the empty
# field between, and an entry with an empty `Because` would arrive here with its
# command shifted out of `run` - which this file would then eval as the empty
# string and report as a pass. The checker beside this one carries the same note
# for the same reason.
entries() {
  awk '
    function emit() {
      if (id != "") printf "%s\037%s\037%s\037%s\n", id, needs, because, run
      id = ""; needs = ""; because = ""; run = ""
    }
    /^#/ { next }
    /^[[:space:]]*$/ { emit(); next }
    { value = $0 }
    /^Id:/      { sub(/^Id:[ ]?/, "", value);      id = value;      next }
    /^Needs:/   { sub(/^Needs:[ ]?/, "", value);   needs = value;   next }
    /^Because:/ { sub(/^Because:[ ]?/, "", value); because = value; next }
    /^Run:/     { sub(/^Run:[ ]?/, "", value);     run = value;     next }
    { printf "UNREADABLE\t%s\n", $0 }
    END { emit() }
  ' "$register"
}

table=$(entries)

if printf '%s\n' "$table" | grep -q '^UNREADABLE'; then
  say "$register holds a line no field name starts:" >&2
  printf '%s\n' "$table" | sed -n 's/^UNREADABLE\t/  /p' >&2
  exit 2
fi

[ -n "$table" ] || { say "read no entries out of $register" >&2; exit 2; }

announce() {
  # announce <id> <needs> <because> <run>
  say "$1"
  say "  needs   $2"
  say "  because $3"
  say "  run     $4"
}

if [ "$mode" = "list" ] || [ -z "$mode" ]; then
  say "nothing here is part of the default suite, and none of it runs without being named"
  while IFS=$'\037' read -r id needs because run; do
    [ -n "$id" ] || continue
    announce "$id" "$needs" "$because" "$run"
  done <<< "$table"
  if [ -z "$mode" ]; then
    say "name one of the ids above to run it. Nothing was run."
    exit 2
  fi
  exit 0
fi

found=""
while IFS=$'\037' read -r id needs because run; do
  [ "$id" = "$mode" ] || continue
  found=$id
  break
done <<< "$table"

if [ -z "$found" ]; then
  say "$register declares no entry called \`$mode\`. Nothing was run." >&2
  say "the ids it does declare:" >&2
  printf '%s\n' "$table" | cut -d$'\037' -f1 | sed 's/^/  /' >&2
  exit 2
fi

# The record. It is printed before the command rather than after it, so a run
# that is killed halfway still leaves the environment it was killed in.
say "----- result record begins -----"
announce "$id" "$needs" "$because" "$run"
say "  commit  $(git rev-parse HEAD 2>/dev/null || echo '<not a git checkout>')"
say "  host    $(uname -srm 2>/dev/null || echo '<unknown>')"
say "  started $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
say "this is not a result of the default suite and may not be quoted as one"

status=0
eval "$run" || status=$?

if [ "$status" -eq 0 ]; then
  say "$id PASSED in the environment recorded above"
else
  say "$id FAILED in the environment recorded above, exit $status"
fi
say "----- result record ends -----"

exit "$status"
