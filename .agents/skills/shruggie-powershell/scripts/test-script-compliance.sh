#!/usr/bin/env bash
#
# test-script-compliance.sh
#
# POSIX-shell twin of Test-ScriptCompliance.ps1. Runs the same deterministic,
# language-agnostic checks against a target .ps1 file so remote agents on
# non-Windows hosts can verify a script without pwsh. The checks are:
#
#   - UTF-8 with no byte-order mark (no leading EF BB BF)
#   - LF line endings (no CR bytes)
#   - No trailing whitespace on any line
#   - Exactly one trailing newline at end of file
#   - No emoji or pictographic characters anywhere in the file
#   - The four named section dividers present, in order, each a '#' followed
#     by exactly 79 underscores
#   - A comment-based help block ('<# ... #>') before the first [CmdletBinding
#
# Exit codes: 0 every check passed, 1 at least one check failed, 2 the target
# file could not be read.
#
# Requires GNU grep (grep -P). Most Linux runners have it.
#
# Usage:
#   ./test-script-compliance.sh <path-to-script.ps1>
#   ./test-script-compliance.sh -q <path>     # only failures and the summary
#   ./test-script-compliance.sh -h            # this help

set -u

QUIET=0
SILENT=0
TARGET=""

print_help() {
    sed -n '2,28p' "$0" | sed 's/^# \{0,1\}//'
}

while [ $# -gt 0 ]; do
    case "$1" in
        -h|--help) print_help; exit 0 ;;
        -q|--quiet) QUIET=1; shift ;;
        --silent) SILENT=1; shift ;;
        -*) echo "FAIL: unknown option: $1" >&2; exit 2 ;;
        *) TARGET="$1"; shift ;;
    esac
done

if [ -z "$TARGET" ]; then
    echo "FAIL: no target file given. Usage: $0 <path-to-script.ps1>" >&2
    exit 2
fi

if [ ! -r "$TARGET" ]; then
    echo "FAIL: target file not found or unreadable: $TARGET" >&2
    exit 2
fi

FAILURES=0

result() {
    # result <pass:0|1> <message>
    local pass="$1"; shift
    local msg="$*"
    if [ "$SILENT" -eq 1 ]; then return; fi
    if [ "$pass" -eq 1 ]; then
        if [ "$QUIET" -eq 1 ]; then return; fi
        printf 'OK:   %s\n' "$msg"
    else
        printf 'FAIL: %s\n' "$msg"
    fi
}

# has_emoji <file> -> exit 0 emoji found, 1 none found, 2 could not check.
# Ranges match Test-ScriptCompliance.ps1: misc symbols and dingbats
# (2600-27BF), misc symbols and arrows (2B00-2BFF), variation selectors
# (FE00-FE0F), the zero-width joiner (200D), and every astral-plane code
# point (1F000 and up). Prefers python3, then perl, then grep -P, because
# grep -P depends on a UTF-8 locale that may be absent.
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
    elif LC_ALL=C.UTF-8 grep -qP '.' "$f" >/dev/null 2>&1; then
        local pat='[\x{2600}-\x{27BF}\x{2B00}-\x{2BFF}\x{FE00}-\x{FE0F}\x{200D}\x{1F000}-\x{1FFFF}]'
        if LC_ALL=C.UTF-8 grep -qP "$pat" "$f"; then return 0; fi
        return 1
    fi
    return 2
}

# UTF-8 with no BOM
bom=$(head -c 3 "$TARGET" | od -An -tx1 | tr -d ' \n')
if [ "$bom" = "efbbbf" ]; then
    result 0 "UTF-8 with no byte-order mark"
    FAILURES=$((FAILURES+1))
else
    result 1 "UTF-8 with no byte-order mark"
fi

# LF line endings (no CR). Counted at the byte level so a Windows grep that
# strips CR cannot mask it.
cr_count=$(LC_ALL=C tr -dc '\r' < "$TARGET" | wc -c | tr -d ' ')
if [ "$cr_count" -ne 0 ]; then
    result 0 "LF line endings (no CR bytes)"
    FAILURES=$((FAILURES+1))
