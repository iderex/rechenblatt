#!/usr/bin/env bash
# Refuse a tracked file holding something the invariant list says may not be there.
#
# Usage:
#   .github/scripts/check-invariants.sh [repository-root]
#
# Exit 0 means every tracked path passed every invariant that reads it. Exit 1
# means at least one refusal, each printed with the property it broke, the file
# and line it broke it on, and the sentence saying what that invariant prevents.
# Exit 2 means the check could not run, which is a failure and never a pass.
#
# THE LIST IS NOT IN HERE. `.github/scripts/invariants.txt` holds it, one record
# per invariant, and this file holds no pattern of its own. That is the whole
# shape of it: an invariant is added by writing a record and a leg, and a
# reviewer reads the record rather than the walk. `docs/invariants.md` argues it
# and says what this instrument cannot judge.
#
# The three refusals:
#
#   invariant-violated          a tracked file the invariant's pattern matches,
#                               named with the line number
#   exception-without-a-reason  an `Except:` naming a path and nothing else, so
#                               the next reader cannot tell whether it still
#                               applies
#   exception-names-nothing     an `Except:` whose prefix matches no tracked
#                               path, which is a rule that quietly stopped
#                               covering a file that moved
#
# The last two are why the exception register fails closed in both directions. An
# exception is how a file that is ABOUT a pattern gets to name it, and a register
# nothing audits is the hole a rule leaves through.
#
# The subject is the tracked path list from `git ls-files`, read in the working
# tree. That differs from the tracked-bytes checker on purpose: this one is about
# what the text says rather than about which bytes git stored, and a pattern is
# the same either way.
#
# bash rather than sh: the walk reads `git ls-files -z`, and a NUL-delimited read
# is a bash feature. A newline in a path is legal, and a checker that word-splits
# its input is a checker that refuses valid work.

set -euo pipefail

root=${1:-.}
cd "$root" || { echo "check-invariants: cannot enter $root" >&2; exit 2; }

if ! git rev-parse --git-dir >/dev/null 2>&1; then
  echo "check-invariants: $root is not a git repository" >&2
  exit 2
fi

list=.github/scripts/invariants.txt
[ -f "$list" ] || { echo "check-invariants: no invariant list at $root/$list" >&2; exit 2; }

# The tracked paths, NUL-delimited on the way in and newline-joined afterwards.
# A path holding a newline would break the joined form, so it is refused as
# unreadable rather than silently mangled.
tracked=""
while IFS= read -r -d '' path; do
  case "$path" in
    *$'\n'*)
      echo "check-invariants: $path holds a newline and this walk cannot name it" >&2
      exit 2
      ;;
  esac
  tracked="$tracked$path
"
done < <(git ls-files -z)

[ -n "$tracked" ] || { echo "check-invariants: read no tracked paths; refusing to report a clean tree" >&2; exit 2; }

refusals=0
honoured=0
invariants=0
examined=0

refuse() {
  # refuse <property> <subject> <detail>
  echo "REFUSED $1: $2 - $3"
  refusals=$((refusals + 1))
}

# Whether a tracked path is one this invariant reads. Nothing is read by default:
# a record with no `Files` never gets here, because the loader refuses it.
reads_it() {
  local path=$1 globs=$2 glob
  for glob in $globs; do
    # shellcheck disable=SC2254 # the glob is the pattern, which is the point
    case "$path" in
      $glob) return 0 ;;
    esac
  done
  return 1
}

