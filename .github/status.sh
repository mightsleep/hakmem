#!/usr/bin/env bash
# The result of every job of one CI run as shields.io endpoint JSON, one
# file per job plus all.json for the run, under <outdir>/<system>/.
#
#   status.sh <system> <jobs.json> <outdir> [cached.json]
#
# jobs.json is the /repos/{repo}/actions/runs/{id}/jobs response; a job's
# name is its check name (the workflows set `name: ${{ matrix.check }}`).
# `plan` and `status` are bookkeeping, not checks, and are left out. The
# optional cached.json is the plan's `cached` array: cells it skipped because
# their output was already in the binary cache, which only a passing build
# produces, so they are written as pass.
# Colours are Catppuccin Mocha: green pass, red fail, overlay0 otherwise.
set -euo pipefail

system=$1
jobs=$2
out=$3/$system
cached=${4:-}
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

overall=success
# A skipped job did not run a check. When the cache holds every cell the
# matrix is empty, and GitHub reports it as one skipped job named
# `matrix.check`, which used to paint the whole badge red: CI failing
# because there was nothing left to fail.
rm -f "$out/matrix.check.json"
while IFS=$'\t' read -r name conclusion; do
  [ "$conclusion" = skipped ] && continue
  read -r message colour < <(badge "$conclusion")
  write "" "$message" "$colour" "$name"
  [ "$conclusion" = success ] || overall=failure
done < <(jq -r '.jobs[] | select(.name != "plan" and .name != "status") | [.name, (.conclusion // "unknown")] | @tsv' "$jobs")

if [ -n "$cached" ]; then
  while IFS= read -r name; do
    write "" pass a6e3a1 "$name"
  done < <(jq -r '.[]' "$cached")
fi

read -r message colour < <(badge "$overall")
write "${system%%-*}" "$message" "$colour" all
