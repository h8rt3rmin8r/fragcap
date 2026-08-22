#!/usr/bin/env bash
# new-secret.sh - Generate a cryptographically secure random secret
#
# NAME
#     new-secret.sh - Generate a cryptographically secure random secret string
#
# SYNOPSIS
#     new-secret.sh [-l LENGTH] [-f FORMAT] [-n COUNT] [-q]
#     new-secret.sh --help
#
# DESCRIPTION
#     Generates random bytes from the operating system CSPRNG (/dev/urandom) and
#     encodes them in the requested format. Suitable for cookie secrets, JWT
#     signing keys, API tokens, and similar credentials. The bytes never come
#     from $RANDOM, which is not cryptographically secure.
#
#     Defaults to 32 bytes encoded as standard base64. With --quiet and a single
#     secret, the value is emitted with no trailing newline so it pipes cleanly
#     into another command.
#
# OPTIONS
#     -l, --length N   Number of random bytes before encoding (default 32). For
#                      the alphanumeric format this is the output character
#                      count.
#     -f, --format F   base64 (default), base64url, hex, or alphanumeric.
#     -n, --count N    Number of secrets to generate, one per line (default 1).
#     -q, --quiet      Emit only the payload; for a single secret, no trailing
#                      newline.
#     -h, --help       Show this help and exit.
#
# EXAMPLES
#     A single 32-byte base64 secret:
#         ./new-secret.sh
#
#     A 64-byte hex secret:
#         ./new-secret.sh --length 64 --format hex
#
#     Five 16-character alphanumeric secrets:
#         ./new-secret.sh --count 5 --format alphanumeric --length 16
#
#     Pipe a clean secret with no trailing newline:
#         ./new-secret.sh --quiet | pbcopy
#
# EXIT CODES
#     0  Success
#     1  Invalid argument
#     2  No usable random source
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

    log_error() {
        printf '[ERROR] %s\n' "$*" >&2
    }

    gen_secret() {
        # gen_secret: one secret of the requested byte length and format. Uses
        # only coreutils (head, od, base64, tr) so there is no dependency on xxd,
        # openssl, or similar. Raw bytes come from /dev/urandom.
        local n="$1"
        local fmt="$2"
        case "${fmt}" in
            hex)
                od -An -tx1 -N "${n}" /dev/urandom | tr -d ' \n'
                ;;
            base64)
                head -c "${n}" /dev/urandom | base64 | tr -d '\n'
                ;;
            base64url)
                head -c "${n}" /dev/urandom | base64 | tr -d '\n' | tr '+/' '-_' | tr -d '='
                ;;
            alphanumeric)
                # tr consumes an unbounded stream and head closes early, so the
                # pipeline can exit 141 (SIGPIPE). Tolerate it explicitly.
                LC_ALL=C tr -dc 'A-Za-z0-9' < /dev/urandom 2>/dev/null | head -c "${n}" || true
                ;;
        esac
    }

    parse_args() {
        while [[ $# -gt 0 ]]; do
            case "$1" in
                -h|--help)     print_help; exit 0 ;;
                -l|--length)   OPT_LENGTH="${2:-}"; shift 2 ;;
                -l=*|--length=*) OPT_LENGTH="${1#*=}"; shift ;;
                -f|--format)   OPT_FORMAT="${2:-}"; shift 2 ;;
                -f=*|--format=*) OPT_FORMAT="${1#*=}"; shift ;;
                -n|--count)    OPT_COUNT="${2:-}"; shift 2 ;;
                -n=*|--count=*) OPT_COUNT="${1#*=}"; shift ;;
                -q|--quiet)    OPT_QUIET=1; shift ;;
                --)            shift; break ;;
                -*)            log_error "Unknown option: $1"; log_error "Try '--help'."; exit 1 ;;
                *)             log_error "Unexpected argument: $1"; exit 1 ;;
            esac
        done
    }

    validate_args() {
        if ! [[ "${OPT_LENGTH}" =~ ^[0-9]+$ ]] || [[ "${OPT_LENGTH}" -lt 1 ]]; then
            log_error "length must be a positive integer: ${OPT_LENGTH}"
            exit 1
        fi
        if ! [[ "${OPT_COUNT}" =~ ^[0-9]+$ ]] || [[ "${OPT_COUNT}" -lt 1 ]]; then
            log_error "count must be a positive integer: ${OPT_COUNT}"
            exit 1
        fi
        case "${OPT_FORMAT}" in
            base64|base64url|hex|alphanumeric) ;;
            *) log_error "unknown format: ${OPT_FORMAT}"; exit 1 ;;
        esac
    }

#_______________________________________________________________________________
# Declare Variables and Arrays

    SCRIPT_PATH="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/$(basename -- "${BASH_SOURCE[0]}")"

    OPT_LENGTH=32
    OPT_FORMAT="base64"
    OPT_COUNT=1
    OPT_QUIET=0

#_______________________________________________________________________________
# Execute Operations

    parse_args "$@"
    validate_args

    if [[ ! -r /dev/urandom ]]; then
        log_error "/dev/urandom is not readable; no usable random source"
        exit 2
    fi

    if [[ ${OPT_QUIET} -eq 1 && ${OPT_COUNT} -eq 1 ]]; then
        gen_secret "${OPT_LENGTH}" "${OPT_FORMAT}"
    else
        i=0
        while [[ ${i} -lt ${OPT_COUNT} ]]; do
            gen_secret "${OPT_LENGTH}" "${OPT_FORMAT}"
            printf '\n'
            i=$((i + 1))
        done
    fi

#_______________________________________________________________________________
# End of script
