#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# NAME
#     cut-release.sh - prepare a fragcap release branch (Phase A)
#
# SYNOPSIS
#     cut-release.sh [minor|patch|major|X.Y.Z]
#     cut-release.sh [LEVEL] --dry-run
#     cut-release.sh [LEVEL] --date YYYY-MM-DD
#     cut-release.sh -h | --help
#
# DESCRIPTION
#     Consolidates the local half of cutting a release into one command. It
#     bumps the workspace version, folds the changelog.d/ fragments into
#     CHANGELOG.md, corrects the two embedded-version assertions and the golden
#     corpus that the bump moves, and runs the full check set, leaving a green
#     release/X.Y.Z branch ready to open as a pull request.
#
#     It deliberately stops there. Two authorizations are required by the
#     constitution and are not automated: pushing the version tag (which fires
#     the release workflow) and approving the crates-io environment (which lets
#     the workflow publish). This script performs neither, and never tags,
#     pushes, or publishes. What it removes is the fiddly, error-prone local
#     dance, not the human gates.
#
#     The default level is minor, matching the versioning scheme in which the
#     first functional release is v0.2.0. A level of major, patch, or an
#     explicit X.Y.Z is accepted instead. The heavy lifting (changelog assembly,
#     release-notes derivation) lives in cargo xtask, not here, so this stays a
#     thin orchestrator over git, cargo release, and the task runner.
#
#     Modes:
#
#     (default)   Execute the preparation on a new release/X.Y.Z branch. Refuses
#                 unless the working tree is a clean main up to date with origin.
#
#     --dry-run   Print the plan and preview the assembled changelog, without
#                 creating a branch, bumping, or writing anything.
#
#     --date      Override the release date stamped into the changelog section
#                 (default: today, from date +%F).
#
# EXIT STATUS
#     0   ran and prepared the branch (or previewed, under --dry-run)
#     1   ran and a step failed
#     2   could not run (a required tool is absent, or preconditions unmet)
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
            WARN) color=$'\033[33m' ;;
            FAIL) color=$'\033[31m' ;;
            *) color="" ;;
        esac
    fi
    printf '%s%s%s %s\n' "$color" "$level" "$reset" "$*" >&2
}

log_info() { log_msg INFO "$@"; }
log_ok() { log_msg OK "$@"; }
log_warn() { log_msg WARN "$@"; }
log_error() { log_msg FAIL "$@"; }

# Run a command, echoing it first, so a failing step is legible.
safe_run() {
    log_info "run: $*"
    "$@"
}

# The current workspace version: the first version key in the root manifest,
# which sits under [workspace.package].
workspace_version() {
    grep -m1 '^version' "$REPO_ROOT/Cargo.toml" | sed -E 's/.*"([^"]+)".*/\1/'
}

# Compute the target version from the current one and a level. An explicit
# X.Y.Z is returned unchanged. Prints the target, or fails on a bad level.
semver_bump() {
    local old="$1" level="$2"
    case "$level" in
        [0-9]*.[0-9]*.[0-9]*)
            printf '%s' "$level"
            return 0
            ;;
    esac
    local major minor patch
    IFS='.' read -r major minor patch <<<"$old"
    case "$level" in
        major) printf '%s.0.0' "$((major + 1))" ;;
        minor) printf '%s.%s.0' "$major" "$((minor + 1))" ;;
        patch) printf '%s.%s.%s' "$major" "$minor" "$((patch + 1))" ;;
        *) return 1 ;;
    esac
}

# Verify the tools and repository state a real cut requires. Returns 2 when a
# tool is absent, 1 when a precondition is unmet.
preflight() {
    if ! has_cmd git || ! has_cmd cargo; then
        log_error "git and cargo are required"
        return 2
    fi
    if ! cargo release --version >/dev/null 2>&1; then
        log_error "cargo-release is required (install: cargo install cargo-release)"
        return 2
    fi
    if ! git -C "$REPO_ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        log_error "not inside a git work tree"
        return 2
    fi
    local branch
    branch="$(git -C "$REPO_ROOT" rev-parse --abbrev-ref HEAD)"
    if [ "$branch" != "main" ]; then
        log_error "must be on main to cut a release (on: $branch)"
        return 1
    fi
    if [ -n "$(git -C "$REPO_ROOT" status --porcelain)" ]; then
        log_error "working tree is not clean; commit or stash first"
        return 1
    fi
    safe_run git -C "$REPO_ROOT" fetch --quiet origin main
    local local_head remote_head
    local_head="$(git -C "$REPO_ROOT" rev-parse HEAD)"
    remote_head="$(git -C "$REPO_ROOT" rev-parse origin/main)"
    if [ "$local_head" != "$remote_head" ]; then
        log_error "local main is not in sync with origin/main; pull or push first"
        return 1
    fi
    return 0
}

