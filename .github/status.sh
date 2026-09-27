#!/usr/bin/env bash
# The result of every cell of one CI run as shields.io endpoint JSON, one
# file per cell plus all.json for the run, under <outdir>/<system>/.
#
#   status.sh <system> <jobs.json> <outdir> <cells.json>
#
# cells.json says what the run was supposed to produce:
#   {"plan": "<result of the plan job>", "build": [...], "cached": [...],
#    "jobs": [...]}
# `build` and `cached` are the plan's two arrays, `jobs` the cells that are
# plain jobs rather than matrix entries (miri). Only these get a file. A
# cached cell is a pass: its output is in the binary cache, which only a
# passing build produces. The rest take their conclusion from jobs.json,
# the /repos/{repo}/actions/runs/{id}/jobs response (the matrix sets
# `name: ${{ matrix.check }}`); one with no job there is `missing`.
#
# The list used to be every job of the run minus the known bookkeeping,
# and each new job taught the lesson again: the empty matrix's skipped
# `matrix.check`, then the `<arch> ok` gate, caught still running by the
# listing, all painted a green run red. Files of anything no longer a cell
# are deleted, so a renamed check leaves no stale badge either.
# Colours are Catppuccin Mocha: green pass, red fail, overlay0 otherwise.
#
#   status.sh --pending <system> <checks.json> <outdir>
#
# Before the build: a grey `pending` for every check in checks.json that
# has no file yet. A badge whose file is missing is shields.io's red
# "resource not found", and a check new in a merge had no file until its
# run finished, so every new row was red for the length of a run and the
# pages cache after it. Files that exist keep their last result.
set -euo pipefail

pending=
if [ "${1:-}" = --pending ]; then
  pending=1
  shift
fi

system=$1
jobs=$2
out=$3/$system
cells=${4:-}
mkdir -p "$out"

badge() {
  case $1 in
    success) echo "pass a6e3a1" ;;
    failure) echo "fail f38ba8" ;;
    cancelled | skipped) echo "$1 6c7086" ;;
    *) echo "$1 f9e2af" ;;
  esac
}

write() {
  jq -n --arg l "$1" --arg m "$2" --arg c "$3" \
    '{schemaVersion: 1, label: $l, message: $m, color: $c, labelColor: "313244"}' \
    > "$out/$4.json"
}

if [ -n "$pending" ]; then
  while IFS= read -r name; do
    [ -e "$out/$name.json" ] || write "" pending 6c7086 "$name"
  done < <(jq -r '.[]' "$jobs")
  exit 0
fi

# A failed plan builds nothing and lists nothing: the run failed, and
# without a list of cells there is nothing to tell stale files by.
overall=success
plan=$(jq -r '.plan' "$cells")
[ "$plan" = success ] || overall=failure

declare -A keep=([all]=1)
while IFS=$'\t' read -r name conclusion; do
  keep[$name]=1
  read -r message colour < <(badge "$conclusion")
  write "" "$message" "$colour" "$name"
  [ "$conclusion" = success ] || overall=failure
done < <(jq -r --slurpfile run "$jobs" '
  ($run[0].jobs | map({(.name): (.conclusion // "unknown")}) | add // {}) as $ran
  | (.cached[] | [., "success"]),
    ((.build + .jobs)[] | [., ($ran[.] // "missing")])
  | @tsv' "$cells")

[ "$plan" = success ] && for f in "$out"/*.json; do
  name=$(basename "$f" .json)
  [ -n "${keep[$name]:-}" ] || rm -f "$f"
done

read -r message colour < <(badge "$overall")
write "${system%%-*}" "$message" "$colour" all