# One invariant against the whole tracked list.
apply() {
  local id=$1 globs=$2 pattern=$3 prevents=$4 exceptions=$5
  local path prefix reason hits line entry candidate covers

  # The exception register first, so a dangling or unexplained exception is
  # reported even on a tree that violates nothing.
  while IFS= read -r entry; do
    [ -n "$entry" ] || continue
    prefix=${entry%%$'\t'*}
    reason=${entry#*$'\t'}
    if [ -z "$reason" ] || [ "$reason" = "$prefix" ]; then
      refuse exception-without-a-reason "$list:$id" \
        "\`$prefix\` is excepted from $id and the record does not say why"
      continue
    fi
    covers=0
    while IFS= read -r candidate; do
      [ -n "$candidate" ] || continue
      case "$candidate" in
        "$prefix"*) covers=1; break ;;
      esac
    done <<< "$tracked"
    if [ "$covers" -eq 0 ]; then
      refuse exception-names-nothing "$list:$id" \
        "\`$prefix\` is excepted from $id and no tracked path starts with it"
    fi
  done <<< "$exceptions"

  while IFS= read -r path; do
    [ -n "$path" ] || continue
    reads_it "$path" "$globs" || continue
    [ -f "$path" ] || continue
    # The list is never a subject. It holds every pattern by definition, so an
    # invariant whose `Files` reached it would refuse the file declaring it, and
    # a rule that has to be taught to skip its own declaration is a rule with a
    # hole in the shape of that lesson.
    if [ "$path" = "$list" ]; then continue; fi

    local skipped=""
    while IFS= read -r entry; do
      [ -n "$entry" ] || continue
      prefix=${entry%%$'\t'*}
      case "$path" in
        "$prefix"*) skipped=$prefix ;;
      esac
    done <<< "$exceptions"
    if [ -n "$skipped" ]; then
      echo "allowed $id: $path under the exception on \`$skipped\`"
      honoured=$((honoured + 1))
      continue
    fi

    examined=$((examined + 1))
    hits=$(grep -nE -- "$pattern" "$path" || true)
    [ -n "$hits" ] || continue
    while IFS= read -r line; do
      [ -n "$line" ] || continue
      refuse invariant-violated "$path:${line%%:*}" "$id - $prevents"
    done <<< "$hits"
  done <<< "$tracked"
}

# The loader. A record is fields until a blank line. Four are required and a
# record missing one stops the run: a list this reads as empty, or as three
# invariants where the file holds four, would report a clean tree it never
# examined.
id=""; globs=""; pattern=""; prevents=""; exceptions=""; started=0

flush() {
  [ "$started" -eq 1 ] || return 0
  local missing=""
  [ -n "$id" ] || missing="Id"
  [ -n "$globs" ] || missing="${missing:+$missing, }Files"
  [ -n "$prevents" ] || missing="${missing:+$missing, }Prevents"
  [ -n "$pattern" ] || missing="${missing:+$missing, }Pattern"
  if [ -n "$missing" ]; then
    echo "check-invariants: a record in $list is missing $missing" >&2
    echo "check-invariants: refusing to judge a tree against a list it could not read" >&2
    exit 2
  fi
  invariants=$((invariants + 1))
  apply "$id" "$globs" "$pattern" "$prevents" "$exceptions"
  id=""; globs=""; pattern=""; prevents=""; exceptions=""; started=0
}

while IFS= read -r line || [ -n "$line" ]; do
  case "$line" in
    '#'*) continue ;;
    '')  flush; continue ;;
  esac
  started=1
  case "$line" in
    'Id: '*)       id=${line#Id: } ;;
    'Files: '*)    globs=${line#Files: } ;;
    'Prevents: '*) prevents=${line#Prevents: } ;;
    'Pattern: '*)  pattern=${line#Pattern: } ;;
    'Except: '*)
      entry=${line#Except: }
      exceptions="$exceptions${entry%% *}	${entry#* }
"
      ;;
    *)
      echo "check-invariants: $list holds a line no field name starts: $line" >&2
      exit 2
      ;;
  esac
done < "$list"
flush

[ "$invariants" -gt 0 ] || { echo "check-invariants: read no invariants out of $list; refusing to report a clean tree" >&2; exit 2; }

# Say what was examined, so a run that covered less than the whole set cannot be
# read as one that covered it and found nothing.
echo "read $invariants invariant(s) from $list; examined $examined file-and-invariant pair(s); honoured $honoured declared exception(s)"

if [ "$refusals" -ne 0 ]; then
  echo "check-invariants: $refusals refusal(s). docs/invariants.md says what each invariant is for."
  exit 1
fi

echo "check-invariants: every tracked path passes every invariant that reads it"