# Replace the embedded version string fragcap/<old> with fragcap/<new> in the
# two source assertions the bump moves. The profile-format comment
# (fragcap:profile=...) uses a colon, not a slash, and is intentionally left
# alone: it versions the profile embedding, not the release.
fix_embedded_versions() {
    local old="$1" new="$2" file
    for file in \
        "$REPO_ROOT/crates/fragcap-sink/src/pcapng/mod.rs" \
        "$REPO_ROOT/crates/fragcap-sink/src/json/mod.rs"; do
        safe_run sed -i "s#fragcap/${old}#fragcap/${new}#g" "$file"
    done
}

# Print the sequence the operator runs after this script, so the two remaining
# authorizations are never a surprise.
print_next_steps() {
    local version="$1"
    cat >&2 <<EOF

Next steps (each is a deliberate, authorized act this script does not perform):

  1. Review the release/${version} branch, then open a pull request:
       git push -u origin release/${version}
       gh pr create --fill

  2. After the operator merges it, tag the release from main:
       git switch main && git pull
       git tag v${version} && git push origin v${version}

  3. The release workflow builds artifacts and creates the GitHub release, then
     the publish job waits on the crates-io environment. Approve it in GitHub to
     publish the eight crates.
EOF
}

#_______________________________________________________________________________
# Declare Variables and Arrays

# The repository root is the parent of this script's scripts/ directory.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Parsed from the command line in Execute Operations.
LEVEL="minor"
DRY_RUN=0
DATE_OVERRIDE=""

#_______________________________________________________________________________
# Execute Operations

while [ $# -gt 0 ]; do
    case "$1" in
        -h | --help | help)
            print_help
            exit 0
            ;;
        --dry-run)
            DRY_RUN=1
            ;;
        --date)
            shift
            DATE_OVERRIDE="${1:-}"
            if [ -z "$DATE_OVERRIDE" ]; then
                log_error "--date needs a value (YYYY-MM-DD)"
                exit 2
            fi
            ;;
        major | minor | patch)
            LEVEL="$1"
            ;;
        [0-9]*.[0-9]*.[0-9]*)
            LEVEL="$1"
            ;;
        *)
            log_error "unknown argument: $1"
            print_help
            exit 2
            ;;
    esac
    shift
done

old_version="$(workspace_version)"
if [ -z "$old_version" ]; then
    log_error "could not read the workspace version from Cargo.toml"
    exit 2
fi

target_version="$(semver_bump "$old_version" "$LEVEL")" || {
    log_error "invalid level or version: $LEVEL"
    exit 2
}

release_date="${DATE_OVERRIDE:-$(date +%F)}"

# Reject a malformed version or date here, before creating a branch or writing
# anything. semver_bump classifies an explicit version loosely (a value like
# 1.2.3junk reaches this point), and a --date typo would otherwise flow through
# to the changelog heading. cargo xtask changelog --release validates the same
# two fields again before it consumes fragments; this guard is the earlier of
# the two, so a bad input never mutates the repository.
if ! [[ "$target_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    log_error "invalid target version: $target_version (expected X.Y.Z)"
    exit 2
fi
if ! [[ "$release_date" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]]; then
    log_error "invalid date: $release_date (expected YYYY-MM-DD)"
    exit 2
fi

log_info "current version: $old_version"
log_info "target version:  $target_version"
log_info "release date:    $release_date"

if [ "$DRY_RUN" -eq 1 ]; then
    log_warn "dry run: nothing will be created, bumped, or written"
    log_info "would create branch: release/$target_version"
    log_info "would run: cargo release $target_version --workspace --execute --no-confirm"
    log_info "changelog preview (cargo xtask changelog --check):"
    ( cd "$REPO_ROOT" && cargo run --quiet --package xtask -- changelog --check ) || {
        log_error "changelog preview failed"
        exit 1
    }
    print_next_steps "$target_version"
    log_ok "dry run complete"
    exit 0
fi

preflight || exit $?

cd "$REPO_ROOT"

safe_run git switch -c "release/$target_version"

# Bump and commit the version. release.toml pins this to move the number only:
# no tag, no push, no publish.
safe_run cargo release "$target_version" --workspace --execute --no-confirm

new_version="$(workspace_version)"
if [ "$new_version" != "$target_version" ]; then
    log_error "version after bump is $new_version, expected $target_version"
    exit 1
fi

# The bump moved fragcap/<version>, embedded in two assertions and every golden.
fix_embedded_versions "$old_version" "$target_version"
log_info "regenerating the golden corpus for the new version"
FRAGCAP_UPDATE_GOLDENS=1 safe_run cargo test --workspace --quiet

# Assemble the changelog and fold everything into the single release commit.
safe_run cargo run --quiet --package xtask -- changelog --release "$target_version" "$release_date"
safe_run git add -A
safe_run git commit --amend --no-edit

log_info "running the full check set (cargo xtask ci)"
if ! cargo xtask ci; then
    log_error "cargo xtask ci failed on release/$target_version; the branch is left for inspection"
    exit 1
fi

log_ok "prepared release/$target_version"
print_next_steps "$target_version"

#_______________________________________________________________________________
# End of script
