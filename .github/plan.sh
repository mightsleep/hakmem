#!/usr/bin/env bash
# Which checks of one system CI has to build: those whose output path is
# not in the binary cache. Nix is input-addressed, so the path of a check
# is known before anything is built, and a path exists in the cache only
# if that exact derivation was built and passed. Writes two JSON arrays to
# $GITHUB_OUTPUT (stdout when unset): `checks` to build, `cached` to skip.
#
#   plan.sh <system> [cache-url]
#
# A cache lookup that fails for any reason counts as a miss: building a
# cell twice is cheap, skipping a stale one is not. PLAN_PATHS=<file>
# reads the {check: outPath} map from a file instead of evaluating (tests).
set -euo pipefail

system=$1
cache=${2:-https://hakmem.cachix.org}
out=${GITHUB_OUTPUT:-/dev/stdout}

if [ -n "${PLAN_PATHS:-}" ]; then
  paths=$(cat "$PLAN_PATHS")
else
  paths=$(nix eval --json ".#checks.$system" --apply 'c: builtins.mapAttrs (_: d: d.outPath) c')
fi

build=()
cached=()
while IFS=$'\t' read -r name path; do
  hash=${path#/nix/store/}
  hash=${hash%%-*}
  if curl -sfI --max-time 30 -o /dev/null "$cache/$hash.narinfo"; then
    cached+=("$name")
    echo "cached  $name"
  else
    build+=("$name")
    echo "build   $name"
  fi
done < <(jq -r 'to_entries[] | [.key, .value] | @tsv' <<<"$paths")

json() {
  if [ $# -eq 0 ]; then echo '[]'; else printf '%s\n' "$@" | jq -Rc . | jq -sc .; fi
}
{
  echo "checks=$(json "${build[@]}")"
  echo "cached=$(json "${cached[@]}")"
} >> "$out"

if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  {
    echo "### $system: ${#build[@]} to build, ${#cached[@]} in cache"
    for c in "${build[@]}"; do echo "- build \`$c\`"; done
    for c in "${cached[@]}"; do echo "- cached \`$c\`"; done
  } >> "$GITHUB_STEP_SUMMARY"
fi
