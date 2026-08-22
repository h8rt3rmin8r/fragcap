#!/usr/bin/env bash
#
# test-script-compliance.sh
#
# Deterministic compliance checker for shruggie-bash scripts. Verifies the
# language-agnostic house rules against a target .sh file, then runs ShellCheck
# when it is installed. Checks:
#
#   - Line 1 is exactly '#!/usr/bin/env bash'
#   - UTF-8 with no byte-order mark (no leading EF BB BF)
#   - LF line endings (no CR bytes anywhere)
#   - No trailing whitespace on any line
#   - Exactly one trailing newline at end of file
#   - No emoji or pictographic characters anywhere in the file
#   - The four named section dividers present, in order, each a '#' followed by
#     exactly 79 underscores
#   - A man-page-style help comment block between the shebang and the first
#     divider (at least a '# NAME' line)
#   - ShellCheck reports no findings (skipped with a warning if not installed)
#
# Exit codes: 0 every check passed, 1 at least one check failed, 2 the target
# file could not be read.
#
# Usage:
#   ./test-script-compliance.sh <path-to-script.sh>
#   ./test-script-compliance.sh -q <path>     # only failures and the summary
#   ./test-script-compliance.sh -h            # this help

set -u

QUIET=0
TARGET=""

print_help() {
    sed -n '2,29p' "$0" | sed 's/^# \{0,1\}//'
}

while [ $# -gt 0 ]; do
    case "$1" in
        -h|--help) print_help; exit 0 ;;
        -q|--quiet) QUIET=1; shift ;;
        -*) echo "FAIL: unknown option: $1" >&2; exit 2 ;;
        *) TARGET="$1"; shift ;;
    esac
done

if [ -z "${TARGET}" ]; then
    echo "FAIL: no target file given. Usage: $0 <path-to-script.sh>" >&2
    exit 2
fi

if [ ! -r "${TARGET}" ]; then
    echo "FAIL: target file not found or unreadable: ${TARGET}" >&2
    exit 2
fi

FAILURES=0

result() {
    # result <pass:0|1> <message>
    local pass="$1"; shift
    local msg="$*"
    if [ "${pass}" -eq 1 ]; then
        if [ "${QUIET}" -eq 1 ]; then return; fi
        printf 'OK:   %s\n' "${msg}"
    else
        printf 'FAIL: %s\n' "${msg}"
        FAILURES=$((FAILURES + 1))
    fi
}

# has_emoji <file> -> 0 emoji found, 1 none, 2 could not check.
has_emoji() {
    local f="$1"
    if command -v python3 >/dev/null 2>&1; then
        python3 - "$f" <<'PY'
import sys
d = open(sys.argv[1], 'rb').read().decode('utf-8', 'replace')
def bad(c):
    o = ord(c)
    return (0x2600 <= o <= 0x27BF or 0x2B00 <= o <= 0x2BFF or
            0xFE00 <= o <= 0xFE0F or o == 0x200D or o >= 0x1F000)
sys.exit(0 if any(bad(c) for c in d) else 1)
PY
        return $?
    elif command -v perl >/dev/null 2>&1; then
        perl -CSD -e 'local $/; my $d=<>; exit(($d =~ /[\x{2600}-\x{27BF}\x{2B00}-\x{2BFF}\x{FE00}-\x{FE0F}\x{200D}\x{1F000}-\x{10FFFF}]/) ? 0 : 1)' "$f"
        return $?
    fi
    return 2
}

# Line 1 shebang
first_line=$(head -n 1 "${TARGET}")
if [ "${first_line}" = "#!/usr/bin/env bash" ]; then
    result 1 "Shebang is '#!/usr/bin/env bash'"
else
    result 0 "Shebang is '#!/usr/bin/env bash' (found: ${first_line})"
fi

# UTF-8 with no BOM
bom=$(head -c 3 "${TARGET}" | od -An -tx1 | tr -d ' \n')
if [ "${bom}" = "efbbbf" ]; then
    result 0 "UTF-8 with no byte-order mark"
