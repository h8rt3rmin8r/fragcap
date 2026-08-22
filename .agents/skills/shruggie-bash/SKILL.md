---
name: shruggie-bash
description: Author or refactor Bash (.sh) scripts to the ShruggieTech scripting standard, comprising a fixed 80-column four-section layout, a load-bearing man-page header help block with a self-parsing print_help, an explicit safety preamble (set -euo pipefail + IFS, or a documented set -o pipefail for graceful-degradation tools), snake_case naming, the has_cmd / log_* / safe_run fixtures, -q/--quiet and --silent verbosity with NO_COLOR and TTY detection, a 0/1/2 exit-code contract, no emojis, and UTF-8-no-BOM LF output that passes the bundled ShellCheck-aware compliance checker. Use whenever the user asks to write a Bash, sh, or shell script, make a .sh file, or bring a shell script up to our standard. Trigger on phrasings like write a bash script that, make me a .sh, refactor this bash to our conventions, or bring this script up to the ShruggieTech standard. Skip PowerShell, Python, Node, and other-language scripts, throwaway one-liners, and repos that declare their own different shell conventions.
disable-model-invocation: false
---

# Shruggie Bash

Author and refactor Bash scripts to the ShruggieTech scripting standard. This
skill is the Bash twin of `shruggie-powershell`: same house shape (the 80-column
underscore dividers, the load-bearing help block, the 0/1/2 exit contract, the
`-q`/`--silent` verbosity family, no emojis, UTF-8-no-BOM / LF output, a
deterministic compliance checker) rendered in idiomatic Bash rather than
PowerShell. The skill bundles the authoritative convention document, copy-paste
fixtures, a blank scaffold, worked examples, and a ShellCheck-aware checker so a
standard-shaped `.sh` can be emitted without re-deriving the rules each time.

Scripts target Bash 5.x on Linux and macOS, invoked through
`#!/usr/bin/env bash`. Assume Bash 4.3+ semantics (associative arrays, `mapfile`,
`[[ ... ]]`); when a script must run under POSIX `sh` or Bash 3.2 (stock macOS
`/bin/bash`), document the version-sensitive constructs inline at the point of
use rather than littering a 5.x script with caveats it will never hit.

## When to Use

Invoke this skill when:

- The user asks to write, author, or generate a Bash, `sh`, or shell script, or
  a `.sh` file, in a ShruggieTech context.
- The user asks to bring an existing shell script up to "our standard", "our
  conventions", or "the ShruggieTech standard", or to refactor a `.sh` for
  compliance.
- The user describes operator tooling to be delivered as Bash: a system
  inventory or health probe, a backup or rotation utility, a deploy or smoke
  test, an install or bootstrap script, a cleanup job, a cron-driven collector.
- The user is iterating on a `.sh` that already follows these rules.

Do not invoke this skill for:

- PowerShell (`shruggie-powershell` owns that), Python, Node, or other-language
  scripts. They follow their own conventions.
- Throwaway interactive one-liners typed at a prompt that are not saved as a
  script file.
- Editing shell scripts inside a repository that declares its own, different
  shell conventions. Defer to the local project standard; if it is unclear which
  applies, ask before reshaping the file.

## Instructions

The deliverable is a single `.sh` file shaped exactly as the standard
prescribes. When you need an exact value, the full gotcha list, or the rationale
behind a rule, read `assets/bash-conventions.md`; it is the single source of
truth and this body is the working summary. Copy the fixtures from
`assets/fixtures.md` verbatim rather than retyping them. Start from
`assets/script-template.sh` and fill it in. Before considering any Bash finished,
mentally run it against `assets/bash-conventions.md`'s "Edge-case gotchas"
section: the failure modes there (CRLF line endings, unquoted expansions, `set
-e` blind spots) are the ones that bite in production, not at author time.

### Canonical shape

The script body is divided into four named sections, each preceded by a divider
that is `#` followed by exactly 79 underscores (80 columns total). The four
headings appear in this exact order, and `# End of script` is the final content
line of the file:

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

The dividers and `#` headings sit flush at column zero. Everything beneath a
heading (function definitions, variable assignments, the operations body) is
indented four spaces, including inside nested helpers. This is the same
flush-left-header / indented-body shape as `shruggie-powershell`, and it exists
for the same reason: clean, predictable editor fold regions. The two `Declare`
sections may be empty, but their dividers are still present so the file shape is
uniform across the whole script collection.

