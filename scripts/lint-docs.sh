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
#     check   Validate the glossary. Four checks: entry completeness (every entry
#             carries a prose blurb or detail, not merely metadata markers, and a
#             References section or matters callout, where present, is not empty);
#             cross-link resolution (every sibling <category>.md#<anchor> link
#             resolves to an existing term); the glossary-reference check (every
#             glossary reference in the canonical documents names a defined term,
#             enforcing the Undefined Term Rule of section 4.2); and index
#             reproducibility (the committed index.md matches a fresh generation).
#             References are not mandated on every entry: much of the glossary is
#             fragcap's own vocabulary, for which no primary source exists, and
#             fabricating one would violate P-9. Reports every failure and exits 1
#             if any failed. This is the mode continuous integration runs, so P-6
#             is enforced on every push.
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
#     Scope note: the Undefined Term Rule is enforced for glossary references
#     (a Markdown link into the glossary) in the canonical documents and within
#     the glossary itself. A bare prose word that happens to be a term but is not
#     referenced as one is not scanned: no sound rule distinguishes it from
#     ordinary English, and many glossary terms are fragcap's own internal
#     vocabulary. The enforced mechanism is the one the documents actually use to
#     mark a term.
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

# shellcheck disable=SC2329  # standard fixture: reserved for future verbose modes
log_info() { log_msg INFO "$@"; }
log_ok() { log_msg OK "$@"; }
log_error() { log_msg FAIL "$@"; }

# Run a command, echoing it first, so a failing step is legible.
# shellcheck disable=SC2329  # standard fixture: kept for Bash compliance parity
safe_run() {
    log_info "run: $*"
    "$@"
}

# The category slug for a page path (its basename without extension).
page_slug() {
    local path="$1"
    path="${path##*/}"
    printf '%s' "${path%.md}"
}