else
    result 1 "LF line endings (no CR bytes)"
fi

# No trailing whitespace on any line. Checked per LF-delimited line to match
# the PowerShell twin; CR-terminated lines are covered by the CR check above.
tw=$(awk 'BEGIN { t = sprintf("\t") } { if ($0 ~ ("[ " t "]+$")) c++ } END { print c+0 }' "$TARGET")
if [ "$tw" -ne 0 ]; then
    result 0 "No trailing whitespace ($tw offending line(s))"
    FAILURES=$((FAILURES+1))
else
    result 1 "No trailing whitespace (0 offending line(s))"
fi

# Exactly one trailing newline at end of file
last2=$(tail -c 2 "$TARGET" | od -An -tx1 | tr -d ' \n')
last1=$(tail -c 1 "$TARGET" | od -An -tx1 | tr -d ' \n')
if [ "$last1" = "0a" ] && [ "$last2" != "0a0a" ]; then
    result 1 "Exactly one trailing newline at end of file"
else
    result 0 "Exactly one trailing newline at end of file"
    FAILURES=$((FAILURES+1))
fi

# No emoji or pictographs
has_emoji "$TARGET"
emoji_rc=$?
if [ "$emoji_rc" -eq 0 ]; then
    result 0 "No emoji or pictographic characters"
    FAILURES=$((FAILURES+1))
elif [ "$emoji_rc" -eq 1 ]; then
    result 1 "No emoji or pictographic characters"
else
    [ "$SILENT" -eq 1 ] || printf 'WARN: emoji check skipped (need python3, perl, or grep -P)\n'
fi

# Four section dividers, each '#' + 79 underscores
dividers=$(grep -cE '^#_{79}$' "$TARGET")
if [ "$dividers" -eq 4 ]; then
    result 1 "Four 80-column section dividers present (found $dividers)"
else
    result 0 "Four 80-column section dividers present (found $dividers)"
    FAILURES=$((FAILURES+1))
fi

# The four named headings present, in order
order_ok=$(awk '
    BEGIN { idx = 0; split("## Declare Functions\n## Declare Variables and Arrays\n## Execute Operations\n## End of script", want, "\n") }
    {
        line = $0
        sub(/^[ \t]+/, "", line)
        sub(/[ \t]+$/, "", line)
        if (idx < 4 && line == want[idx+1]) { idx++ }
    }
    END { print (idx == 4) ? "1" : "0" }
' "$TARGET")
if [ "$order_ok" = "1" ]; then
    result 1 "Named section headings present in canonical order"
else
    result 0 "Named section headings present in canonical order"
    FAILURES=$((FAILURES+1))
fi

# Comment-based help block before the first [CmdletBinding
open_line=$(grep -n -m1 '<#' "$TARGET" | head -n1 | cut -d: -f1)
close_line=$(grep -n -m1 '#>' "$TARGET" | head -n1 | cut -d: -f1)
bind_line=$(grep -n -m1 '\[CmdletBinding' "$TARGET" | head -n1 | cut -d: -f1)
help_ok=1
if [ -z "$open_line" ] || [ -z "$close_line" ]; then
    help_ok=0
elif [ "$close_line" -le "$open_line" ]; then
    help_ok=0
elif [ -n "$bind_line" ] && [ "$close_line" -ge "$bind_line" ]; then
    help_ok=0
fi
if [ "$help_ok" -eq 1 ]; then
    result 1 "Comment-based help block precedes [CmdletBinding"
else
    result 0 "Comment-based help block precedes [CmdletBinding"
    FAILURES=$((FAILURES+1))
fi

if [ "$SILENT" -ne 1 ]; then
    echo ""
    if [ "$FAILURES" -eq 0 ]; then
        printf 'OK:   %s is compliant.\n' "$TARGET"
    else
        printf 'FAIL: %s has %s compliance issue(s).\n' "$TARGET" "$FAILURES"
    fi
fi

if [ "$FAILURES" -gt 0 ]; then
    exit 1
fi

exit 0
