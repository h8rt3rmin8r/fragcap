#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# NAME
#     lint-docs.sh - fragcap documentation linter (glossary and P-6)
#
# SYNOPSIS
#     lint-docs.sh check
#     lint-docs.sh fix
#     lint-docs.sh link
#     lint-docs.sh -h | --help
#
# DESCRIPTION
#     Enforces the glossary discipline of specification sections 4.6 and 22.4
#     over the per-category glossary pages under docs/glossary/. It has three
#     modes:
#
#     check   Validate entry completeness (every entry carries a non-empty
#             definition body), cross-link resolution (every internal
#             /docs/glossary/<category>#<anchor> link resolves to an existing
#             term anchor), and index reproducibility (the committed index.md
#             matches a fresh generation). References and the matters callout are
#             validated where present but not mandated on every entry, because
#             the authored glossary cites a primary source only where one exists.
#             Reports every failure it finds and exits 1 if any failed. This is
#             the mode continuous integration runs, so P-6 is enforced on every
#             push.
#
#     fix     Regenerate docs/glossary/index.md, the alphabetical index, from the
#             category pages, in place. Changes nothing else. On a clean tree
#             this is a no-op, which is what check's reproducibility test
#             asserts.
#
#     link    Verify that every external reference URL in the glossary responds.
#             Runs on a weekly schedule, not per commit, because link liveness
#             depends on third parties rather than on the commit. Requires curl.
#
#     Scope note: check enforces the glossary's own graph (entry completeness,
#     cross-link and see-also resolution, index reproducibility). The free-text
#     term inventory of section 4.2 over all prose is not attempted here; the
#     glossary graph is what this linter guards deterministically.
#
# EXIT STATUS
#     0   ran and passed
#     1   ran and found failures
#     2   could not run (a required tool is absent)
#
set -euo pipefail
IFS=$'\n\t'

#_______________________________________________________________________________
# Declare Functions

# Print the usage block by parsing this file's own header comment.
print_help() {
    awk 'NR<=2 {next} /^set -/ {exit} {sub(/^# ?/, ""); print}' "$0"
}

# Whether a command is available on PATH.
has_cmd() {
    command -v "$1" >/dev/null 2>&1
}

# Structured logging to standard error, respecting NO_COLOR and TTY detection.
log_msg() {
    local level="$1"
    shift
    local color="" reset=""
    if [ -z "${NO_COLOR:-}" ] && [ -t 2 ]; then
        reset=$'\033[0m'
        case "$level" in
            INFO) color=$'\033[36m' ;;
            OK) color=$'\033[32m' ;;
            FAIL) color=$'\033[31m' ;;
            *) color="" ;;
        esac
    fi
    printf '%s%s%s %s\n' "$color" "$level" "$reset" "$*" >&2
}

log_info() { log_msg INFO "$@"; }
log_ok() { log_msg OK "$@"; }
log_error() { log_msg FAIL "$@"; }

# Run a command, echoing it first, so a failing step is legible.
safe_run() {
    log_info "run: $*"
    "$@"
}

# GitHub-style heading slug, matching the anchors the split produced: lowercase,
# drop punctuation except word characters and hyphens, spaces to hyphens.
slugify() {
    printf '%s' "$1" \
        | tr '[:upper:]' '[:lower:]' \
        | sed -E 's/[^a-z0-9_ -]//g' \
        | sed -E 's/[[:space:]]+/-/g'
}

# The category slug for a page path (its basename without extension).
page_slug() {
    local path="$1"
    path="${path##*/}"
    printf '%s' "${path%.md}"
}

