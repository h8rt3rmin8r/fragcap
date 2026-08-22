# Shruggie Bash Fixtures

Copy-paste-exact building blocks for standard-shaped Bash scripts. Copy these
verbatim rather than retyping them; adjust only with reason. Everything here is
four-space indented so it drops straight under a section divider.

## Safety preamble

Abort-on-failure (default):

```bash
set -euo pipefail
IFS=$'\n\t'
```

Graceful-degradation (probes, collectors, inventory tools):

```bash
set -o pipefail
# pipefail only: this tool degrades gracefully past missing optional tools
```

Optional hardening, add when the logic depends on it:

```bash
shopt -s inherit_errexit   # errexit reaches inside $(...)
shopt -s nullglob          # an unmatched glob expands to nothing
```

## Self-parsing print_help

Reads the script's own header comment block and prints it back out, so the help
text lives in exactly one place. Requires `SCRIPT_PATH` to point at the running
script (see the variables fixture).

```bash
    print_help() {
        # print_help: emit the comment help block between the shebang and the
        # first non-help line. Help lines are "# " + content (two chars stripped)
        # or a bare "#" (emitted blank). The first divider or any "#"+non-space
        # continuation ends the block.
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

    print_version() {
        # print_version: emit script name and version.
        printf '%s v%s\n' "${SCRIPT_NAME}" "${SCRIPT_VERSION}"
    }
```

## Script identity variables

Goes in `Declare Variables and Arrays`. `BASH_SOURCE[0]` is correct whether the
script is run or sourced; `$0` is not.

```bash
    SCRIPT_PATH="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/$(basename -- "${BASH_SOURCE[0]}")"
    SCRIPT_NAME="$(basename -- "${BASH_SOURCE[0]}")"
    SCRIPT_VERSION="1.0.0"
```

## Command and privilege checks

```bash
    has_cmd() {
        # has_cmd: return 0 if the given command is on PATH.
        command -v "$1" >/dev/null 2>&1
    }

    is_root() {
        # is_root: return 0 if the effective UID is 0.
        [[ ${EUID:-$(id -u)} -eq 0 ]]
    }

    require_cmd() {
        # require_cmd: exit 2 with an install hint if a command is missing.
        local cmd="$1"
        local pkg="${2:-$1}"
        if ! has_cmd "${cmd}"; then
            log_error "missing required command: ${cmd}"
            log_error "install with: sudo apt-get install -y ${pkg}"
            exit 2
        fi
    }
```

## Color setup and logging

Declare the color and suppression state in `Declare Variables and Arrays`:

```bash
    OPT_QUIET=0
    OPT_SILENT=0
    OPT_NO_COLOR=0
    C_RED=$'\033[31m'
    C_GREEN=$'\033[32m'
    C_YELLOW=$'\033[33m'
    C_BLUE=$'\033[34m'
    C_BOLD=$'\033[1m'
    C_DIM=$'\033[2m'
    C_RESET=$'\033[0m'
```

The logging fixtures go in `Declare Functions`:

```bash
    disable_colors() {
        # disable_colors: blank every color escape.
        C_RED='' ; C_GREEN='' ; C_YELLOW='' ; C_BLUE=''
        C_BOLD='' ; C_DIM='' ; C_RESET=''
    }

    maybe_disable_colors() {
        # maybe_disable_colors: blank colors when asked, when NO_COLOR is set,
        # or when stdout is not a terminal. Call once before any logging.
        if [[ ${OPT_NO_COLOR} -eq 1 || -n "${NO_COLOR:-}" || ! -t 1 ]]; then
            disable_colors
        fi
    }

    log_info() {
        # log_info: info to stderr; suppressed under --quiet/--silent.
        [[ ${OPT_QUIET} -eq 1 || ${OPT_SILENT} -eq 1 ]] && return 0
        printf '%s[INFO]%s %s\n' "${C_BLUE}" "${C_RESET}" "$*" >&2
    }

    log_success() {
        # log_success: success to stderr; suppressed under --quiet/--silent.
        [[ ${OPT_QUIET} -eq 1 || ${OPT_SILENT} -eq 1 ]] && return 0
        printf '%s[ OK ]%s %s\n' "${C_GREEN}" "${C_RESET}" "$*" >&2
    }

    log_warn() {
        # log_warn: warning to stderr; suppressed only under --silent.
        [[ ${OPT_SILENT} -eq 1 ]] && return 0
        printf '%s[WARN]%s %s\n' "${C_YELLOW}" "${C_RESET}" "$*" >&2
    }

    log_error() {
        # log_error: error to stderr; always emitted.
        printf '%s[ERROR]%s %s\n' "${C_RED}" "${C_RESET}" "$*" >&2
    }

    debug_log() {
        # debug_log: per-step trace on fd 3, gated on SCRIPT_DEBUG. Survives the
        # 2>/dev/null redirects collectors apply. No-op when SCRIPT_DEBUG unset.
        [[ -z "${SCRIPT_DEBUG:-}" ]] && return 0
        printf '%s[DBG ]%s %s\n' "${C_DIM}" "${C_RESET}" "$*" >&3
    }

    setup_debug_fd() {
        # setup_debug_fd: open fd 3 for debug. Call once before collectors run.
        if [[ -n "${SCRIPT_DEBUG:-}" ]]; then
            exec 3>&2
        else
            exec 3>/dev/null
        fi
    }
```

