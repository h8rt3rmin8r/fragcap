# Bash Script Conventions and Standards

This document defines the Bash scripting conventions used across ShruggieTech
work. It is self-contained and serves as the authoritative foundation for the
`shruggie-bash` skill. It is the deliberate twin of the PowerShell standard: the
house shape (80-column underscore dividers, a load-bearing help block, the 0/1/2
exit contract, the `-q`/`--silent` verbosity family, no emojis, UTF-8-no-BOM /
LF output, a deterministic compliance checker) is identical; only the language
idiom changes.

Bash is the primary shell for ShruggieTech Linux and server tooling. Several
rules below exist because shell has more silent-failure modes than almost any
other language an agent will author in, and because AI coding agents authoring
shell have predictable failure modes (unquoted expansions, `ls` parsing, CRLF
contamination, over-trusting `set -e`) that these conventions head off at the
source.

When updating an existing script, bring it to compliance with this document
rather than carrying forward divergent forms.

## Target Runtime

Scripts target Bash 5.x on Linux and macOS, invoked through
`#!/usr/bin/env bash` (never a hardcoded `/bin/bash`, which is Bash 3.2 on stock
macOS and absent on some minimal Linux images). Assume Bash 4.3+ semantics:
associative arrays (`declare -A`), `mapfile` / `readarray`, `[[ ... ]]`, `local
-n` namerefs. When a script must run under POSIX `sh` or Bash 3.2, document the
version-sensitive constructs and their fallbacks inline at the point of use;
do not litter a 5.x script with caveats it will never encounter.

## Shebang and File Naming

Line 1 is exactly `#!/usr/bin/env bash`. This resolves Bash through `PATH`,
which picks up a newer Homebrew or `/usr/local` Bash on macOS ahead of the stock
3.2. A hardcoded interpreter path defeats that.

File names are lowercase with hyphens and end in `.sh` (`collect-system.sh`,
`rotate-secret.sh`). Function names are lowercase with underscores
(`collect_system`, `check_dependencies`). The `verb_noun` intent mirrors the
PowerShell `Verb-Noun` rule (a verb-like lead, then a noun phrase) without
importing PascalCase. Hyphens in files, underscores in functions: the split
keeps file names shell-completion friendly and function names valid identifiers.

## Header Help Block

Immediately under the shebang, every script carries a man-page-style comment help
block: a run of `# ` comment lines using the conventional section headings, in
this order where present:

```
# script-name.sh - one-line description
#
# NAME
#     script-name.sh - one-line description
#
# SYNOPSIS
#     script-name.sh [options] [args]
#     script-name.sh --help
#
# DESCRIPTION
#     Multi-paragraph explanation including side effects, network or privilege
#     requirements, and anything an operator should know before running it.
#
# OPTIONS
#     -h, --help      Show this help and exit.
#     -q, --quiet     Suppress informational output.
#     ...
#
# EXAMPLES
#     Common invocation:
#         ./script-name.sh
#     ...at least two, generously beyond...
#
# EXIT CODES
#     0  Success
#     1  Runtime or assertion failure
#     2  Environment precondition failure
#
# AUTHOR
#     h8rt3rmin8r for ShruggieTech (Shruggie LLC)
```

Add `ENVIRONMENT` (documented environment variables the script reads),
`DEPENDENCIES` (required and optional commands, with the install hint), and
`COMPATIBILITY` (tested distributions and minimum Bash version) where they earn
their place. The help block is load-bearing: it is the interface boundary
between the agent that authored the script and the operator who runs it, so more
help is better than less. At least two worked `EXAMPLES` are required. Separate
the last help line from the first divider with a blank `#` line so the block does
not read as truncated.

The block is emitted at runtime by the self-parsing `print_help` fixture, which
reads the script's own source and prints the header comment lines back out. This
keeps the help text and its source in exactly one place: there is no second
here-doc copy to drift out of sync. The fixture is in `fixtures.md`.

### Library and Function-Level Docs

