#!/usr/bin/env bash
# remove-stale-artifact.sh - Delete build artifacts older than a cutoff
#
# NAME
#     remove-stale-artifact.sh - Delete files older than N days from a directory
#
# SYNOPSIS
#     remove-stale-artifact.sh [--days N] [--force] [-q] DIRECTORY
#     remove-stale-artifact.sh --help
#
# DESCRIPTION
#     Finds regular files under DIRECTORY whose modification time is older than
#     the cutoff and deletes them. Defaults to a dry run: nothing is deleted
#     unless --force is given, so an operator always previews the blast radius
#     first.
#
#     File discovery uses find -print0 and a NUL-delimited read loop, so names
#     containing spaces, newlines, or leading dashes are handled correctly.
#
# OPTIONS
#     --days N     Age threshold in days (default 14). Files modified more than N
#                  days ago are candidates.
#     --force      Actually delete. Without it, the script only lists what it
#                  would remove.
#     -q, --quiet  Suppress informational output.
#     -h, --help   Show this help and exit.
#
# EXAMPLES
#     Preview what would be deleted from ./build:
#         ./remove-stale-artifact.sh ./build
#
#     Delete artifacts older than 30 days:
#         ./remove-stale-artifact.sh --days 30 --force ./build
#
# EXIT CODES
#     0  Success (including a clean dry run)
#     1  One or more deletions failed
#     2  DIRECTORY missing, not a directory, or not given
#
# AUTHOR
#     h8rt3rmin8r for ShruggieTech (Shruggie LLC)
#

set -euo pipefail
IFS=$'\n\t'

#_______________________________________________________________________________
# Declare Functions

    print_help() {
        local line
        local in_help=0
        while IFS= read -r line; do
            if [[ ${in_help} -eq 0 ]]; then
                [[ "${line}" == '#!'* ]] && in_help=1
                continue
            fi
            if [[ "${line}" == '#' ]]; then
                printf '\n'
            elif [[ "${line}" =~ ^#[[:space:]] ]]; then
                printf '%s\n' "${line:2}"
            else
                break
            fi
        done < "${SCRIPT_PATH}"
    }

    log_info() {
        [[ ${OPT_QUIET} -eq 1 ]] && return 0
        printf '[INFO] %s\n' "$*" >&2
    }

    log_warn() {
        printf '[WARN] %s\n' "$*" >&2
    }

    log_error() {
        printf '[ERROR] %s\n' "$*" >&2
    }

    cleanup() {
        # cleanup: idempotent teardown of the run's temp list.
        [[ -n "${RUN_LIST:-}" && -f "${RUN_LIST:-}" ]] && rm -f -- "${RUN_LIST}"
    }

    parse_args() {
        local -a positional=()
        while [[ $# -gt 0 ]]; do
            case "$1" in
                -h|--help)    print_help; exit 0 ;;
                --days)       OPT_DAYS="${2:-}"; shift 2 ;;
                --days=*)     OPT_DAYS="${1#*=}"; shift ;;
                --force)      OPT_FORCE=1; shift ;;
                -q|--quiet)   OPT_QUIET=1; shift ;;
                --)           shift; positional+=("$@"); break ;;
                -*)           log_error "Unknown option: $1"; log_error "Try '--help'."; exit 1 ;;
                *)            positional+=("$1"); shift ;;
            esac
        done
        if [[ ${#positional[@]} -ne 1 ]]; then
            log_error "exactly one DIRECTORY argument is required"
            exit 2
        fi
        OPT_DIR="${positional[0]}"
    }

#_______________________________________________________________________________
# Declare Variables and Arrays

    SCRIPT_PATH="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/$(basename -- "${BASH_SOURCE[0]}")"

    OPT_DAYS=14
    OPT_FORCE=0
    OPT_QUIET=0
    OPT_DIR=""
    RUN_LIST=""

    trap cleanup EXIT INT TERM

#_______________________________________________________________________________
# Execute Operations

    parse_args "$@"

    if ! [[ "${OPT_DAYS}" =~ ^[0-9]+$ ]]; then
        log_error "--days must be a non-negative integer: ${OPT_DAYS}"
        exit 2
    fi
    if [[ ! -d "${OPT_DIR}" ]]; then
        log_error "not a directory: ${OPT_DIR}"
        exit 2
    fi

    if [[ ${OPT_FORCE} -eq 1 ]]; then
        log_info "deleting files older than ${OPT_DAYS} days under ${OPT_DIR}"
    else
        log_info "DRY RUN: files older than ${OPT_DAYS} days under ${OPT_DIR} (pass --force to delete)"
    fi

    failures=0
    count=0
    while IFS= read -r -d '' f; do
        count=$((count + 1))
        if [[ ${OPT_FORCE} -eq 1 ]]; then
            if rm -f -- "${f}"; then
                log_info "removed: ${f}"
            else
                log_warn "failed to remove: ${f}"
                failures=$((failures + 1))
            fi
        else
            printf 'would remove: %s\n' "${f}"
        fi
    done < <(find "${OPT_DIR}" -type f -mtime "+${OPT_DAYS}" -print0)

    log_info "matched ${count} file(s)"

    if [[ ${failures} -gt 0 ]]; then
        log_error "${failures} deletion(s) failed"
        exit 1
    fi

#_______________________________________________________________________________
# End of script
