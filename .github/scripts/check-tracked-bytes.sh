#!/usr/bin/env bash
# Refuse tracked bytes that .gitattributes says may not be there.
#
# Usage:
#   .github/scripts/check-tracked-bytes.sh [repository-root]
#
# Exit 0 means every tracked file passed. Exit 1 means at least one refusal, each
# printed with the property it broke and the path it broke it on. Exit 2 means the
# check could not run, which is a failure and never a pass.
#
# The subject is the bytes in git, read with `git cat-file blob :path`, and not
# the bytes in the working tree. A clone with core.autocrlf=true holds CRLF in its
# working tree for files stored with LF, so a check reading the working tree would
# refuse a clean repository on one platform and pass it on another.
#
# The properties, one refusal each:
#
#   cr-in-tracked-text        a carriage return in a file git treats as text
#   bom-in-tracked-text       a UTF-8 byte order mark at the start of such a file
#   text-is-not-utf8          such a file that is not valid UTF-8
#   declared-text-holds-nul   such a file containing a NUL byte
#   declared-binary-is-text   a file declared binary whose content is text
#
# The last two are the two directions of one thing: a tracked file whose declared
# type does not match its content. They are separate properties because the
# repairs differ. The first says declare the file binary; the second says stop
# declaring it binary.
#
# bash rather than sh: the walk reads `git ls-files -z`, and a NUL-delimited read
# is a bash feature. A newline in a path is legal, and a checker that word-splits
# its input is a checker that refuses valid work.

set -euo pipefail

root=${1:-.}
cd "$root" || { echo "check-tracked-bytes: cannot enter $root" >&2; exit 2; }

if ! git rev-parse --git-dir >/dev/null 2>&1; then
  echo "check-tracked-bytes: $root is not a git repository" >&2
  exit 2
fi

if ! command -v iconv >/dev/null 2>&1; then
  echo "check-tracked-bytes: iconv is not installed, so encoding cannot be judged" >&2
  echo "check-tracked-bytes: failing closed rather than reporting a tree it did not read" >&2
  exit 2
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
blob="$tmp/blob"

fail=0
examined=0
exceptions=0

# `git check-attr` prints `<path>: <attribute>: <value>`. A path may itself hold
# `: `, so take everything after the last one rather than splitting on the first.
attr() {
  local line
  line=$(git check-attr "$1" -- "$2")
  printf '%s' "${line##*: }"
}

refuse() {
  echo "REFUSED $1: $2"
  fail=1
}

while IFS= read -r -d '' path; do
  examined=$((examined + 1))

  if ! git cat-file blob ":$path" > "$blob" 2>/dev/null; then
    echo "check-tracked-bytes: cannot read the tracked bytes of $path" >&2
    exit 2
  fi

  size=$(wc -c < "$blob")
  bytes_cr=$(LC_ALL=C tr -dc '\r' < "$blob" | wc -c)
  bytes_nul=$(LC_ALL=C tr -dc '\000' < "$blob" | wc -c)
  first3=$(head -c 3 "$blob" | od -An -tx1 | tr -d ' \n')
  utf8=0
  iconv -f UTF-8 -t UTF-8 < "$blob" >/dev/null 2>&1 || utf8=1

  if [ "$(attr text "$path")" = "unset" ]; then
    # Declared binary. Content holding no NUL and valid as UTF-8 is text wearing a
    # binary declaration, which hides it from every check that reads tracked text.
    # An empty file is evidence of neither and is left alone.
    if [ "$size" -gt 0 ] && [ "$bytes_nul" -eq 0 ] && [ "$utf8" -eq 0 ]; then
      refuse declared-binary-is-text "$path"
    fi
    continue
  fi

  # Declared text, whether by an explicit `text` or by `text=auto`.
  if [ "$bytes_nul" -gt 0 ]; then
    refuse declared-text-holds-nul "$path"
  fi

  if [ "$bytes_cr" -gt 0 ]; then
    if [ "$(attr allow-cr "$path")" = "set" ]; then
      echo "allowed cr-in-tracked-text: $path"
      exceptions=$((exceptions + 1))
    else
      refuse cr-in-tracked-text "$path"
    fi
  fi

  if [ "$first3" = "efbbbf" ]; then
    if [ "$(attr allow-bom "$path")" = "set" ]; then
      echo "allowed bom-in-tracked-text: $path"
      exceptions=$((exceptions + 1))
    else
      refuse bom-in-tracked-text "$path"
    fi
  fi

  # A file holding a NUL is already refused above for that, and reporting it as
  # bad UTF-8 as well would name one file twice for one cause.
  if [ "$utf8" -ne 0 ] && [ "$bytes_nul" -eq 0 ]; then
    refuse text-is-not-utf8 "$path"
  fi
done < <(git ls-files -z)

# Say what was examined, so a run that covered less than the whole tree cannot be
# read as one that covered it and found nothing.
echo "examined $examined tracked path(s); honoured $exceptions declared exception(s)"

if [ "$fail" -ne 0 ]; then
  echo "check-tracked-bytes: docs/tracked-bytes.md says what each property means"
  exit 1
fi

echo "check-tracked-bytes: every tracked path matches what .gitattributes declares"