A reusable library file (a collection of `source`-able functions, as opposed to
an executable script) documents each function with an inline tagged-doc block,
the `novx` house style:

```
some_function() {
    #A> ABOUT:
    #A>
    #A>    What the function does.
    #A>
    #U> USAGE:
    #U>
    #U>    some_function <INPUT>
    #U>    some_function <OPTION>
    #U>
    #E> EXAMPLES:
    #E>
    #E>    some_function "hello"
    #E>
    #R> REFERENCE:
    #F> RELATED:
    #F>
    #F>    related_function
    #T> TAGS:
    #T>
    #T>    @category-some-grouping
    #________________________________________________________________________________
    if [[ "$1" == "-h" || "$1" == "--help" ]]; then print_function_help; return $?; fi
    # body...
}
```

The tags are `#A>` (about), `#U>` (usage, with a `|`-aligned option table),
`#E>` (examples), `#R>` (reference), `#F>` (related functions), and `#T>` (tags,
`@category-*` for grouping). A shared help renderer greps these tags out of the
calling function's body when it sees `-h`/`--help`. Use this style for
multi-function libraries; use the man-page header block for executable scripts.

## Safety Preamble

Directly after the help block, before the first divider, set the shell options.
Two modes are conventional, chosen by the script's error posture.

### Default: abort on failure

```bash
set -euo pipefail
IFS=$'\n\t'
```

- `set -e` (`errexit`): the shell exits on any command that returns non-zero and
  is not explicitly handled. Fail fast rather than blundering on with a broken
  assumption.
- `set -u` (`nounset`): expanding an unset variable is an error, not an empty
  string. This turns a typo (`$fille` for `$file`) into an immediate failure
  instead of a silent `rm -rf /$fille/`. Guard genuinely-optional variables with
  `"${VAR:-default}"`.
- `set -o pipefail`: a pipeline's exit status is the last non-zero of any stage,
  not just the final command. Without it, `false | tee log` succeeds.
- `IFS=$'\n\t'`: drop the space from the word-splitting set so unquoted
  expansions split only on newlines and tabs, not spaces. Reduces the blast
  radius of an accidental unquoted expansion. (Quoting is still the real fix.)

Consider also `shopt -s inherit_errexit` (so `errexit` reaches inside `$(...)`
command substitutions, which it does not by default) and `shopt -s nullglob`
(so an unmatched glob expands to nothing rather than the literal pattern) when
the script's logic depends on either.

### Graceful degradation: pipefail only

```bash
set -o pipefail
# pipefail only: this tool degrades gracefully past missing optional tools
```

