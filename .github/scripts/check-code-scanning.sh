#!/usr/bin/env bash
# Read the shipped code with the lints this repository has decided are about a
# hostile document, and refuse a finding at or above the declared threshold.
#
# Usage:
#   .github/scripts/check-code-scanning.sh [repository-root]
#   .github/scripts/check-code-scanning.sh [repository-root] --sarif <path>
#   .github/scripts/check-code-scanning.sh [repository-root] --flags
#
# The analyser is clippy, which is pinned in `rust-toolchain.toml` alongside the
# compiler, so this gate has no tool of its own to install and no version that
# can move without a commit. `docs/code-scanning.md` is where that choice is
# argued against the alternatives and where the residual is written down.
#
# The lints are in `.github/scripts/code-scanning-lints.txt` and this file holds
# none of them. The threshold is in `docs/quality-parity.md` and this file holds
# neither the number nor a default for it: a run that cannot read one stops
# rather than choosing.
#
# Every lint the register names is off by default, so the `Lint` gate beside
# this one reaches none of them and this is not that gate run twice. A lint at
# or above the threshold is passed as `-D` and fails the run. One below it is
# passed as `--force-warn`, which reports it and cannot be silenced by anything
# in a manifest, so a finding under the threshold is visible without being a
# refusal. `-A warnings` goes first so that the verdict here is about this
# register and nothing else; the default set is the other gate's subject.
#
# Two refusals:
#
#   finding-at-or-above-the-threshold  the analyser reported a construct the
#                                      register puts at or above the threshold,
#                                      named with the file, the line and the
#                                      lint
#   suppression-without-a-reason       a tracked source silences a lint with no
#                                      `reason = "..."` in the attribute, which
#                                      is how a gate keeps a green run while
#                                      covering less every month
#
# Exit 0 means the shipped code carries nothing at or above the threshold and
# every suppression says why. Exit 1 means at least one refusal. Exit 2 means
# the check could not run, which is a failure and never a pass: a missing or
# unreadable register, a record naming no lint or an unknown severity, a
# threshold the parity document does not declare or declares twice, a threshold
# naming a severity the register's vocabulary does not hold, and no cargo on
# PATH.
#
# WHAT THIS CANNOT JUDGE. It reads `--lib` and `--bins` and not the tests. A
# panic in a test is a failing test and a panic in the shipped code is the
# failure mode this register is about, and the `Lint` gate already reaches every
# target. So a test may `unwrap` and this says nothing about it, which is
# deliberate and is the largest thing this check does not cover.
#
# It also judges what a lint can see. Every one of these is a syntactic rule
# over one function at a time: it refuses the abort that is written down and
# says nothing about the one three calls away in a dependency, and nothing at
# all about whether a value came out of a document. `docs/code-scanning.md`
# carries that residual where a reader meets it rather than leaving it to be
# discovered from a green run.
#
# bash rather than sh: the register is read as separated records out of awk and
# the array and here-string forms below are bash.

set -euo pipefail

root=.
sarif=""
flags_only=0

while [ "$#" -gt 0 ]; do
  case "$1" in
    --sarif) shift; sarif=${1:-}; [ -n "$sarif" ] || { echo "check-code-scanning: --sarif takes a path" >&2; exit 2; }; shift ;;
    --flags) flags_only=1; shift ;;
    -*) echo "check-code-scanning: unknown option $1" >&2; exit 2 ;;
    *) root=$1; shift ;;
  esac
done

cd "$root" || { echo "check-code-scanning: cannot enter $root" >&2; exit 2; }

register=.github/scripts/code-scanning-lints.txt
parity=docs/quality-parity.md

[ -f "$register" ] || {
  echo "check-code-scanning: no register at $root/$register" >&2
  exit 2
}

[ -f "$parity" ] || {
  echo "check-code-scanning: no parity document at $root/$parity" >&2
  echo "check-code-scanning: the threshold is declared there and this check holds no default for it" >&2
  exit 2
}

