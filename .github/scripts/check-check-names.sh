#!/usr/bin/env bash
# Refuse a check name that only one of the two sides knows about.
#
# Usage:
#   .github/scripts/check-check-names.sh [root]
#
# A branch protection rule matches a check by its literal name. The name comes
# from a job's `name:` field, or, where a job has none, from the job id. So a
# workflow edit that adds, removes or retypes one of those lines renames the
# check, and the rule that required the old name is now guarding nothing while
# every run on the pull request stays green. That failure has no symptom, which
# is why it gets a checker rather than a paragraph.
#
# Three refusals:
#
#   job-without-a-name          a job with no explicit `name:`, whose check is
#                               therefore named after its id and moves whenever
#                               somebody renames the id
#   check-not-documented        a job whose check name docs/checks.md does not
#                               carry, so a reader cannot find out what
#                               reproduces it
#   documented-check-is-absent  a name docs/checks.md carries that no job
#                               produces, which is the document describing a
#                               gate that is not running
#
# It reads the two sides and compares them. It says nothing about whether a name
# is required by a rule on the default branch, which is a repository setting and
# not in the tree.

set -euo pipefail

root=${1:-.}
workflows="$root/.github/workflows"
document="$root/docs/checks.md"

[ -d "$workflows" ] || { echo "no workflow directory at $workflows" >&2; exit 2; }
[ -f "$document" ] || { echo "no document at $document" >&2; exit 2; }

refusals=0

refuse() {
  # refuse <property> <subject> <detail>
  echo "REFUSED $1: $2 - $3"
  refusals=$((refusals + 1))
}

# Every job in every workflow, as `file<TAB>id<TAB>name`, where an absent
# job-level name is reported as the empty string. A job-level `name:` sits at
# exactly four spaces; a step's sits deeper and a workflow's at column zero, so
# the indent is what tells them apart.
jobs_in_tree() {
  local file
  for file in "$workflows"/*.yml "$workflows"/*.yaml; do
    [ -f "$file" ] || continue
    awk -v file="$file" '
      function emit() {
        if (job != "") printf "%s\t%s\t%s\n", file, job, jobname
        job = ""; jobname = ""
      }
      /^jobs:[ \t]*$/ { injobs = 1; next }
      injobs && /^[^ \t#]/ { emit(); injobs = 0 }
      injobs && /^  [A-Za-z0-9_.-]+:[ \t]*$/ {
        emit()
        job = $0; sub(/^  /, "", job); sub(/:[ \t]*$/, "", job)
        next
      }
      injobs && job != "" && jobname == "" && /^    name:[ \t]/ {
        jobname = $0
        sub(/^    name:[ \t]*/, "", jobname)
        sub(/[ \t]+$/, "", jobname)
        gsub(/^"|"$/, "", jobname)
        gsub(/^'"'"'|'"'"'$/, "", jobname)
        next
      }
      END { emit() }
    ' "$file"
  done
}

# Every check name docs/checks.md carries: the first backticked field of a table
# row. The document is free to say anything else it likes anywhere else.
documented_names() {
  sed -n 's/^| `\([^`]*\)` |.*/\1/p' "$document"
}

jobs=$(jobs_in_tree)
[ -n "$jobs" ] || { echo "read no jobs out of $workflows; refusing to report a clean tree" >&2; exit 2; }

documented=$(documented_names)
[ -n "$documented" ] || { echo "read no check names out of $document; refusing to report a clean tree" >&2; exit 2; }

produced=""
while IFS=$'\t' read -r file id name; do
  [ -n "$file" ] || continue
  if [ -z "$name" ]; then
    refuse job-without-a-name "${file#"$root"/}:$id" \
      "the check is named after the job id, so renaming the id renames the check"
    continue
  fi
  produced="$produced$name
"
  if ! printf '%s\n' "$documented" | grep -qxF "$name"; then
    refuse check-not-documented "${file#"$root"/}:$id" \
      "no row in docs/checks.md carries \`$name\`"
  fi
done <<< "$jobs"

while IFS= read -r name; do
  [ -n "$name" ] || continue
  if ! printf '%s' "$produced" | grep -qxF "$name"; then
    refuse documented-check-is-absent "docs/checks.md" \
      "\`$name\` is documented and no job in .github/workflows/ produces it"
  fi
done <<< "$documented"

count=$(printf '%s\n' "$jobs" | grep -c . || true)
echo "examined $count job(s) against $(printf '%s\n' "$documented" | grep -c . || true) documented name(s)"
if [ "$refusals" -ne 0 ]; then
  echo "check-check-names: $refusals refusal(s)"
  exit 1
fi
echo "check-check-names: every job names itself and every name is documented"
