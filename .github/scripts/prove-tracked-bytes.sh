#!/usr/bin/env bash
# Prove that check-tracked-bytes.sh bites, once per property it names.
#
# Usage:
#   .github/scripts/prove-tracked-bytes.sh
#
# Each property gets two legs. The first builds a repository holding exactly the
# byte the property is about and requires the checker to refuse THAT property and
# no other, so a checker refusing everything cannot pass this. The second changes
# the one byte back and requires the checker to refuse nothing, so a checker
# refusing nothing cannot pass either.
#
# The declarations these legs are judged against are written here rather than
# copied from the repository's own .gitattributes. A proof that reads the real
# declarations proves the state of the tree on the day it ran; this proves the
# checker.
#
# The bytes are written with printf escapes rather than committed as files. A
# fixture holding a carriage return is exactly what the checker refuses, so a
# tracked one would either red this repository or need the exception it is
# supposed to test.

set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
checker="$here/check-tracked-bytes.sh"
[ -x "$checker" ] || [ -f "$checker" ] || { echo "no checker at $checker" >&2; exit 2; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

cases=0
failures=0

# make_repo <name> <attributes-file-content>; the payload files come after.
make_repo() {
  local name=$1 attributes=$2
  local dir="$work/$name"
  mkdir -p "$dir"
  git -C "$dir" init --quiet
  printf '%s' "$attributes" > "$dir/.gitattributes"
  printf '%s' "$dir"
}

add() {
  # add <dir> <path>; content on stdin, added to the index exactly as given.
  local dir=$1 path=$2
  cat > "$dir/$path"
  git -C "$dir" add -- "$path"
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

text_only='* text=auto eol=lf
*.md text eol=lf
'

binary_png='* text=auto eol=lf
*.png binary
'

# --- cr-in-tracked-text -------------------------------------------------------
# A lone carriage return, not CRLF. git's own eol conversion rewrites CRLF into LF
# on the way into the index and leaves a lone CR alone, so this is the byte the
# attribute cannot reach and the checker has to.
d=$(make_repo cr-bad "$text_only")
printf 'first\rsecond\n' | add "$d" a.md
expect "$d" "a lone carriage return" cr-in-tracked-text

d=$(make_repo cr-good "$text_only")
printf 'first\nsecond\n' | add "$d" a.md
expect "$d" "the same file without it" clean

d=$(make_repo cr-allowed "$text_only"'a.md allow-cr
')
printf 'first\rsecond\n' | add "$d" a.md
expect "$d" "a lone carriage return under a declared exception" clean

# --- bom-in-tracked-text ------------------------------------------------------
d=$(make_repo bom-bad "$text_only")
printf '\xef\xbb\xbf# title\n' | add "$d" b.md
expect "$d" "a UTF-8 byte order mark" bom-in-tracked-text

d=$(make_repo bom-good "$text_only")
printf '# title\n' | add "$d" b.md
expect "$d" "the same file without it" clean

d=$(make_repo bom-allowed "$text_only"'b.md allow-bom
')
printf '\xef\xbb\xbf# title\n' | add "$d" b.md
expect "$d" "a byte order mark under a declared exception" clean

# --- text-is-not-utf8 ---------------------------------------------------------
# 0xE4 0xF6 is the same two letters in latin-1 and is not valid UTF-8.
d=$(make_repo utf8-bad "$text_only")
printf 'gr\xe4\xdfer\n' | add "$d" c.md
expect "$d" "a latin-1 byte sequence" text-is-not-utf8

d=$(make_repo utf8-good "$text_only")
printf 'gr\xc3\xa4\xc3\x9fer\n' | add "$d" c.md
expect "$d" "the same letters as UTF-8" clean

# --- declared-text-holds-nul --------------------------------------------------
d=$(make_repo nul-bad "$text_only")
printf 'before\x00after\n' | add "$d" d.md
expect "$d" "a NUL byte in a file declared text" declared-text-holds-nul

d=$(make_repo nul-good "$text_only")
printf 'beforeafter\n' | add "$d" d.md
expect "$d" "the same file without it" clean

# --- declared-binary-is-text --------------------------------------------------
d=$(make_repo binary-bad "$binary_png")
printf 'this is not a png\n' | add "$d" e.png
expect "$d" "readable text under a binary declaration" declared-binary-is-text

d=$(make_repo binary-good "$binary_png")
printf '\x89PNG\r\n\x1a\n\x00\x00\x00\x0d' | add "$d" e.png
expect "$d" "an actual binary header" clean

echo
echo "$cases case(s), $failures failure(s)"
[ "$failures" -eq 0 ] || exit 1