else
    result 1 "UTF-8 with no byte-order mark"
fi

# LF line endings (no CR bytes)
cr_count=$(LC_ALL=C tr -dc '\r' < "${TARGET}" | wc -c | tr -d ' ')
if [ "${cr_count}" -ne 0 ]; then
    result 0 "LF line endings (found ${cr_count} CR byte(s))"
else
    result 1 "LF line endings (no CR bytes)"
fi

# No trailing whitespace
tw=$(awk 'BEGIN { t = sprintf("\t") } { if ($0 ~ ("[ " t "]+$")) c++ } END { print c+0 }' "${TARGET}")
if [ "${tw}" -ne 0 ]; then
    result 0 "No trailing whitespace (${tw} offending line(s))"
else
    result 1 "No trailing whitespace (0 offending line(s))"
fi

# Exactly one trailing newline
last2=$(tail -c 2 "${TARGET}" | od -An -tx1 | tr -d ' \n')
last1=$(tail -c 1 "${TARGET}" | od -An -tx1 | tr -d ' \n')
if [ "${last1}" = "0a" ] && [ "${last2}" != "0a0a" ]; then
    result 1 "Exactly one trailing newline at end of file"
else
    result 0 "Exactly one trailing newline at end of file"
fi

# No emoji or pictographs
has_emoji "${TARGET}"
emoji_rc=$?
if [ "${emoji_rc}" -eq 0 ]; then
    result 0 "No emoji or pictographic characters"
elif [ "${emoji_rc}" -eq 1 ]; then
    result 1 "No emoji or pictographic characters"
else
    printf 'WARN: emoji check skipped (need python3 or perl)\n'
fi

# Four section dividers, each '#' + 79 underscores
dividers=$(grep -cE '^#_{79}$' "${TARGET}")
if [ "${dividers}" -eq 4 ]; then
    result 1 "Four 80-column section dividers present (found ${dividers})"
else
    result 0 "Four 80-column section dividers present (found ${dividers})"
fi

# The four named headings present, in order
order_ok=$(awk '
    BEGIN { idx = 0; split("# Declare Functions\n# Declare Variables and Arrays\n# Execute Operations\n# End of script", want, "\n") }
    {
        line = $0
        sub(/^[ \t]+/, "", line)
        sub(/[ \t]+$/, "", line)
        if (idx < 4 && line == want[idx+1]) { idx++ }
    }
    END { print (idx == 4) ? "1" : "0" }
' "${TARGET}")
if [ "${order_ok}" = "1" ]; then
    result 1 "Named section headings present in canonical order"
else
    result 0 "Named section headings present in canonical order"
fi

# A help comment block with a NAME section before the first divider
help_ok=$(awk '
    BEGIN { seen_shebang = 0; found = 0 }
    /^#_{79}$/ { exit }
    {
        if ($0 ~ /^#!/) { seen_shebang = 1; next }
        if (seen_shebang && $0 ~ /^#[[:space:]]+NAME[[:space:]]*$/) { found = 1 }
    }
    END { print found }
' "${TARGET}")
if [ "${help_ok}" = "1" ]; then
    result 1 "Man-page help block with a NAME section precedes the first divider"
else
    result 0 "Man-page help block with a NAME section precedes the first divider"
fi

# ShellCheck
if command -v shellcheck >/dev/null 2>&1; then
    if shellcheck -x "${TARGET}" >/dev/null 2>&1; then
        result 1 "ShellCheck reports no findings"
    else
        result 0 "ShellCheck reports findings (run: shellcheck ${TARGET})"
    fi
else
    printf 'WARN: ShellCheck not installed; static analysis skipped\n'
fi

echo ""
if [ "${FAILURES}" -eq 0 ]; then
    printf 'OK:   %s is compliant.\n' "${TARGET}"
else
    printf 'FAIL: %s has %s compliance issue(s).\n' "${TARGET}" "${FAILURES}"
    exit 1
fi

exit 0
