#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# NAME
#     fragcap.sh - shell wrapper for the fragcap capture tool
#
# SYNOPSIS
#     fragcap.sh [-q|--quiet] [--silent] [--no-color] [--dry-run]
#                --profile <name> [-o <template>] [fragcap options...]
#     fragcap.sh -h | --help
#
# DESCRIPTION
#     A thin wrapper around the native fragcap binary for use from a Linux or
#     WSL2 shell (specification section 18.3). It handles the one environment
#     concern the binary cannot: the subsystem boundary. Under WSL2 it invokes
#     the native Windows binary through interop and translates the output path
#     from the Linux form to the Windows form, reporting the resulting file back
#     in Linux form. On a Linux host with no reachable Windows binary it reports
#     that capture is unavailable and exits 1.
#
#     The wrapper contains no capture logic and does not parse fragcap's
#     human-readable output. It reacts to the structured event stream fragcap
#     emits under --json (specification section 17.5), which this wrapper adds to
#     every invocation. Unrecognized options are passed through to fragcap
#     unchanged, so a new binary flag works without a wrapper change.
#
# OPTIONS
#     --profile <name>   The profile to capture with.
#     -o <template>      Output path template. Tokens {profile}, {date}, and
#                        {time} are expanded before capture.
#     --dry-run          Print the assembled fragcap invocation and exit without
#                        capturing.
#     -q, --quiet        Suppress informational output; keep warnings and errors.
#     --silent           Suppress warnings too; errors still emit.
#     --no-color         Disable colored output (NO_COLOR is also honored).
#     -h, --help         Print this help and exit.
#
# EXAMPLES
#     Capture a title with a templated output path:
#         fragcap.sh --profile eso -o "caps/{profile}-{date}.fcapng"
#
#     Preview the assembled invocation without capturing:
#         fragcap.sh --dry-run --profile eso -o "caps/{profile}.fcapng" --loopback
#
# EXIT CODES
#     0   Capture completed, or help or dry-run printed.
#     1   Capture is unavailable on this platform (no reachable Windows binary).
#     2   A precondition failed (a required argument is missing).
#
# AUTHOR
#     A ShruggieTech project.
#
set -euo pipefail
IFS=$'\n\t'
#_______________________________________________________________________________
# Declare Functions

    # Print the help block above by parsing this file's own header comments.
    print_help() {
        awk 'f && /^set /{exit} /^# NAME$/{f=1} f' "$0" | sed 's/^#\( \|$\)//'
    }

    # Whether a command is available on the PATH.
    has_cmd() {
        command -v "$1" >/dev/null 2>&1
    }

    # Colorized, level-tagged logging to standard error. Color is disabled when
    # --no-color or NO_COLOR is set or standard error is not a terminal.
    setup_color() {
        if [[ -n "${NO_COLOR:-}" ]] || [[ "${USE_COLOR}" == "0" ]] || [[ ! -t 2 ]]; then
            C_INFO=""; C_WARN=""; C_ERROR=""; C_RESET=""
        else
            C_INFO=$'\033[0;36m'; C_WARN=$'\033[0;33m'
            C_ERROR=$'\033[0;31m'; C_RESET=$'\033[0m'
        fi
    }

    log_info() {
        [[ "${VERBOSITY}" == "quiet" || "${VERBOSITY}" == "silent" ]] && return 0
        printf '%sINFO%s  %s\n' "${C_INFO}" "${C_RESET}" "$*" >&2
    }

    log_warn() {
        [[ "${VERBOSITY}" == "silent" ]] && return 0
        printf '%sWARN%s  %s\n' "${C_WARN}" "${C_RESET}" "$*" >&2
    }

    log_error() {
        printf '%sERROR%s %s\n' "${C_ERROR}" "${C_RESET}" "$*" >&2
    }

    # Expand the {profile}, {date}, and {time} tokens in an output template.
    expand_template() {
        local t="$1"
        t="${t//\{profile\}/${PROFILE}}"
        t="${t//\{date\}/$(date +%Y-%m-%d)}"
        t="${t//\{time\}/$(date +%H%M%S)}"
        printf '%s' "${t}"
    }

    # Join an argument list with single spaces, regardless of the current IFS
    # (which the safety preamble tightens to newline and tab).
    format_command() {
        local IFS=' '
        printf '%s' "$*"
    }

    # Whether this shell is running under WSL2.
    is_wsl() {
        [[ -n "${WSL_DISTRO_NAME:-}" ]] && return 0
        [[ -r /proc/version ]] && grep -qiE 'microsoft|wsl' /proc/version
    }

    # Run a command, reporting a non-zero exit through the logger rather than
    # letting the strict-mode shell abort with no explanation.
    safe_run() {
        if ! "$@"; then
            log_error "command failed: $(format_command "$@")"
            return 1
        fi
    }

    # Abort with a usage error (exit 2) when a required option value is missing
    # or is itself another option, so a swallowed value never reaches capture.
    need_value() {
        local flag="$1" value="$2"
        if [[ -z "${value}" || "${value}" == -* ]]; then
            log_error "option ${flag} needs a value"
            exit 2
        fi
    }

