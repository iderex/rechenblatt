#!/usr/bin/env bash
# Prove that check-invariants.sh bites, once per invariant and once per refusal
# the register itself carries.
#
# Usage:
#   .github/scripts/prove-invariants.sh
#
# Each property gets two legs. The first builds a repository holding exactly the
# thing the property is about and requires the checker to refuse THAT id and no
# other, so a checker refusing everything cannot pass this. The second changes
# the one thing back and requires the checker to refuse nothing, so a checker
# refusing nothing cannot pass either.
#
# The violations are written here rather than committed as fixtures. A tracked
# file holding a host font path is exactly what this gate refuses, so a fixture
# of one would either red this repository or need the exception it exists to
# test. That is the same reason the tracked-bytes proof writes its bytes.
#
# The invariant list each leg is judged against is written here as well, and not
# read from `.github/scripts/invariants.txt`. A proof that reads the real list
# proves the state of the tree on the day it ran; this proves the checker. The
# patterns below are copies for that reason, and the one thing they are not is
# the authority for what this repository enforces.

set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
checker="$here/check-invariants.sh"
[ -f "$checker" ] || { echo "no checker at $checker" >&2; exit 2; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

cases=0
failures=0

# make_repo <name> <list-content>
make_repo() {
  local name=$1 list=$2
  local dir="$work/$name"
  mkdir -p "$dir/.github/scripts"
  git -C "$dir" init --quiet
  # The bytes the legs write are the subject, so nothing rewrites them on the way
  # into this scratch index whatever the surrounding clone has configured.
  git -C "$dir" config core.autocrlf false
  printf '%s' "$list" > "$dir/.github/scripts/invariants.txt"
  git -C "$dir" add -- .github/scripts/invariants.txt
  printf '%s' "$dir"
}

add() {
  # add <dir> <path>; content on stdin, added to the index exactly as given.
  local dir=$1 path=$2
  mkdir -p "$dir/$(dirname "$path")"
  cat > "$dir/$path"
  git -C "$dir" add -- "$path"
}

# The three shapes a leg can expect, each comparing the WHOLE set of refusal
# lines rather than looking for one. A checker that refuses everything fails the
# clean legs, and one that refuses everything under one id fails the exactness
# below, so neither can pass this file by being loud.
#
# A violated invariant prints its id inside the detail, because the property is
# `invariant-violated` for every one of them and the id is what a reader acts on.
# The register's own refusals print their property in the usual place. Hence two
# matchers rather than one.

# expect_clean <dir> <label>
expect_clean() {
  local dir=$1 label=$2 out status refusals
  cases=$((cases + 1))
  set +e
  out=$(bash "$checker" "$dir" 2>&1)
  status=$?
  set -e
  refusals=$(printf '%s\n' "$out" | grep '^REFUSED ' || true)
  if [ "$status" -eq 0 ] && [ -z "$refusals" ]; then
    echo "ok    $label refuses nothing"
    return
  fi
  echo "FAIL  $label should refuse nothing, got status $status"
  printf '%s\n' "$out" | sed 's/^/      /'
  failures=$((failures + 1))
}

# expect_cannot_run <dir> <label>
expect_cannot_run() {
  local dir=$1 label=$2 out status
  cases=$((cases + 1))
  set +e
  out=$(bash "$checker" "$dir" 2>&1)
  status=$?
  set -e
  if [ "$status" -eq 2 ]; then
    echo "ok    $label stops the run rather than reporting a tree it did not read"
    return
  fi
  echo "FAIL  $label should exit 2, got status $status"
  printf '%s\n' "$out" | sed 's/^/      /'
  failures=$((failures + 1))
}

# expect_only <dir> <label> <grep-expression matching the wanted refusal lines>
expect_only() {
  local dir=$1 label=$2 want=$3 wanted_desc=$4
  local out status refusals wanted other
  cases=$((cases + 1))
  set +e
  out=$(bash "$checker" "$dir" 2>&1)
  status=$?
  set -e
  refusals=$(printf '%s\n' "$out" | grep '^REFUSED ' || true)
  wanted=$(printf '%s\n' "$refusals" | grep -cE "$want" || true)
  other=$(printf '%s\n' "$refusals" | grep '^REFUSED ' | grep -vE "$want" || true)
  if [ "$status" -eq 1 ] && [ "$wanted" -ge 1 ] && [ -z "$other" ]; then
    echo "ok    $label refuses exactly $wanted_desc"
    return
  fi
  echo "FAIL  $label should refuse exactly $wanted_desc, got status $status"
  printf '%s\n' "$out" | sed 's/^/      /'
  failures=$((failures + 1))
}

# expect_invariant <dir> <label> <id>
expect_invariant() {
  expect_only "$1" "$2" "^REFUSED invariant-violated: .* - $3 - " "$3"
}

# expect_register <dir> <label> <property>
expect_register() {
  expect_only "$1" "$2" "^REFUSED $3: " "$3"
}

fonts='Id: host-font-path
Files: *.rs
Prevents: a rendering that depends on what is installed on the machine.
Pattern: /usr/share/fonts|[Ww]indows\\+[Ff]onts

'

network='Id: network-outside-host
Files: *.rs
Prevents: a parsing component reaching the network.
Pattern: std::net|TcpStream
Except: crates/host/ the one component that may hold a capability

'

secret='Id: secret-in-a-plain-string
Files: *.rs
Prevents: a credential in a type whose Debug prints it.
Pattern: (password|secret|[a-z_]+_token)[a-z_]*[[:space:]]*:[[:space:]]*(String|&str)

'

absolute='Id: absolute-path-in-a-test
Files: */tests/*.rs
Prevents: a test that passes only on the machine it was written on.
Pattern: "/[A-Za-z]

'

# --- host-font-path -----------------------------------------------------------
d=$(make_repo font-bad "$fonts")
printf 'const DIR: &str = "/usr/share/fonts";\n' | add "$d" crates/render/src/lib.rs
expect_invariant "$d" "a source naming the host font directory" host-font-path

d=$(make_repo font-good "$fonts")
printf 'const DIR: &str = "tests/fixtures/fonts";\n' | add "$d" crates/render/src/lib.rs
expect_clean "$d" "the same source pointed at a fixture directory"

d=$(make_repo font-windows "$fonts")
printf 'const DIR: &str = "C:\\\\Windows\\\\Fonts";\n' | add "$d" crates/render/src/lib.rs
expect_invariant "$d" "the same directory spelled for the other platform" host-font-path

# --- network-outside-host -----------------------------------------------------
# Every leg here tracks a file under the excepted prefix, because the register
# refuses an exception naming nothing and would otherwise report that instead of
# the thing the leg is about. The exception legs below are where that refusal is
# proved on purpose.
d=$(make_repo net-bad "$network")
printf 'pub const COMPONENT: &str = "host";\n' | add "$d" crates/host/src/lib.rs
printf 'use std::net::TcpStream;\n' | add "$d" crates/model/src/lib.rs
expect_invariant "$d" "a parsing component opening a socket" network-outside-host

d=$(make_repo net-good "$network")
printf 'pub const COMPONENT: &str = "host";\n' | add "$d" crates/host/src/lib.rs
printf 'pub const COMPONENT: &str = "model";\n' | add "$d" crates/model/src/lib.rs
expect_clean "$d" "the same component without it"

d=$(make_repo net-excepted "$network")
printf 'use std::net::TcpStream;\n' | add "$d" crates/host/src/lib.rs
expect_clean "$d" "the identical line in the declared place"

# --- secret-in-a-plain-string -------------------------------------------------
d=$(make_repo secret-bad "$secret")
printf 'struct Config {\n    password: String,\n}\n' | add "$d" crates/host/src/lib.rs
expect_invariant "$d" "a credential held as a plain string" secret-in-a-plain-string

d=$(make_repo secret-good "$secret")
printf 'struct Config {\n    password: Secret,\n}\n' | add "$d" crates/host/src/lib.rs
expect_clean "$d" "the same field in a type that does not print itself"

# The near-miss this pattern was narrowed for. `token` on its own matched a
# parameter holding a backticked token in this repository's own documentation
# check, which is a rule people silence rather than obey, so the pattern requires
# a qualifier and this leg is what holds it to that.
d=$(make_repo secret-near-miss "$secret")
printf 'fn judged(token: &str) -> bool {\n    !token.is_empty()\n}\n' | add "$d" crates/cli/src/main.rs
expect_clean "$d" "an unrelated parameter called token"

# --- absolute-path-in-a-test --------------------------------------------------
d=$(make_repo absolute-bad "$absolute")
printf 'fn t() { let p = "/tmp/rechenblatt"; let _ = p; }\n' | add "$d" crates/cli/tests/a.rs
expect_invariant "$d" "a test naming an absolute path" absolute-path-in-a-test

d=$(make_repo absolute-good "$absolute")
printf 'fn t() { let p = std::env::temp_dir(); let _ = p; }\n' | add "$d" crates/cli/tests/a.rs
expect_clean "$d" "the same test asking for a directory it created"

# The Files field is doing work rather than decorating the record: the identical
# literal outside a test directory is not this invariant's subject.
d=$(make_repo absolute-not-a-test "$absolute")
printf 'fn t() { let p = "/tmp/rechenblatt"; let _ = p; }\n' | add "$d" crates/cli/src/main.rs
expect_clean "$d" "the identical literal outside a test file"

# --- the exception register ---------------------------------------------------
d=$(make_repo except-no-reason 'Id: network-outside-host
Files: *.rs
Prevents: a parsing component reaching the network.
Pattern: TcpStream
Except: crates/host/

')
printf 'pub const COMPONENT: &str = "host";\n' | add "$d" crates/host/src/lib.rs
expect_register "$d" "an exception that does not say why" exception-without-a-reason

d=$(make_repo except-dangling 'Id: network-outside-host
Files: *.rs
Prevents: a parsing component reaching the network.
Pattern: TcpStream
Except: crates/nowhere/ the component this exception was written for

')
printf 'pub const COMPONENT: &str = "host";\n' | add "$d" crates/host/src/lib.rs
expect_register "$d" "an exception whose path left the tree" exception-names-nothing

# --- the list is never its own subject ----------------------------------------
# A record whose Files would reach the list itself must not refuse the file that
# declares it, or every invariant would need a lesson about skipping its own
# declaration.
d=$(make_repo list-not-a-subject 'Id: host-font-path
Files: *.rs *.txt
Prevents: a rendering that depends on what is installed on the machine.
Pattern: /usr/share/fonts

')
printf 'pub const COMPONENT: &str = "model";\n' | add "$d" crates/model/src/lib.rs
expect_clean "$d" "an invariant whose globs reach the list declaring it"

# --- the loader fails closed --------------------------------------------------
d=$(make_repo list-incomplete 'Id: host-font-path
Files: *.rs
Prevents: a rendering that depends on what is installed on the machine.

')
printf 'const DIR: &str = "/usr/share/fonts";\n' | add "$d" crates/render/src/lib.rs
expect_cannot_run "$d" "a record with no pattern"

d=$(make_repo list-empty '# nothing but this comment
')
printf 'const DIR: &str = "/usr/share/fonts";\n' | add "$d" crates/render/src/lib.rs
expect_cannot_run "$d" "a list holding no invariant"

d="$work/no-list"
mkdir -p "$d"
git -C "$d" init --quiet
git -C "$d" config core.autocrlf false
printf 'const DIR: &str = "/usr/share/fonts";\n' > "$d/a.rs"
git -C "$d" add -- a.rs
expect_cannot_run "$d" "a tree with no list at all"

echo
echo "$cases case(s), $failures failure(s)"
[ "$failures" -eq 0 ] || exit 1