Functions are declared before variables because nothing executes until the
`Execute Operations` section, so a function may safely reference a global defined
below it. Names use lowercase `snake_case` with a verb-like lead
(`collect_system`, `check_dependencies`, `print_help`, `safe_run`); the
`verb_noun` intent mirrors the PowerShell `Verb-Noun` rule without importing its
PascalCase. Reserve `main` for the single entry point when a script uses one.

### Header help block

Every script opens, immediately under the shebang, with a man-page-style comment
help block: a run of `# ` comment lines using the conventional section headings
`NAME`, `SYNOPSIS`, `DESCRIPTION`, `OPTIONS`, `EXAMPLES`, `EXIT CODES`, and
`AUTHOR`, adding `ENVIRONMENT`, `DEPENDENCIES`, and `COMPATIBILITY` where they
earn their place. Treat it as load-bearing, not boilerplate; it is the interface
boundary between the agent that authored the script and the operator who runs it,
so more help is better than less, and at least two worked `EXAMPLES` are
required. The block is emitted at runtime by the self-parsing `print_help`
fixture (it prints the header comments back out), which keeps the help text and
its source in exactly one place. Never let a `# ` help line be followed
immediately by the first divider without a blank `#` separator, or the block
reads as truncated.

### Safety preamble

Directly after the help block, before the first divider, set the shell options.
Two modes are conventional, chosen by the script's error posture:

- Default (`set -euo pipefail` with `IFS=$'\n\t'`): the right choice for scripts
  that should abort on the first unhandled failure. `errexit` stops on any
  uncaught non-zero, `nounset` turns a typo'd `$var` into an immediate error,
  `pipefail` propagates failure from any stage of a pipe, and the tightened
  `IFS` removes the space from the word-splitting set so unquoted expansions
  split only on newlines and tabs.
- Graceful-degradation (`set -o pipefail` alone): the right choice for probes,
  collectors, and inventory tools that deliberately continue past missing
  optional commands and use `|| note "..."` fallbacks on nearly every call. Under
  `errexit` those fallbacks and optional-tool checks would abort the run, so
  `errexit` is dropped on purpose. When a script uses this mode, say so in a
  one-line comment (`# pipefail only: this tool degrades gracefully past missing
  tools`) so the omission reads as deliberate, not forgotten.

`set -euo pipefail` has documented blind spots (a failure inside `$(...)`, in the
non-last stage of a pipe consumed by another command, in a function called from
an `if` condition). `assets/bash-conventions.md` lists them; do not assume
`errexit` catches everything. A script that is `source`d rather than executed
must NOT set any of these, because they would leak into the caller's shell.

### Verbosity, logging, and no emojis

Scripts default to active, informative output on stderr, leaving stdout clean for
the actual payload. Suppress it with a Unix-style flag family: `-q` / `--quiet`
suppress informational chatter (info, success, debug) while warnings and errors
still emit; `--silent` suppresses warnings too, but genuine errors still reach
stderr. Never use `-s` as a suppression alias; it is too commonly bound to
"string" or "short". Progress reporting uses the `log_info` / `log_warn` /
`log_error` / `debug_log` fixtures (in `assets/fixtures.md`): they write to
stderr, colorize by level through `C_*` variables, and honor `--no-color`, the
`NO_COLOR` environment variable, and a non-TTY stdout by blanking the color
codes. `debug_log` routes through file descriptor 3 so it survives the
`2>/dev/null` redirects collectors apply to suppress tool noise.

Scripts never emit emojis, anywhere: not in log lines, banners, or status
markers. They render inconsistently across terminals and encodings and are a tell
of unreviewed generated code. Status is carried by the level label and color
(`INFO`, `WARN`, `ERROR`, `OK`, `FAIL`), never a pictograph.

### Error handling and exit codes

Errors are loud and attributed. Route every diagnostic to stderr with `>&2` (the
`log_*` fixtures already do). Reserve a bare fatal message plus `exit` for
conditions the script cannot proceed past; use `log_warn` and continue for
recoverable gaps. The exit-code contract has three baseline values: `0` success,
`1` a runtime or assertion failure (the work ran but a check failed or a required
step errored), `2` an environment precondition failure (the work could not start:
a missing dependency, an unwritable target, a version too low). A script may
extend the contract with higher codes (`3`, `4`, ...) for distinct failure
classes when that genuinely helps an operator or a CI dispatcher, but every code
it uses must be listed under `EXIT CODES` in the header. Exit codes are an
inter-script contract; undocumented additions break the operators and pipelines
that read them.