Probes, collectors, and inventory tools deliberately continue past missing
commands and use `|| note "..."` fallbacks on nearly every call. Under `errexit`
those fallbacks and `has_cmd` checks would abort the run at the first optional
tool that is absent. Dropping `errexit` is the correct, considered choice for
this class of script (William's `sysinv.sh` is the reference). When a script
uses this mode, state the omission in a one-line comment so it reads as
deliberate, not forgotten. Such scripts must handle errors explicitly instead:
`has_cmd` guards, `|| note`, `|| return 0`, and checked exit statuses.

### When errexit betrays you

`set -e` is not a substitute for handling errors; it has documented blind spots.
Do not assume it catches everything:

- A command whose failure is "used" is not fatal: the left side of `&&`/`||`, any
  command in an `if`/`while`/`until` condition, or a command negated with `!`. A
  function called from `if some_func; then` runs with `errexit` effectively
  disabled inside it.
- Failure inside `$(...)` does not trigger the outer `errexit` unless
  `inherit_errexit` is set.
- Only the last command in a pipeline is checked unless `pipefail` is set; and
  even then, a pipeline feeding another command (`cmd | while read ...`) can mask
  status.
- `local x="$(cmd)"` always returns 0 because the `local` builtin's own status
  wins; `cmd`'s failure is lost. Declare then assign when the status matters
  (see Path and Expansion Hygiene).

## Named Section Dividers

The script body is divided into four named sections, each preceded by a divider
that is a `#` followed by exactly 79 underscores, for a total of 80 characters (a
nod to classic 80-column terminals, and identical to the PowerShell twin):

```
#_______________________________________________________________________________
# Declare Functions

#_______________________________________________________________________________
# Declare Variables and Arrays

#_______________________________________________________________________________
# Execute Operations

#_______________________________________________________________________________
# End of script
```

The four headings appear in this exact order. `# End of script` is the final
content line of the file. `# Declare Functions` and `# Declare Variables and
Arrays` may be empty if the script does not need them, but the dividers are still
present so the file shape is uniform across the whole collection. Long scripts
may add indented sub-dividers (a `#` plus 75 underscores at four-space indent)
inside `Declare Functions` to group collectors or subsystems, as `sysinv.sh`
does.

Functions are declared before variables and before the operations body because
nothing executes until `Execute Operations`; a function may therefore reference a
global defined in the section below it. This ordering is safe in an executable
script (as opposed to a sourced library, where definition order can matter).

## Body Indentation and Code Folding

All content beneath each section divider and heading is indented four spaces. The
divider lines and the `#` headings sit flush at column zero; everything else is
indented one level under them. This is a hard convention, not cosmetic: the
flush-left headers with uniformly indented bodies create clean, predictable fold
regions. An operator or agent can collapse `# Declare Functions` and read the
operations flow at a glance. Maintain the four-space body indent throughout,
including inside nested helper functions. Use four spaces, never tabs, for
indentation (a literal tab is reserved for the rare here-doc that needs `<<-`).

## Verbosity, Logging, and Colorized Output

Robust progress reporting is a feature, not noise. Scripts default to active,
informative output. All diagnostics go to stderr (`>&2`), leaving stdout clean
for the actual payload so the script composes in a pipe.

### Suppression Flags

- `-q` / `--quiet` suppress informational chatter (info, success, debug).
  Warnings and errors still emit, because a silenced failure helps no one.
- `--silent` suppresses warnings too. Genuine errors still reach stderr so
  failures are never fully hidden.

Do not use `-s` as a suppression alias; it is too commonly bound to "string" or
"short" across the scripting world. For a script whose stdout is a structured
payload, `-q` additionally means "emit only the payload, no decoration" (for
example, no trailing status line), so the value pipes cleanly.

### Logging Fixtures

Scripts that report operator-facing progress use the `log_info` / `log_warn` /
`log_error` / `debug_log` fixtures rather than scattered `printf` calls. They
write to stderr, colorize by level through `C_*` variables, and honor
`--no-color`, the `NO_COLOR` environment variable, and a non-TTY stdout by
blanking the color codes (color auto-off when `[[ ! -t 1 ]]`, when writing to a
file, or when emitting JSON). `debug_log` routes through file descriptor 3, gated
on a `*_DEBUG` environment variable, so debug lines survive the `2>/dev/null`
redirects collectors apply to suppress tool noise. The fixtures are in
`fixtures.md`. Sample output:

```
[INFO] collecting system inventory
[WARN] docker daemon unreachable; skipping container section
[ERROR] output parent directory is not writable: /var/reports
```

## No Emojis in Script Output

Scripts do not emit emojis. Not in log lines, banners, status markers, or
anywhere. Emoji-decorated output renders inconsistently across terminals and
encodings and is a frequent tell of unreviewed AI-generated code. Status is
conveyed through the level label and color of the logging fixtures (`INFO`,
`WARN`, `ERROR`, `OK`, `FAIL`, and color), never through pictographs. This rule
is absolute.

## Error Handling Policy

The right posture depends on a script's maturity and audience. During development
and for anything not yet thoroughly tested, errors are loud and fatal: run under
`set -euo pipefail` so the first uncaught failure stops the script at the line
that caused it. Mature graceful-degradation tooling uses the documented `set -o
pipefail`-only mode with explicit `has_cmd` guards and `|| note` fallbacks. In
either mode, route diagnostics to stderr with attribution, and prefer a bare
fatal message plus `exit` for unrecoverable conditions over letting a raw tool
error be the only signal.

### Operator-Facing Diagnostics

For scripts whose audience is an interactive operator (smoke tests, health
probes, lifecycle wrappers), emit a single structured `OK:` or `FAIL:` line per
check with the remediation hint embedded in the message, for example
`log_error "FAIL: dev server unreachable. Is it running on port 8787?"`. Reserve
a raw stack-style dump for genuine script-internal failures. An environmental
failure that reads as "the script broke" sends the operator to debug the wrong
layer.

## Exit-Code Conventions

Three baseline codes:

- `0` success.
- `1` a runtime or assertion failure. The script reached its work but a check
  failed or a required step errored.
- `2` an environment precondition failure. The script could not start its work
  (a missing dependency, an unwritable output target, a version too low).

A script may extend the contract with higher codes (`3`, `4`, ...) for distinct
failure classes when that genuinely helps an operator or a CI dispatcher;
`sysinv.sh`, for example, uses `1` for argument errors, `2` for output-path
validation, and `3` for a missing dependency. Every code a script uses must be
listed under `EXIT CODES` in the header. Exit codes are an inter-script contract;
undocumented additions break operators and any pipeline logic that reads them.

## Argument Parsing

Parse arguments with an explicit loop:

```bash
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -h|--help) print_help; exit 0 ;;
            -q|--quiet) opt_quiet=1; shift ;;
            -o|--out|--output) consume_output "$1" "${2:-}"; shift 2 ;;
            -o=*|--out=*|--output=*) consume_output "${1%%=*}" "${1#*=}"; shift ;;
            --) shift; break ;;
            -*) log_error "Unknown option: $1"; log_error "Try '--help'."; exit 1 ;;
            *) positional+=("$1"); shift ;;
        esac
    done
    set -- "${positional[@]:-}"
```

Handle both `--flag value` and `--flag=value` forms, treat `--` as the
end-of-options terminator, reject unknown `-*` options with a pointer to
`--help`, and make the `-h`/`--help` gate exit `0` before any work. Use `getopts`
only for a script that is purely short-option and will never grow long options;
the manual loop is the house default because it scales to long options and
`=value` without a rewrite. `-h` is reserved exclusively for help.

## Path and Expansion Hygiene

This is the single largest source of silent breakage in AI-authored shell, so it
gets explicit treatment.

### Quote everything

Quote every expansion unless word-splitting is the explicit intent:
`"$var"`, `"${arr[@]}"`, `"$(cmd)"`, `"$@"`. An unquoted `$var` undergoes word
splitting and glob expansion, so a value with spaces or a `*` silently becomes
multiple arguments or a directory listing. `"$@"` (quoted) preserves each
argument exactly; `$@` and `$*` do not. `"${arr[@]}"` expands to one word per
element; `"${arr[*]}"` joins on the first `IFS` char.

### Test with [[ ]]

Prefer `[[ ... ]]` over `[ ... ]` / `test`. `[[ ]]` does not word-split its
operands, supports `&&` / `||` / `=~`, and will not misfire when a variable is
empty or contains a metacharacter. Use `(( ... ))` for arithmetic tests.

### Never parse ls

`for f in $(ls)` breaks on any filename with a space, newline, or glob
character. Iterate with a glob (`for f in ./*.log`, guarded by `shopt -s
nullglob` when the directory may be empty) or, for recursion or arbitrary names,
`find ... -print0 | while IFS= read -r -d '' f; do ...; done`, or `mapfile -d ''
-t files < <(find ... -print0)`. The `-print0` / `-d ''` pairing is the only
robust way to carry filenames that may contain newlines.

### Guard leading dashes with --

Pass `--` before positional paths to commands that take options
(`rm -- "$f"`, `cp -- "$src" "$dst"`, `dirname -- "$p"`, `grep -- "$pat" "$f"`)
so a filename like `-rf` or `-n` is read as data, not a flag.

### local masks exit status

`local x="$(cmd)"` always returns 0 because the `local` builtin's own success is
the statement's status; `cmd`'s failure is lost, and `errexit` never fires.
Declare then assign on separate lines when the status matters:

```bash
    local x
    x="$(cmd)"     # now errexit sees cmd's failure
```

### Command substitution strips newlines

`"$(cmd)"` removes all trailing newlines. This is usually convenient, but when
trailing newlines are significant, append a sentinel (`x="$(cmd; printf x)";
x="${x%x}"`) or read with `mapfile`.

### Subshell and pipeline scope

A pipeline runs each stage in a subshell, so `cmd | while read ...; do count=$((count+1)); done`
loses `count` when the loop ends. Feed the loop with process substitution instead
(`while read ...; do ...; done < <(cmd)`) or enable `shopt -s lastpipe` (only
effective when job control is off). Any variable set inside a subshell `( ... )`
or a `$(...)` is likewise invisible to the parent.

### Arithmetic returns non-zero on a zero result

`(( x ))` and `(( x = 0 ))` return exit status 1 when the arithmetic result is 0,
which kills a script under `errexit`. Use `x=$(( ... ))` assignment form, or
guard with `|| true`, or use `(( x++ ))` with care (`(( i++ ))` returns the
pre-increment value's truthiness). For incrementing under `errexit`, `i=$((i+1))`
is safe.

## printf over echo

Use `printf`, not `echo`, for anything beyond the most trivial literal. `echo`
is not portable: its handling of `-n`, `-e`, and backslash escapes varies by
shell and by the `xpg_echo` / `posix` options, and a value that starts with `-`
is swallowed as a flag. The house idiom is `printf '%s\n' "$value"` for a line
and `printf '%s' "$value"` for no trailing newline. Never pass untrusted data as
the `printf` format string; it goes in an argument (`printf '%s\n' "$data"`,
never `printf "$data"`).

## trap and Cleanup

A script that creates temp files, background processes, or lock files installs a
cleanup trap so it tidies up on normal exit and on interruption:

```bash
    cleanup() {
        [[ -n "${TMPDIR_RUN:-}" && -d "${TMPDIR_RUN}" ]] && rm -rf -- "${TMPDIR_RUN}"
    }
    trap cleanup EXIT INT TERM
    TMPDIR_RUN="$(mktemp -d)"
```

Create temp files with `mktemp` / `mktemp -d`, never a predictable
`/tmp/myscript.$$`. Make cleanup idempotent (guard every removal with an
existence check) because `EXIT` can fire after `INT` has already run it.

## No Trailing Whitespace

No line ends with stray whitespace: code, comments, and blank lines alike. A line
used for spacing is genuinely empty. Trailing whitespace produces noisy diffs,
trips linters and pre-commit hooks, and is invisible until it causes a problem.
Strip it before saving, including blank lines inside nested function bodies.

## File Encoding and the BOM

Scripts are saved as UTF-8 without a BOM. A leading BOM (bytes `EF BB BF`) sits
in front of the shebang, so the kernel does not see `#!` at offset 0 and the
script fails to exec (or runs under the wrong interpreter). Verify with
`file script.sh` (expect `ASCII text` or `UTF-8 Unicode text`, never
`UTF-8 Unicode (with BOM) text`) or `head -c3 script.sh | od -An -tx1` (must not
be `ef bb bf`).

## Line Endings: the CRLF Trap

Normalize line endings to LF. CRLF is the single most destructive invisible bug
in shell scripts, which is why it gets its own section:

- A trailing `\r` on the shebang line makes the interpreter path literally
  `/usr/bin/env bash\r`, and the kernel reports `bad interpreter: /usr/bin/env
  bash^M: No such file or directory`. The path looks correct in every editor.
- A `\r` at the end of any line becomes part of the last token on that line. A
  comparison `[[ "$mode" == "start" ]]` silently fails because the value is
  actually `start\r`. A variable read from the file carries the `\r`. A
  here-doc terminator `EOF\r` never matches its opening `EOF`.
- `read` includes the trailing `\r` in the last field, so parsed values are
  contaminated one field at a time.

Detect it: `file script.sh` reports `with CRLF line terminators`; `cat -A
script.sh` shows a `^M` before each `$`; `grep -lU $'\r' script.sh` names files
that contain a CR byte. Fix it: `sed -i 's/\r$//' script.sh`, or `tr -d '\r' <
in > out`, or `dos2unix script.sh`. Prevent it: a `.gitattributes` entry
`*.sh text eol=lf` forces LF on checkout across platforms, and an `.editorconfig`
`[*.sh] end_of_line = lf` keeps editors honest.

The file ends with the `# End of script` divider section followed by exactly one
trailing LF. No content follows it and no blank lines accumulate after it.

## ShellCheck

Every script is checked with ShellCheck (`shellcheck script.sh`) before it is
considered done, and a clean run is a release gate. ShellCheck catches the
majority of the pitfalls in this document mechanically: unquoted expansions
(SC2086), `local` masking a return value (SC2155), useless `ls` parsing (SC2012),
`$(...)` word-split hazards (SC2046), unquoted array expansions (SC2068), unset
variables under `nounset`, and non-portable `echo` usage. When a check must be
suppressed, do it with a narrowly-scoped `# shellcheck disable=SCxxxx` comment
directly above the line, and only with a reason: a blanket disable at the top of
the file hides real bugs. Treat a suppression the way you would treat a cast in a
typed language: occasionally correct, always deliberate.

## Edge-case gotchas (the cheat-list)

The failure modes that most often break a Bash script in the field, one line
each. Run through these before declaring a script done:

- CRLF line endings: a `\r` on the shebang breaks exec; a `\r` anywhere breaks
  string comparisons, `read`, and here-doc terminators. Enforce LF.
- A BOM before the shebang stops the script from exec-ing. UTF-8, no BOM.
- Unquoted `$var` word-splits and glob-expands; quote every expansion.
- `"$@"` preserves arguments; `$@` and `$*` mangle them.
- `for f in $(ls)` breaks on spaces and newlines; use globs or `find -print0`.
- Missing `--` lets a `-`-leading filename be read as a flag.
- `local x="$(cmd)"` hides `cmd`'s exit status; declare then assign.
- `set -e` does not fire inside `$(...)` (without `inherit_errexit`), in `if`
  conditions, on the left of `&&`/`||`, or on non-final pipe stages without
  `pipefail`.
- `(( x = 0 ))` returns non-zero and aborts under `errexit`; use `x=$(( ... ))`.
- `cmd | while read` runs the loop in a subshell; variables set there are lost.
  Use `< <(cmd)` process substitution.
- `$(...)` strips trailing newlines; use a sentinel when they matter.
- `[ $var = x ]` breaks when `$var` is empty or has spaces; use `[[ ]]`.
- `echo "$x"` mangles values that start with `-` or contain escapes; use
  `printf '%s\n' "$x"`.
- `printf "$user_data"` treats data as a format string; put data in an argument.
- Predictable temp paths (`/tmp/x.$$`) race and collide; use `mktemp`.
- A missing cleanup `trap` leaks temp files and processes on interrupt.
- `read` without `-r` eats backslashes; almost always use `read -r`.
- `nounset` (`set -u`) aborts on an unset optional var; use `"${VAR:-default}"`.
- Sourced libraries must not `set -e`/`-u`/`pipefail`; it leaks into the caller.

## Reference and Precedence

The reference implementations for these conventions are `sysinv.sh` (the
graceful-degradation operator-diagnostic shape) and the bundled examples in
`assets/examples/`. If a standalone script and this document ever drift, the
standalone script is updated to match this document.