# rank <severity>: the vocabulary, in one place, so the register and the
# threshold are judged against the same three words.
rank() {
  case "$1" in
    high) printf '3' ;;
    medium) printf '2' ;;
    low) printf '1' ;;
    *) printf '0' ;;
  esac
}

# The threshold, read out of the parity document rather than written here, so
# the number and the reasoning for it cannot drift apart. One line, at column
# zero, in the shape the coverage floor uses in `docs/test-harness.md`.
threshold=$(sed -n 's/^Code scanning threshold: \([a-z][a-z]*\)$/\1/p' "$parity")

if [ -z "$threshold" ]; then
  echo "check-code-scanning: $parity declares no threshold this gate can read" >&2
  echo "check-code-scanning: expected a line reading 'Code scanning threshold: <severity>' at column zero" >&2
  exit 2
fi

if [ "$(printf '%s\n' "$threshold" | wc -l)" -ne 1 ]; then
  echo "check-code-scanning: $parity declares more than one threshold" >&2
  exit 2
fi

if [ "$(rank "$threshold")" = "0" ]; then
  echo "check-code-scanning: $parity declares the threshold \`$threshold\`, which is not one of high, medium or low" >&2
  exit 2
fi

# The register as one line per record, with the record's first line number in
# front so a refusal can name where to go. The fields are separated by a unit
# separator rather than by a tab, for the reason the neighbouring checkers give:
# a tab is an IFS whitespace character, so `read` collapses a run of them and a
# record with an empty field would arrive with the next one shifted left.
table=$(awk '
  function emit() {
    if (started) printf "%s\037%s\037%s\037%s\n", line, lint, severity, prevents
    started = 0; line = 0; lint = ""; severity = ""; prevents = ""
  }
  /^#/ { next }
  /^[[:space:]]*$/ { emit(); next }
  {
    if (!started) { started = 1; line = NR }
    value = $0
  }
  /^Lint:/     { sub(/^Lint:[ ]?/, "", value);     lint = value;     next }
  /^Severity:/ { sub(/^Severity:[ ]?/, "", value); severity = value; next }
  /^Prevents:/ { sub(/^Prevents:[ ]?/, "", value); prevents = value; next }
  { printf "UNREADABLE\t%s\t%s\n", NR, $0 }
  END { emit() }
' "$register")

if printf '%s\n' "$table" | grep -q '^UNREADABLE'; then
  echo "check-code-scanning: $register holds a line no field name starts:" >&2
  printf '%s\n' "$table" | sed -n 's/^UNREADABLE\t/  line /p' >&2
  echo "check-code-scanning: refusing to judge a tree against a register it could not read" >&2
  exit 2
fi

records=0
denied=0
warned=0
lint_flags=()
declare -A severity_of=()

while IFS=$'\037' read -r line lint severity prevents; do
  [ -n "$line" ] || continue
  records=$((records + 1))

  if [ -z "$lint" ]; then
    echo "check-code-scanning: the record at $register:$line names no Lint" >&2
    exit 2
  fi

  if [ "$(rank "$severity")" = "0" ]; then
    echo "check-code-scanning: \`$lint\` at $register:$line carries the severity \`${severity:-<nothing>}\`" >&2
    echo "check-code-scanning: a lint whose severity is not one of high, medium or low cannot be placed against the threshold" >&2
    exit 2
  fi

  if [ -z "$prevents" ]; then
    echo "check-code-scanning: \`$lint\` at $register:$line says nothing about what it prevents" >&2
    echo "check-code-scanning: a lint nobody can argue for is one nobody can argue against either" >&2
    exit 2
  fi

  severity_of["$lint"]=$severity

  if [ "$(rank "$severity")" -ge "$(rank "$threshold")" ]; then
    lint_flags+=("-D" "$lint")
    denied=$((denied + 1))
  else
    lint_flags+=("--force-warn" "$lint")
    warned=$((warned + 1))
  fi
done <<< "$table"

if [ "$records" -eq 0 ]; then
  echo "check-code-scanning: $register holds no record" >&2
  echo "check-code-scanning: an empty register scans for nothing and would report a clean tree" >&2
  exit 2
fi

# `--flags` is how the workflow's upload step gets the same flag list the verdict
# was made with, so the two cannot drift into scanning different things.
if [ "$flags_only" -eq 1 ]; then
  printf '%s\n' "-A" "warnings" "${lint_flags[@]}"
  exit 0
fi

command -v cargo >/dev/null 2>&1 || { echo "check-code-scanning: no cargo on PATH" >&2; exit 2; }

refusals=0

refuse() {
  # refuse <property> <subject> <detail>
  echo "REFUSED $1: $2 - $3"
  refusals=$((refusals + 1))
}

# What is being read, and with what, printed before any verdict, so a run that
# scanned one lint cannot be read as one that scanned the set.
echo "threshold read from $parity: $threshold"
echo "read $records lint(s) from $register: $denied denied, $warned forced to warn and reported"
echo "subject: cargo clippy --locked --workspace --lib --bins"

set +e
scan=$(cargo clippy --locked --workspace --lib --bins --color never -- -A warnings "${lint_flags[@]}" 2>&1)
scan_status=$?
set -e

printf '%s\n' "$scan" | sed 's/^/  /'

# Every diagnostic the run produced, as `<level><US><file:line:col><US><lint><US><message>`.
#
# A clippy diagnostic is a level line, then an arrow line carrying the location,
# then notes, one of which is a link whose fragment is the lint's own name. The
# summary lines at the end of a failed run carry no arrow line, which is what
# keeps them out of this.
findings=$(printf '%s\n' "$scan" | awk '
  function flush() {
    if (open && loc != "") printf "%s\037%s\037%s\037%s\n", level, loc, (lint == "" ? "-" : lint), message
    open = 0; level = ""; loc = ""; lint = ""; message = ""
  }
  /^error: / { flush(); open = 1; level = "error"; message = substr($0, 8); next }
  /^warning: / { flush(); open = 1; level = "warning"; message = substr($0, 10); next }
  open && /^[[:space:]]*--> / { loc = $0; sub(/^[[:space:]]*--> /, "", loc); next }
  open && /rust-clippy\/.*index\.html#/ {
    lint = $0
    sub(/^.*index\.html#/, "", lint)
    sub(/[^A-Za-z0-9_].*$/, "", lint)
    lint = "clippy::" lint
    next
  }
  /^[[:space:]]*$/ { flush(); next }
  END { flush() }
')

reported=0
while IFS=$'\037' read -r level loc lint message; do
  [ -n "$loc" ] || continue
  reported=$((reported + 1))
  severity=${severity_of[$lint]:-unregistered}
  if [ "$level" = "error" ]; then
    refuse finding-at-or-above-the-threshold "$loc" \
      "\`$lint\` is $severity, the threshold is $threshold, and the code says: $message"
  else
    echo "reported below the threshold: $loc - \`$lint\` is $severity - $message"
  fi
done <<< "$findings"

# A run that failed for a reason no diagnostic explains is not a clean tree. The
# analyser not finishing is the case a gate must never read as nothing found.
if [ "$scan_status" -ne 0 ] && [ "$refusals" -eq 0 ]; then
  echo "check-code-scanning: the analyser exited $scan_status and no diagnostic above accounts for it" >&2
  echo "check-code-scanning: a scan that did not finish is not a scan that found nothing" >&2
  exit 2
fi

# Every suppression in tracked Rust, and whether it says why.
#
# `#[allow(...)]` and `#[expect(...)]`, inner and outer, possibly spread over
# several lines. The attribute is accumulated until its closing bracket and then
# read for a `reason = "..."`, which is the field the compiler itself carries for
# this and which a diff shows beside the code it excuses.
suppressions=0
while IFS=$'\t' read -r file line text; do
  [ -n "$file" ] || continue
  suppressions=$((suppressions + 1))
  case "$text" in
    *'reason'*'='*'"'*) ;;
    *)
      refuse suppression-without-a-reason "$file:$line" \
        "$(printf '%s' "$text" | cut -c1-72) silences a lint and says nothing about why"
      ;;
  esac