### Robust argument parsing

Parse arguments with an explicit `while [[ $# -gt 0 ]]; do case "$1" in ... esac`
loop (see the fixture and `assets/examples/`). Handle both `--flag value` and
`--flag=value` forms, treat `--` as the end-of-options terminator, reject unknown
`-*` options with a pointer to `--help`, and make the `-h` / `--help` gate exit
`0` before any work. Use `getopts` only for a script that is purely short-option
and will never grow long options; the manual loop is the house default because it
scales to long options and `=value` forms without a rewrite. `-h` is reserved
exclusively for help.

### Path and expansion hygiene

Unquoted expansions are the most reliable source of silent breakage in shell, so
the rules are strict. Quote every expansion (`"$var"`, `"${arr[@]}"`,
`"$(cmd)"`) unless word-splitting is the explicit intent. Prefer `[[ ... ]]` over
`[ ... ]`. Never parse `ls`; iterate files with a glob (guarded by `shopt -s
nullglob` when empty is possible) or `find ... -print0 | while IFS= read -r -d ''
f`. Pass `--` before positional paths to `rm`, `cp`, `mv`, `dirname`, `basename`
so a leading-dash filename is not read as a flag. `local x="$(cmd)"` swallows
`cmd`'s exit status (the `local` builtin succeeds), so under `errexit` declare
then assign on separate lines when the status matters. `assets/bash-conventions.md`
has the full list with reasons.

### Output hygiene

Every file the skill writes complies with the repo conventions: UTF-8 with no
BOM, LF line endings (never CRLF), no trailing whitespace on any line (blank
lines are genuinely empty), a single trailing newline at end of file, and zero
emojis. CRLF is called out separately because a single trailing `\r` on the
shebang line produces `bad interpreter: /usr/bin/env bash^M` and a `\r` anywhere
else silently corrupts string comparisons, `read` results, and here-docs. A
`.gitattributes` entry such as `*.sh text eol=lf` keeps this stable across
checkouts. Make the script executable conceptually (`chmod +x`); the shebang is
`#!/usr/bin/env bash`, never a hardcoded `/bin/bash`.

### Build procedure

When ready to emit the file:

1. Read `assets/script-template.sh` and `assets/fixtures.md`. Skim
   `assets/bash-conventions.md` for any rule you are unsure about, especially the
   "Edge-case gotchas" section, and read a matching example in
   `assets/examples/`.
2. Pick a lowercase `verb_noun`-style file name ending in `.sh`
   (`collect-system.sh`, `rotate-secret.sh`, `check-deploy.sh`). Hyphens in file
   names, underscores in function names.
3. Choose the safety preamble: `set -euo pipefail` + `IFS` for abort-on-failure
   scripts, or the documented `set -o pipefail` for graceful-degradation tools.
4. Write the man-page header help block: NAME, SYNOPSIS, DESCRIPTION, OPTIONS,
   at least two EXAMPLES, EXIT CODES, AUTHOR, plus ENVIRONMENT / DEPENDENCIES /
   COMPATIBILITY where useful.
5. Add only the fixtures the script needs: `print_help`; `log_*` and the color
   setup if it reports progress; `has_cmd` for optional-tool checks; `safe_run` /
   `safe_capture` if it shells out to commands that can hang; a `trap ... EXIT`
   cleanup if it creates temp files.
6. Fill the four sections under their dividers with four-space body indentation,
   with the `--help` / `-h` gate first in `Execute Operations` and argument
   parsing before any real work.
7. Run the pre-output checklist, write the file UTF-8-no-BOM with LF, then run
   `scripts/test-script-compliance.sh <file>` and `shellcheck <file>` and fix
   anything either flags.

### Pre-output checklist

Before declaring the script done, verify:

- Line 1 is exactly `#!/usr/bin/env bash`; the man-page help block follows, with
  a blank `#` line before the first divider.
- The safety preamble is present and correct for the script's error posture; a
  `set -o pipefail`-only script carries the one-line justification comment.
- The four section dividers are present, in order, each `#` plus 79 underscores;
  body content is indented four spaces; `# End of script` is the last content
  line.
