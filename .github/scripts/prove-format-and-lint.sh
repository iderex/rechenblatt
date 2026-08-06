#!/usr/bin/env bash
# Prove that the formatting gate and the lint gate bite, and that the declaration
# in the tree is what makes the lint gate bite rather than a tool default.
#
# Usage:
#   .github/scripts/prove-format-and-lint.sh
#
# Each property gets a leg that plants exactly the mistake it is about and
# requires a red run naming the file, and a neighbouring leg that changes the one
# thing back and requires a green run. A gate that refuses everything fails the
# second kind; one that refuses nothing fails the first.
#
# The third kind of leg is the one worth arguing for. It plants the same mistake
# and takes the declaration out of Cargo.toml, and requires the run to go green.
# Without it, a leg that passes proves only that some default somewhere refuses
# the mistake, which is not what the tracked line claims. That is how the clippy
# declaration this repository nearly shipped was found to be doing nothing: with
# `warnings` denied, a second line denying `clippy::all` moved no run in either
# direction.
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

# tree <name>: HEAD, unpacked. The copy carries this repository's own
# rustfmt.toml and its own lint declarations, because those are the thing under
# test. A fixture writing its own would prove the fixture.
tree() {
  local name=$1
  local dir="$work/$name"
  mkdir -p "$dir"
  git -C "$repo" archive --format=tar HEAD | tar -xf - -C "$dir"
  printf '%s' "$dir"
}

# plant <dir> <body>: a public module whose name appears in any diagnostic about
# it, so a leg can require the output to name the file it is about. Public
# because a private one nothing uses is itself a warning, which would make every
# leg red for a reason no leg is about.
plant() {
  local dir=$1 body=$2
  printf '%s' "$body" > "$dir/crates/calc/src/probe.rs"
  printf '\npub mod probe;\n' >> "$dir/crates/calc/src/lib.rs"
}

# undeclare <dir> <line-prefix>: remove the tracked declaration whose line starts
# with the given text, so a leg can ask what the declaration was doing.
undeclare() {
  local dir=$1 prefix=$2
  grep -q "^$prefix" "$dir/Cargo.toml" || {
    echo "FAIL  Cargo.toml carries no line starting '$prefix'; this proof is out of date" >&2
    failures=$((failures + 1))
    return
  }
  grep -v "^$prefix" "$dir/Cargo.toml" > "$dir/Cargo.toml.next"
  mv "$dir/Cargo.toml.next" "$dir/Cargo.toml"
}

# expect <dir> <label> <format|lint> <red <substring>...|green>
expect() {
  local dir=$1 label=$2 gate=$3 want=$4
  shift 4
  local out status
  cases=$((cases + 1))
  set +e
  case "$gate" in
    format) out=$(cd "$dir" && cargo fmt --all -- --check 2>&1) ;;
    lint) out=$(cd "$dir" && cargo clippy --locked --offline --workspace --all-targets 2>&1) ;;
    *) echo "unknown gate $gate" >&2; exit 2 ;;
  esac
  status=$?
  set -e

  if [ "$want" = "green" ]; then
    if [ "$status" -eq 0 ]; then
      echo "ok    $label passes the $gate gate"
      return
    fi
    echo "FAIL  $label should pass the $gate gate, got status $status"
    printf '%s\n' "$out" | sed 's/^/      /'
    failures=$((failures + 1))
    return
  fi

  local missing=""
  local needle
  for needle in "$@"; do
    printf '%s\n' "$out" | grep -qF -- "$needle" || missing="$missing $needle"
  done
  if [ "$status" -ne 0 ] && [ -z "$missing" ]; then
    echo "ok    $label fails the $gate gate, naming$(printf ' %s' "$@")"
    return
  fi
  if [ "$status" -eq 0 ]; then
    echo "FAIL  $label should fail the $gate gate and it passed"
  else
    echo "FAIL  $label fails the $gate gate without naming:$missing"
  fi
  printf '%s\n' "$out" | sed 's/^/      /'
  failures=$((failures + 1))
}

misformatted='pub fn probe() -> i32 {
        1
}
'

formatted='pub fn probe() -> i32 {
    1
}
'

# A lint clippy carries and the compiler does not, so the leg says something
# about clippy specifically.
clippy_mistake='pub fn probe() -> i32 {
    return 1;
}
'

# A lint the compiler carries on its own, so the same declaration is shown to
# reach both tools.
compiler_mistake='pub fn probe() -> i32 {
    let unread = 2;
    1
}
'

# --- the formatting gate ------------------------------------------------------
d=$(tree fmt-bad); plant "$d" "$misformatted"
expect "$d" "an over-indented body" format red "probe.rs"

d=$(tree fmt-good); plant "$d" "$formatted"
expect "$d" "the same body at the right indent" format green

# rustfmt.toml is asked the same question the lint declaration is asked below.
# Its one setting is about line endings rather than about this indent, so taking
# it away must NOT rescue the misformatted file: this leg reads red, and a green
# one would mean the setting was doing the refusing.
d=$(tree fmt-undeclared); plant "$d" "$misformatted"; rm -f "$d/rustfmt.toml"
expect "$d" "the same body with rustfmt.toml gone" format red "probe.rs"

# --- the lint gate, on a lint only clippy has ---------------------------------
d=$(tree clippy-bad); plant "$d" "$clippy_mistake"
expect "$d" "an unneeded return" lint red "probe.rs" "needless_return"

d=$(tree clippy-good); plant "$d" "$formatted"
expect "$d" "the same function without it" lint green

d=$(tree clippy-undeclared); plant "$d" "$clippy_mistake"; undeclare "$d" "warnings = "
expect "$d" "an unneeded return with the denial removed from Cargo.toml" lint green

# --- the lint gate, on a lint the compiler has --------------------------------
d=$(tree rustc-bad); plant "$d" "$compiler_mistake"
expect "$d" "a variable nothing reads" lint red "probe.rs" "unused_variables"

d=$(tree rustc-good); plant "$d" "$formatted"
expect "$d" "the same function without it" lint green

d=$(tree rustc-undeclared); plant "$d" "$compiler_mistake"; undeclare "$d" "warnings = "
expect "$d" "a variable nothing reads with the denial removed from Cargo.toml" lint green

echo
echo "$cases case(s), $failures failure(s)"
[ "$failures" -eq 0 ] || exit 1
