#!/usr/bin/env bash
# probe-health.sh - Quick host health probe with OK/FAIL checks
#
# NAME
#     probe-health.sh - Run a set of host health checks and report OK/FAIL
#
# SYNOPSIS
#     probe-health.sh [--disk-pct N] [--no-color] [-q]
#     probe-health.sh --help
#
# DESCRIPTION
#     Runs a handful of environmental health checks and prints one structured
#     OK: or FAIL: line per check. Every optional tool is guarded with has_cmd
#     so the probe degrades gracefully on minimal hosts rather than aborting.
#     Color is auto-disabled when stdout is not a terminal, when --no-color is
#     given, or when NO_COLOR is set.
#
# OPTIONS
#     --disk-pct N   Fail the disk check when root usage is at or above N percent
#                    (default 90).
#     --no-color     Disable ANSI color.
#     -q, --quiet    Suppress the summary line; per-check OK/FAIL still prints.
#     -h, --help     Show this help and exit.
#
# EXAMPLES
#     Run all checks:
#         ./probe-health.sh
#
#     Stricter disk threshold, no color, for a log:
#         ./probe-health.sh --disk-pct 80 --no-color >> /var/log/health.txt
#
# EXIT CODES
#     0  All checks passed
#     1  One or more checks failed
#
# AUTHOR
#     h8rt3rmin8r for ShruggieTech (Shruggie LLC)
#

set -o pipefail
# pipefail only: this probe degrades gracefully past missing optional tools

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

    has_cmd() {
        command -v "$1" >/dev/null 2>&1
    }

    maybe_disable_colors() {
        if [[ ${OPT_NO_COLOR} -eq 1 || -n "${NO_COLOR:-}" || ! -t 1 ]]; then
            C_GREEN='' ; C_RED='' ; C_RESET=''
        fi
    }

    ok() {
        # ok: record and print a passing check.
        printf '%sOK:%s   %s\n' "${C_GREEN}" "${C_RESET}" "$*"
    }

    fail() {
        # fail: record and print a failing check.
        FAILURES=$((FAILURES + 1))
        printf '%sFAIL:%s %s\n' "${C_RED}" "${C_RESET}" "$*"
    }

    check_disk() {
        # check_disk: root filesystem usage against the threshold.
        if ! has_cmd df; then
            fail "disk: df not available"
            return 0
        fi
        local pct
        pct="$(df -P / | awk 'NR==2 {gsub(/%/,"",$5); print $5}')"
        if [[ -z "${pct}" ]]; then
            fail "disk: could not read root usage"
        elif [[ "${pct}" -ge "${OPT_DISK_PCT}" ]]; then
            fail "disk: root at ${pct}% (threshold ${OPT_DISK_PCT}%)"
        else
            ok "disk: root at ${pct}% (threshold ${OPT_DISK_PCT}%)"
        fi
    }

    check_memory() {
        # check_memory: available memory is above a small floor.
        if [[ ! -r /proc/meminfo ]]; then
            ok "memory: /proc/meminfo absent, skipped"
            return 0
        fi
        local avail_kb
        avail_kb="$(awk '/^MemAvailable:/ {print $2}' /proc/meminfo)"
        if [[ -z "${avail_kb}" ]]; then
            fail "memory: MemAvailable not reported"
        elif [[ "${avail_kb}" -lt 102400 ]]; then
            fail "memory: only ${avail_kb} kB available"
        else
            ok "memory: ${avail_kb} kB available"
        fi
    }

    check_load() {
        # check_load: 1-minute load average is finite and readable.
        if [[ ! -r /proc/loadavg ]]; then
            ok "load: /proc/loadavg absent, skipped"
            return 0
        fi
        local one
        read -r one _ < /proc/loadavg
        ok "load: 1-minute average ${one}"
    }

    parse_args() {
        while [[ $# -gt 0 ]]; do
            case "$1" in
                -h|--help)      print_help; exit 0 ;;
                --disk-pct)     OPT_DISK_PCT="${2:-}"; shift 2 ;;
                --disk-pct=*)   OPT_DISK_PCT="${1#*=}"; shift ;;
                --no-color)     OPT_NO_COLOR=1; shift ;;
                -q|--quiet)     OPT_QUIET=1; shift ;;
                --)             shift; break ;;
                -*)             printf '[ERROR] Unknown option: %s\n' "$1" >&2; exit 1 ;;
                *)              printf '[ERROR] Unexpected argument: %s\n' "$1" >&2; exit 1 ;;
            esac
        done
    }

#_______________________________________________________________________________
# Declare Variables and Arrays

    SCRIPT_PATH="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/$(basename -- "${BASH_SOURCE[0]}")"

    OPT_DISK_PCT=90
    OPT_NO_COLOR=0
    OPT_QUIET=0
    FAILURES=0

    C_GREEN=$'\033[32m'
    C_RED=$'\033[31m'
    C_RESET=$'\033[0m'

#_______________________________________________________________________________
# Execute Operations

    parse_args "$@"
    maybe_disable_colors

    if ! [[ "${OPT_DISK_PCT}" =~ ^[0-9]+$ ]]; then
        printf '[ERROR] --disk-pct must be an integer: %s\n' "${OPT_DISK_PCT}" >&2
        exit 1
    fi

    check_disk
    check_memory
    check_load

    if [[ ${OPT_QUIET} -ne 1 ]]; then
        if [[ ${FAILURES} -eq 0 ]]; then
            printf 'all checks passed\n'
        else
            printf '%d check(s) failed\n' "${FAILURES}"
        fi
    fi

    [[ ${FAILURES} -eq 0 ]] || exit 1

#_______________________________________________________________________________
# End of script
