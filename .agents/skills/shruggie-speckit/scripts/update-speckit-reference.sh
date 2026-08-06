#!/usr/bin/env bash
# update-speckit-reference.sh
# Maintainer aid: fetch the current upstream spec-kit command templates and docs
# into a scratch directory so a maintainer can diff them against
# assets/speckit-reference.md and refresh it. This is never run at skill
# runtime. It performs a read-only network fetch and does not modify the repo.
set -euo pipefail

REPO="${SPECKIT_REPO:-https://github.com/github/spec-kit.git}"
REF="${SPECKIT_REF:-main}"
outdir=""

usage() {
  cat <<'USAGE'
Usage: update-speckit-reference.sh -o <out-dir>
  -o  Scratch directory to receive the fetched command templates and docs. Required.
  -h  Show this help.

Environment:
  SPECKIT_REPO  Upstream git URL (default: https://github.com/github/spec-kit.git)
  SPECKIT_REF   Branch or tag to fetch (default: main)

The upstream layout can change. If the expected paths are absent, the script
copies whatever command/doc material it finds and prints a note; refresh the
reference by hand against the upstream repository if needed.
USAGE
}

while getopts ":o:h" opt; do
  case "$opt" in
    o) outdir="$OPTARG" ;;
    h) usage; exit 0 ;;
    :) echo "error: -$OPTARG requires an argument" >&2; usage; exit 2 ;;
    *) echo "error: unknown option -$OPTARG" >&2; usage; exit 2 ;;
  esac
done

if [ -z "$outdir" ]; then
  echo "error: -o <out-dir> is required" >&2
  usage
  exit 2
fi

if ! command -v git >/dev/null 2>&1; then
  echo "error: git is required" >&2
  exit 1
fi

mkdir -p "$outdir"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "Cloning $REPO ($REF) ..."
git clone --depth 1 --branch "$REF" "$REPO" "$tmp/spec-kit" >/dev/null 2>&1 \
  || git clone --depth 1 "$REPO" "$tmp/spec-kit" >/dev/null 2>&1

found=0
for path in "templates/commands" "docs" "spec-driven.md" "README.md"; do
  src="$tmp/spec-kit/$path"
  if [ -e "$src" ]; then
    dest="$outdir/$(basename "$path")"
    cp -R "$src" "$dest"
    echo "  fetched: $path -> $dest"
    found=1
  fi
done

if [ "$found" -eq 0 ]; then
  echo "note: none of the expected upstream paths were found; the upstream layout"
  echo "      may have changed. Inspect $tmp/spec-kit and refresh the reference by hand." >&2
fi

echo "Done. Diff the fetched material against assets/speckit-reference.md."