done <<< "$(
  find . -name target -prune -o -name '*.rs' -print 2>/dev/null | LC_ALL=C sort | while IFS= read -r file; do
    awk -v file="${file#./}" '
      function reset() { open = 0; depth = 0; buf = ""; start = 0 }
      {
        line = $0
        if (!open && line ~ /#!?\[[[:space:]]*(allow|expect)[[:space:]]*\(/) {
          open = 1; start = NR; buf = ""
          sub(/^.*#!?\[/, "", line)
        }
        if (!open) next
        buf = buf (buf == "" ? "" : " ") line
        n = split(line, chars, "")
        for (i = 1; i <= n; i++) {
          if (chars[i] == "(") depth++
          else if (chars[i] == ")") depth--
        }
        if (depth <= 0) {
          gsub(/\t/, " ", buf)
          printf "%s\t%s\t%s\n", file, start, buf
          reset()
        }
      }
    ' "$file"
  done
)"

echo "read $reported diagnostic(s) from the analyser and $suppressions suppression(s) in tracked sources"

if [ "$refusals" -ne 0 ]; then
  echo "check-code-scanning: $refusals refusal(s). docs/code-scanning.md says what each one is for."
  [ -n "$sarif" ] && write_sarif=1 || write_sarif=0
else
  [ -n "$sarif" ] && write_sarif=1 || write_sarif=0