## Hang-proof command execution

For probes that shell out to commands that can block (setuid wrappers, network
tools). Guarantees a return within `SAFE_RUN_TIMEOUT + SAFE_RUN_KILL_AFTER`
seconds. Declare `SAFE_RUN_TIMEOUT=10` and `SAFE_RUN_KILL_AFTER=5` as variables.

```bash
    safe_run() {
        # safe_run: run a command with hard termination guarantees; stderr merged
        # into stdout. SIGKILL follows SIGTERM if the command ignores it.
        if has_cmd timeout && has_cmd setsid; then
            setsid -w timeout --preserve-status \
                --kill-after="${SAFE_RUN_KILL_AFTER}" \
                "${SAFE_RUN_TIMEOUT}" "$@" </dev/null 2>&1 3>&-
        elif has_cmd timeout; then
            timeout --preserve-status \
                --kill-after="${SAFE_RUN_KILL_AFTER}" \
                "${SAFE_RUN_TIMEOUT}" "$@" </dev/null 2>&1 3>&-
        else
            "$@" </dev/null 2>&1 3>&-
        fi
    }

    safe_capture() {
        # safe_capture: stdout-only sibling of safe_run; stderr discarded. Use
        # when feeding output into a variable.
        if has_cmd timeout && has_cmd setsid; then
            setsid -w timeout --preserve-status \
                --kill-after="${SAFE_RUN_KILL_AFTER}" \
                "${SAFE_RUN_TIMEOUT}" "$@" </dev/null 2>/dev/null 3>&-
        elif has_cmd timeout; then
            timeout --preserve-status \
                --kill-after="${SAFE_RUN_KILL_AFTER}" \
                "${SAFE_RUN_TIMEOUT}" "$@" </dev/null 2>/dev/null 3>&-
        else
            "$@" </dev/null 2>/dev/null 3>&-
        fi
    }
```

## trap cleanup

```bash
    cleanup() {
        # cleanup: idempotent teardown; safe to run more than once.
        [[ -n "${RUN_TMPDIR:-}" && -d "${RUN_TMPDIR:-}" ]] && rm -rf -- "${RUN_TMPDIR}"
    }
    trap cleanup EXIT INT TERM
    RUN_TMPDIR="$(mktemp -d)"
```

## Argument-parsing skeleton

```bash
    parse_args() {
        # parse_args: consume the argument vector, set OPT_* globals, collect
        # positionals. Exit 1 on malformed input.
        local -a positional=()
        while [[ $# -gt 0 ]]; do
            case "$1" in
                -h|--help)      print_help; exit 0 ;;
                -V|--version)   print_version; exit 0 ;;
                -q|--quiet)     OPT_QUIET=1; shift ;;
                --silent)       OPT_SILENT=1; shift ;;
                --no-color)     OPT_NO_COLOR=1; shift ;;
                -o|--out)       OPT_OUT="${2:-}"; [[ -z "${OPT_OUT}" ]] && { log_error "-o requires a value"; exit 1; }; shift 2 ;;
                -o=*|--out=*)   OPT_OUT="${1#*=}"; shift ;;
                --)             shift; positional+=("$@"); break ;;
                -*)             log_error "Unknown option: $1"; log_error "Try '--help'."; exit 1 ;;
                *)              positional+=("$1"); shift ;;
            esac
        done
        ARGV=("${positional[@]}")
    }
```

## Safe file iteration

```bash
    # Glob (guard empties with nullglob):
    shopt -s nullglob
    for f in ./*.log; do
        process -- "${f}"
    done
    shopt -u nullglob

    # Arbitrary names, including newlines, via find:
    while IFS= read -r -d '' f; do
        process -- "${f}"
    done < <(find . -type f -name '*.log' -print0)

    # Into an array:
    mapfile -d '' -t files < <(find . -type f -print0)
```