# Emit "slug<TAB>term<TAB>anchor" for every term heading across the category
# pages. The anchor is the GitHub-style heading slug, computed inside awk in a
# single pass so the check does not spawn a process per term (there are 125).
all_terms() {
    local file slug
    for file in "$GLOSSARY_DIR"/*.md; do
        slug="$(page_slug "$file")"
        [ "$slug" = "index" ] && continue
        awk -v slug="$slug" '
            function slugify(s,   r) {
                r = tolower(s)
                gsub(/[^a-z0-9_ -]/, "", r)
                gsub(/[ \t]+/, "-", r)
                return r
            }
            /^## / {
                term = substr($0, 4)
                printf "%s\t%s\t%s\n", slug, term, slugify(term)
            }
        ' "$file"
    done
}

# Generate the alphabetical index body to standard output. Links are
# repository-relative sibling paths (<category>.md#<anchor>) so they resolve on
# disk and on GitHub, per CONVENTIONS.md.
generate_index() {
    printf '%s\n' "$INDEX_HEADER"
    local slug term anchor
    all_terms | sort -f -t $'\t' -k2 | while IFS=$'\t' read -r slug term anchor; do
        printf -- '- [%s](%s.md#%s)\n' "$term" "$slug" "$anchor"
    done
}

# check: entry completeness, cross-link resolution, index reproducibility, and
# the glossary-reference (undefined-term) check across the canonical documents.
run_check() {
    local fails=0 file

    # 1. Entry completeness. Each "## Term" block must carry a definition (at
    #    least one prose line: the blurb or detail, not merely a metadata marker
    #    such as "Also known as" or a "See also" line). A References section or a
    #    matters callout, where present, must not be empty. References are not
    #    mandated on every entry: much of the glossary is fragcap's own internal
    #    vocabulary (for example "Sink thread") for which no primary source
    #    exists, and fabricating one to satisfy the linter would violate P-9.
    for file in "$GLOSSARY_DIR"/*.md; do
        [ "$(page_slug "$file")" = "index" ] && continue
        awk '
            function flush() {
                if (term == "") return
                if (!prose) {
                    printf "%s: entry \"%s\" has no definition blurb or detail\n", FILENAME, term
                    rc = 1
                }
                if (refs_open && !refs_body) {
                    printf "%s: entry \"%s\" has an empty References section\n", FILENAME, term
                    rc = 1
                }
                if (matters_open && !matters_body) {
                    printf "%s: entry \"%s\" has an empty matters callout\n", FILENAME, term
                    rc = 1
                }
            }
            /^## / {
                flush()
                term = substr($0, 4)
                prose = 0; refs_open = 0; refs_body = 0; matters_open = 0; matters_body = 0
                next
            }
            term == "" { next }
            /^\*\*References:\*\*/ { refs_open = 1; next }
            /^\{:[[:space:]]*\.matters/ { matters_open = 1; next }
            {
                if (refs_open && $0 ~ /^[[:space:]]*-/) refs_body = 1
                if (matters_open && $0 ~ /^>[[:space:]]*[^[:space:]]/) matters_body = 1
                if ($0 ~ /^[[:space:]]*$/) next
                if ($0 ~ /^#/) next
                if ($0 ~ /^\*\*/) next
                if ($0 ~ /^>/) next
                if ($0 ~ /^\{:/) next
                if ($0 ~ /^[[:space:]]*[-*+|]/) next
                prose = 1
            }
            END { flush(); exit rc }
        ' "$file" >>"$FAIL_LOG" 2>&1 || fails=1
    done

    # Build the set of valid glossary references: <slug>#<anchor> for every term.
    local valid slug term anchor
    valid="$(mktemp)"
    all_terms | while IFS=$'\t' read -r slug term anchor; do
        printf '%s#%s\n' "$slug" "$anchor"
    done | sort -u >"$valid"

    # 2. Cross-link resolution within the glossary: every sibling link
    #    <slug>.md#<anchor> resolves to an existing term on that page.
    local link linkslug linkanchor
    for file in "$GLOSSARY_DIR"/*.md; do
        [ "$(page_slug "$file")" = "index" ] && continue
        { grep -oE '\]\([a-z0-9-]+\.md#[a-z0-9_-]+\)' "$file" 2>/dev/null || true; } | while read -r link; do
            link="${link#](}"
            link="${link%)}"
            linkslug="${link%.md#*}"
            linkanchor="${link##*#}"
            if ! grep -qxF "${linkslug}#${linkanchor}" "$valid"; then
                printf '%s: cross-link does not resolve: %s\n' "$file" "$link" >>"$FAIL_LOG"
            fi
        done
    done

    # 3. Glossary-reference (undefined-term) check across the canonical documents.
    #    The Undefined Term Rule (section 4.2, constitution P-6) is enforced for
    #    the mechanism the docs use to mark a term: a link into the glossary. Any
    #    glossary reference in a canonical document that names a term with no
    #    entry fails here. A bare prose word is not distinguishable from ordinary
    #    English by any sound rule, so it is not scanned; a term is caught when it
    #    is referenced as a glossary term and the entry is missing.
    local doc ref refslug refanchor
    for doc in "${CANONICAL_DOCS[@]}"; do
        [ -f "$doc" ] || continue
        { grep -oE 'glossary/[a-z0-9-]+(\.md)?#[a-z0-9_-]+' "$doc" 2>/dev/null || true; } | while read -r ref; do
            ref="${ref#glossary/}"
            refslug="${ref%%#*}"
            refslug="${refslug%.md}"
            refanchor="${ref##*#}"
            if ! grep -qxF "${refslug}#${refanchor}" "$valid"; then
                printf '%s: glossary reference names an undefined term: %s\n' "$doc" "$ref" >>"$FAIL_LOG"
            fi
        done
    done

    if [ -s "$FAIL_LOG" ] && grep -qE 'does not resolve|undefined term' "$FAIL_LOG"; then
        fails=1
    fi
    rm -f "$valid"

    # 4. Index reproducibility: committed index matches a fresh generation.
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

# The canonical documents scanned for undefined glossary references (section
# 4.2): the top-level README and the top-level docs. The glossary pages are
# covered by the cross-link check; process artifacts under specs/ and
# changelog.d/ are not canonical documentation. Website copy joins this set with
# the site in sub-slice S18c-2.
CANONICAL_DOCS=("$REPO_ROOT/README.md" "$REPO_ROOT"/docs/*.md)

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