#_______________________________________________________________________________
# Declare Variables and Arrays

    VERBOSITY="normal"
    USE_COLOR="1"
    DRY_RUN="0"
    PROFILE=""
    OUT_TEMPLATE=""
    PASSTHROUGH=()
    # Initialized empty so the log fixtures are safe under `set -u` before
    # setup_color runs; setup_color fills in the real codes after parsing.
    C_INFO=""
    C_WARN=""
    C_ERROR=""
    C_RESET=""

#_______________________________________________________________________________
# Execute Operations

    # The help gate is the first action, before any work.
    case "${1:-}" in
        -h | --help)
            print_help
            exit 0
            ;;
    esac

    # Parse arguments. Both --flag value and --flag=value forms are handled; --
    # ends option processing; unknown options are passed through to fragcap.
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -h | --help)
                print_help
                exit 0
                ;;
            -q | --quiet)
                VERBOSITY="quiet"
                ;;
            --silent)
                VERBOSITY="silent"
                ;;
            --no-color)
                USE_COLOR="0"
                ;;
            --dry-run)
                DRY_RUN="1"
                ;;
            --profile)
                need_value "--profile" "${2:-}"
                PROFILE="$2"
                shift
                ;;
            --profile=*)
                PROFILE="${1#*=}"
                ;;
            -o)
                need_value "-o" "${2:-}"
                OUT_TEMPLATE="$2"
                shift
                ;;
            -o=*)
                OUT_TEMPLATE="${1#*=}"
                ;;
            --)
                shift
                while [[ $# -gt 0 ]]; do
                    PASSTHROUGH+=("$1")
                    shift
                done
                break
                ;;
            *)
                PASSTHROUGH+=("$1")
                ;;
        esac
        shift
    done

    setup_color

    if [[ -z "${PROFILE}" ]]; then
        log_error "a profile is required; pass --profile <name>"
        exit 2
    fi

    # A dry run prints the assembled invocation and exits, without resolving the
    # binary, preparing directories, translating paths, or capturing. It shows the
    # logical `fragcap run` invocation, expanded template and passed-through
    # options included.
    if [[ "${DRY_RUN}" == "1" ]]; then
        DRY=(fragcap run --profile "${PROFILE}")
        if [[ -n "${OUT_TEMPLATE}" ]]; then
            DRY+=(--out "$(expand_template "${OUT_TEMPLATE}")")
        fi
        DRY+=(--json)
        if [[ ${#PASSTHROUGH[@]} -gt 0 ]]; then
            DRY+=("${PASSTHROUGH[@]}")
        fi
        printf '%s\n' "$(format_command "${DRY[@]}")"
        exit 0
    fi

    # Resolve the binary. Prefer the native Windows binary (reached directly or
    # through WSL2 interop); fall back to fragcap on the PATH; if neither is
    # reachable, capture is unavailable on this platform.
    if has_cmd fragcap.exe; then
        BINARY="fragcap.exe"
    elif has_cmd fragcap; then
        BINARY="fragcap"
    else
        log_error "capture is unavailable on this platform: no fragcap binary is reachable"
        exit 1
    fi

    # Assemble the invocation, always headed by the resolved binary. Expand the
    # output template, prepare its directory, and translate the path to the
    # Windows form when the native binary is reached through WSL2 interop.
    COMMAND=("${BINARY}" run --profile "${PROFILE}")
    if [[ -n "${OUT_TEMPLATE}" ]]; then
        OUT_PATH="$(expand_template "${OUT_TEMPLATE}")"
        OUT_DIR="$(dirname -- "${OUT_PATH}")"
        if [[ -n "${OUT_DIR}" && ! -d "${OUT_DIR}" ]] && ! safe_run mkdir -p -- "${OUT_DIR}"; then
            exit 1
        fi
        OUT_ARG="${OUT_PATH}"
        if is_wsl && [[ "${BINARY}" == "fragcap.exe" ]] && has_cmd wslpath; then
            OUT_ARG="$(wslpath -w -- "${OUT_PATH}")"
            log_info "output ${OUT_PATH} (Windows: ${OUT_ARG})"
        fi
        COMMAND+=(--out "${OUT_ARG}")
    fi
    COMMAND+=(--json)
    if [[ ${#PASSTHROUGH[@]} -gt 0 ]]; then
        COMMAND+=("${PASSTHROUGH[@]}")
    fi

    # The native binary's own exit code is this wrapper's exit code.
    log_info "invoking: $(format_command "${COMMAND[@]}")"
    "${COMMAND[@]}"

#_______________________________________________________________________________
# End of script