- The `-h` / `--help` gate is the first action in `Execute Operations` and exits
  `0`; unknown options are rejected; exit codes match the documented contract and
  every code used appears under `EXIT CODES`.
- Every expansion that should not word-split is quoted; no `ls` parsing; `--`
  guards positional paths; `local x="$(cmd)"` is split when the status matters.
- No emojis anywhere; no `-s` suppression alias; diagnostics go to stderr.
- UTF-8 no BOM, LF endings (no CR bytes), no trailing whitespace, single trailing
  newline.
- Run `scripts/test-script-compliance.sh <file>` and confirm it exits 0, then run
  `shellcheck <file>` and confirm it is clean (or every suppression carries a
  justified `# shellcheck disable=SCxxxx` comment).

## Examples

### Example: a non-destructive utility

**User input:**

```
Write me a bash script that generates a secure random secret, with options for
length and output format.
```

**Expected output:**

A single `.sh` named with a `verb-noun` (for example `new-secret.sh`), shaped
like `assets/examples/new-secret.sh`: the man-page help block with multiple
examples, `set -euo pipefail` + `IFS`, a `Default`-style option set parsed by the
manual `case` loop with `-h`/`--help` first, clean payload to stdout honoring
`-q`/`--quiet` (no trailing decoration), and random bytes drawn from
`/dev/urandom` (via `head -c` / `od` / `openssl rand`), never `$RANDOM`. Passes
the bundled checker and ShellCheck.

### Example: a destructive script

**User input:**

```
Make me a .sh that deletes build artifacts older than two weeks from a folder.
```

**Expected output:**

A `.sh` shaped like `assets/examples/remove-stale-artifact.sh`: a `--dry-run`
default posture or an explicit `--force` gate before anything is deleted,
`find ... -print0 | while IFS= read -r -d ''` iteration (never `ls` parsing),
`rm --` with the literal path, `log_*` progress, a `trap 'cleanup' EXIT` for any
temp state, an environment-precondition check (`exit 2`) when the target
directory is missing, and `exit 1` if a deletion fails. The operator gets a
preview before any irreversible action.

### Example: an operator diagnostic or lifecycle tool

**User input:**

```
Write a bash script that collects a system health snapshot for a Linux server.
```

**Expected output:**

A `.sh` shaped like `assets/examples/probe-health.sh`: the graceful-degradation
`set -o pipefail` posture with its justification comment, `has_cmd` guards around
every optional tool, `safe_run` around commands that can hang, a single
structured `OK:` / `FAIL:` line per check, `--no-color` / `NO_COLOR` / TTY-aware
coloring, and the three exit codes. This is the shape William's own `sysinv.sh`
follows at larger scale.

## Additional Resources

- [`assets/bash-conventions.md`](assets/bash-conventions.md): the authoritative
  standard. Read it for exact values, the full quoting and path-handling rules,
  the `set -e` blind spots, the CRLF and encoding gotchas, the ShellCheck policy,
  the library / sourced-file conventions (including the `novx`-style `#A>`/`#U>`
  tagged-doc system for multi-function libraries), and the rationale behind every
  rule.
- [`assets/fixtures.md`](assets/fixtures.md): copy-paste-exact fixtures for the
  safety preamble, the self-parsing `print_help`, the `log_*` and color setup,
  `has_cmd` / `is_root`, `safe_run` / `safe_capture`, the `trap` cleanup, and the
  manual argument-parsing skeleton.
- [`assets/script-template.sh`](assets/script-template.sh): blank scaffold with
  the shebang, the header help skeleton, the safety preamble, the four dividers,
  and the help gate. Copy it and fill it in.
- [`assets/examples/new-secret.sh`](assets/examples/new-secret.sh): the canonical
  non-destructive, payload-to-stdout exemplar.
- [`assets/examples/remove-stale-artifact.sh`](assets/examples/remove-stale-artifact.sh):
  the destructive exemplar with a dry-run / force gate and `trap` cleanup.
- [`assets/examples/probe-health.sh`](assets/examples/probe-health.sh): the
  graceful-degradation operator-diagnostic exemplar.
- [`scripts/test-script-compliance.sh`](scripts/test-script-compliance.sh): a
  deterministic, ShellCheck-aware compliance checker that verifies encoding, line
  endings, trailing whitespace, emojis, the shebang, the section dividers, and
  the help block, and runs ShellCheck when it is installed. Run it after writing
  a script.