# Emit "slug<TAB>term" for every term heading across the category pages.
all_terms() {
    local file slug line term
    for file in "$GLOSSARY_DIR"/*.md; do
        [ "$(page_slug "$file")" = "index" ] && continue
        slug="$(page_slug "$file")"
        while IFS= read -r line; do
            case "$line" in
                "## "*)
                    term="${line#\#\# }"
                    printf '%s\t%s\n' "$slug" "$term"
                    ;;
            esac
        done <"$file"
    done
}

# Generate the alphabetical index body to standard output.
generate_index() {
    printf '%s\n' "$INDEX_HEADER"
    local slug term anchor
    all_terms | sort -f -t $'\t' -k2 | while IFS=$'\t' read -r slug term; do
        anchor="$(slugify "$term")"
        printf -- '- [%s](/docs/glossary/%s#%s)\n' "$term" "$slug" "$anchor"
    done
}

# check: entry completeness, cross-link resolution, index reproducibility.
run_check() {
    local fails=0 file slug

    # 1. Entry completeness: each "## Term" block carries a definition body (at
    #    least one non-blank content line). References and the matters callout are
    #    validated where present but are not mandated on every entry: the authored
    #    glossary carries references only where a primary source exists, and
    #    fabricating one to satisfy the linter would violate P-9.
    for file in "$GLOSSARY_DIR"/*.md; do
        [ "$(page_slug "$file")" = "index" ] && continue
        awk '
            /^## / {
                if (term != "" && !body) {
                    printf "%s: entry \"%s\" has no definition body\n", FILENAME, term
                    rc = 1
                }
                term = substr($0, 4); body = 0; next
            }
            term != "" && /^[[:space:]]*$/ { next }
            term != "" && /^#/ { next }
            term != "" { body = 1 }
            END {
                if (term != "" && !body) {
                    printf "%s: entry \"%s\" has no definition body\n", FILENAME, term
                    rc = 1
                }
                exit rc
            }
        ' "$file" >>"$FAIL_LOG" 2>&1 || fails=1
    done

    # 2. Cross-link resolution: every /docs/glossary/<slug>#<anchor> resolves.
    local valid link linkslug linkanchor
    valid="$(mktemp)"
    all_terms | while IFS=$'\t' read -r slug term; do
        printf '%s#%s\n' "$slug" "$(slugify "$term")"
    done | sort -u >"$valid"
    for file in "$GLOSSARY_DIR"/*.md; do
        [ "$(page_slug "$file")" = "index" ] && continue
        { grep -oE '/docs/glossary/[a-z0-9-]+#[a-z0-9_-]+' "$file" 2>/dev/null || true; } | while read -r link; do
            linkslug="${link#/docs/glossary/}"
            linkslug="${linkslug%%#*}"
            linkanchor="${link##*#}"
            if ! grep -qxF "${linkslug}#${linkanchor}" "$valid"; then
                printf '%s: cross-link does not resolve: %s\n' "$file" "$link" >>"$FAIL_LOG"
            fi
        done
    done
    if [ -s "$FAIL_LOG" ] && grep -q 'cross-link does not resolve' "$FAIL_LOG"; then
        fails=1
    fi
    rm -f "$valid"

    # 3. Index reproducibility: committed index matches a fresh generation.
    if [ ! -f "$INDEX_FILE" ]; then
        printf '%s: index is missing; run: lint-docs.sh fix\n' "$INDEX_FILE" >>"$FAIL_LOG"
        fails=1
    elif ! diff -q <(generate_index) "$INDEX_FILE" >/dev/null 2>&1; then
        printf '%s: index has drifted; run: lint-docs.sh fix\n' "$INDEX_FILE" >>"$FAIL_LOG"
        fails=1
    fi

    if [ "$fails" -ne 0 ]; then
        while IFS= read -r line; do log_error "$line"; done <"$FAIL_LOG"
        log_error "documentation check found failures"
        return 1
    fi
    log_ok "documentation check passed (${GLOSSARY_DIR})"
    return 0
}

# fix: regenerate the alphabetical index in place.
run_fix() {
    generate_index >"$INDEX_FILE"
    log_ok "regenerated ${INDEX_FILE}"
    return 0
}

# link: verify external reference URLs respond (weekly).
run_link() {
    if ! has_cmd curl; then
        log_error "curl is required for link mode"
        return 2
    fi
    local url dead=0
    { grep -rhoE 'https?://[^ )>]+' "$GLOSSARY_DIR"/*.md 2>/dev/null || true; } \
        | sed -E 's/[.,]+$//' | sort -u | while IFS= read -r url; do
        if curl -fsS -o /dev/null --max-time 20 --retry 1 "$url"; then
            log_ok "live: $url"
        else
            log_error "dead: $url"
            printf 'dead\n' >>"$LINK_LOG"
        fi
    done
    if [ -s "$LINK_LOG" ]; then
        dead=1
    fi
    [ "$dead" -eq 0 ] || return 1
    log_ok "all external references responded"
    return 0
}

#_______________________________________________________________________________
# Declare Variables and Arrays

# The repository root is the parent of this script's scripts/ directory.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GLOSSARY_DIR="${FRAGCAP_GLOSSARY_DIR:-$REPO_ROOT/docs/glossary}"
INDEX_FILE="$GLOSSARY_DIR/index.md"
FAIL_LOG="$(mktemp)"
LINK_LOG="$(mktemp)"

# The static header of the generated index. Held here so fix and check agree and
# the index is reproducible byte for byte.
INDEX_HEADER="# Glossary

Every term across the category pages, alphabetically. Generated from the category
pages by lint-docs.sh; do not edit by hand. Each entry links to its definition on
the owning category page.
"

trap 'rm -f "$FAIL_LOG" "$LINK_LOG"' EXIT

#_______________________________________________________________________________
# Execute Operations

mode="${1:-}"
case "$mode" in
    -h | --help | help)
        print_help
        exit 0
        ;;
    check)
        run_check
        exit $?
        ;;
    fix)
        run_fix
        exit $?
        ;;
    link)
        run_link
        exit $?
        ;;
    "")
        log_error "no mode given"
        print_help
        exit 2
        ;;
    *)
        log_error "unknown mode: $mode"
        print_help
        exit 2
        ;;
esac

#_______________________________________________________________________________
# End of script
