#!/usr/bin/env bash
# encode-image-datauri.sh
# Encode an image as a base64 data URI and emit a Markdown reference-style
# image definition suitable for appending to the bottom of a document.
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: encode-image-datauri.sh -i <image> [-l <label>] [-a <alt>] [-o <markdown-file>]
  -i  Path to the source image (png, jpg, jpeg, gif, webp, svg). Required.
  -l  Reference label (default: derived from the file name).
  -a  Alt text for the in-body reference (default: the label).
  -o  Markdown file to append the definition to (default: print to stdout).
USAGE
}

img=""; label=""; alt=""; out=""
while getopts ":i:l:a:o:h" opt; do
  case "$opt" in
    i) img="$OPTARG" ;;
    l) label="$OPTARG" ;;
    a) alt="$OPTARG" ;;
    o) out="$OPTARG" ;;
    h) usage; exit 0 ;;
    *) usage >&2; exit 2 ;;
  esac
done

[ -n "$img" ] || { echo "error: -i <image> is required" >&2; usage >&2; exit 2; }
[ -f "$img" ] || { echo "error: file not found: $img" >&2; exit 1; }

# Derive a default label from the basename: strip extension, lowercase, hyphenate.
if [ -z "$label" ]; then
  base="$(basename "$img")"
  label="img-$(printf '%s' "${base%.*}" | tr '[:upper:] ' '[:lower:]-' | tr -cd 'a-z0-9-')"
fi
[ -n "$alt" ] || alt="$label"

# Resolve the MIME type: prefer `file`, fall back to an extension map.
mime=""
if command -v file >/dev/null 2>&1; then
  mime="$(file -b --mime-type "$img" 2>/dev/null || true)"
fi
if [ -z "$mime" ] || [ "$mime" = "application/octet-stream" ]; then
  ext="$(printf '%s' "${img##*.}" | tr '[:upper:]' '[:lower:]')"
  case "$ext" in
    png) mime="image/png" ;;
    jpg|jpeg) mime="image/jpeg" ;;
    gif) mime="image/gif" ;;
    webp) mime="image/webp" ;;
    svg) mime="image/svg+xml" ;;
    *) echo "error: unsupported image type: .$ext" >&2; exit 1 ;;
  esac
fi

# Base64 with no line wrapping, portable across GNU (-w0 unsupported on BSD).
b64="$(base64 < "$img" | tr -d '\n')"
definition="[$label]: data:${mime};base64,${b64}"

if [ -n "$out" ]; then
  printf '\n%s\n' "$definition" >> "$out"
  echo "Appended reference '[$label]' to $out" >&2
  echo "In-body usage: ![$alt][$label]" >&2
else
  echo "In-body usage (place where the image should appear):" >&2
  echo "  ![$alt][$label]" >&2
  echo >&2
  echo "Reference definition (place at the BOTTOM of the document):" >&2
  printf '%s\n' "$definition"
fi
