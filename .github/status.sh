#!/usr/bin/env bash
# The result of every job of one CI run as shields.io endpoint JSON, one
# file per job plus all.json for the run, under <outdir>/<system>/.
#
#   status.sh <system> <jobs.json> <outdir>
#
# jobs.json is the /repos/{repo}/actions/runs/{id}/jobs response; a job's
# name is its check name (the workflows set `name: ${{ matrix.check }}`).
# `plan` and `status` are bookkeeping, not checks, and are left out.
# Colours are Catppuccin Mocha: green pass, red fail, overlay0 otherwise.
set -euo pipefail

system=$1
jobs=$2
out=$3/$system
mkdir -p "$out"

badge() {
  case $1 in
    success) echo "pass a6e3a1" ;;
    failure) echo "fail f38ba8" ;;
    cancelled | skipped) echo "$1 6c7086" ;;
    *) echo "$1 f9e2af" ;;
  esac
}

overall=success
while IFS=$'\t' read -r name conclusion; do
  read -r message colour < <(badge "$conclusion")
  jq -n --arg m "$message" --arg c "$colour" \
    '{schemaVersion: 1, label: "", message: $m, color: $c, labelColor: "313244"}' \
    > "$out/$name.json"
  [ "$conclusion" = success ] || overall=failure
done < <(jq -r '.jobs[] | select(.name != "plan" and .name != "status") | [.name, (.conclusion // "unknown")] | @tsv' "$jobs")

read -r message colour < <(badge "$overall")
jq -n --arg l "${system%%-*}" --arg m "$message" --arg c "$colour" \
  '{schemaVersion: 1, label: $l, message: $m, color: $c, labelColor: "313244"}' \
  > "$out/all.json"