fi

# The same findings as SARIF, for the code scanning tab. Written here rather than
# by a converter fetched at run time, so the gate needs nothing this repository
# has not already pinned and can be reproduced offline exactly as it runs.
if [ "$write_sarif" -eq 1 ]; then
  {
    printf '{\n'
    printf '  "version": "2.1.0",\n'
    printf '  "$schema": "https://json.schemastore.org/sarif-2.1.0.json",\n'
    printf '  "runs": [\n'
    printf '    {\n'
    printf '      "tool": { "driver": { "name": "clippy", "informationUri": "https://github.com/rust-lang/rust-clippy" } },\n'
    printf '      "results": [\n'
    first=1
    while IFS=$'\037' read -r level loc lint message; do
      [ -n "$loc" ] || continue
      uri=$(printf '%s' "${loc%%:*}" | tr '\\' '/')
      rest=${loc#*:}
      start_line=${rest%%:*}
      case "$start_line" in ''|*[!0-9]*) start_line=1 ;; esac
      esc_message=$(printf '%s' "$message" | sed 's/\\/\\\\/g; s/"/\\"/g')
      esc_uri=$(printf '%s' "$uri" | sed 's/\\/\\\\/g; s/"/\\"/g')
      [ "$first" -eq 1 ] || printf ',\n'
      first=0
      printf '        {\n'
      printf '          "ruleId": "%s",\n' "$lint"
      printf '          "level": "%s",\n' "$level"
      printf '          "message": { "text": "%s" },\n' "$esc_message"
      printf '          "locations": [ { "physicalLocation": { "artifactLocation": { "uri": "%s" }, "region": { "startLine": %s } } } ]\n' "$esc_uri" "$start_line"
      printf '        }'
    done <<< "$findings"
    [ "$first" -eq 1 ] || printf '\n'
    printf '      ]\n'
    printf '    }\n'
    printf '  ]\n'
    printf '}\n'
  } > "$sarif"
  echo "wrote $reported result(s) to $sarif"
fi

[ "$refusals" -eq 0 ] || exit 1

echo "check-code-scanning: the shipped code carries nothing at or above $threshold, and every suppression says why"
