---
name: shruggie-powershell
description: Author or refactor PowerShell (.ps1) scripts to the ShruggieTech scripting standard: a fixed four-section 80-column layout, a load-bearing comment-based help block, an explicit top-level CmdletBinding, Default plus HelpText parameter sets with single-letter aliases, the Write-Log and Assert-PSVersion fixtures, ShouldProcess gating for destructive actions, -LiteralPath path handling, a 0/1/2 exit-code contract, no emojis, and UTF-8-no-BOM with LF output. Use whenever the user asks to write a PowerShell or pwsh script, make a .ps1, or bring an existing script up to our standard. Trigger on phrasings like "write a PowerShell script that ...", "make me a .ps1", "a pwsh script to ...", "refactor this PowerShell to our conventions", or "bring this script up to the ShruggieTech standard". Skip Bash, Python, and other-shell scripts, throwaway interactive one-liners, and editing scripts in a repo that declares its own different PowerShell conventions.
disable-model-invocation: false
---

# Shruggie PowerShell

Author and refactor PowerShell scripts to the ShruggieTech scripting standard. The
skill bundles the authoritative convention document, the stable copy-paste
fixtures, a blank scaffold, four worked examples, and a deterministic compliance
checker (PowerShell and Bash twins) so Claude can emit a standard-shaped `.ps1`
without re-deriving the rules each time. The default deliverable is a single `.ps1`
file with the four named 80-column section dividers, a full comment-based help
block, an explicit top-level `[CmdletBinding(...)]`, `Default` plus `HelpText`
parameter sets, and UTF-8-no-BOM / LF output that passes the bundled checker.

Scripts target the latest PowerShell (7 or higher, invoked as `pwsh`). Assume 7+
semantics unless a script has a stated need for Windows PowerShell 5.1 backward
compatibility, in which case document the version-sensitive constructs inline.

## When to Use

Invoke this skill when:

- The user asks to write, author, or generate a PowerShell or `pwsh` script, or a
  `.ps1` file, in a ShruggieTech context.
- The user asks to bring an existing PowerShell script up to "our standard", "our
  conventions", or "the ShruggieTech standard", or to refactor a `.ps1` for
  compliance.
- The user describes operator tooling to be delivered as PowerShell: a secret or
  token generator, a dev-server lifecycle wrapper, a health probe, a deploy
  verifier, a cleanup or rotation utility.
- The user is iterating on a `.ps1` that already follows these rules.

Do not invoke this skill for:

- Bash, Python, Node, or other-language and other-shell scripts. They follow their
  own conventions.
- Throwaway interactive one-liners typed at a prompt that are not saved as a script
  file.
- Editing PowerShell inside a repository that declares its own, different PowerShell
  conventions. Defer to the local project standard; if it is unclear which applies,
  ask before reshaping the file.

## Instructions

The deliverable is a single `.ps1` file shaped exactly as the standard prescribes.
When you need an exact value, the full cmdlet lists, or the rationale behind a rule,
read `assets/powershell-conventions.md`; it is the single source of truth and this
body is the working summary. Copy the fixtures from `assets/fixtures.md` verbatim
rather than retyping them. Start from `assets/script-template.ps1` and fill it in.

### Canonical shape

The script body is divided into four named sections, each preceded by a divider
that is `#` followed by exactly 79 underscores (80 columns total). The four
headings appear in this exact order, and `## End of script` is the final content
line of the file:

```
#_______________________________________________________________________________
## Declare Functions

#_______________________________________________________________________________
## Declare Variables and Arrays

#_______________________________________________________________________________
## Execute Operations

#_______________________________________________________________________________
## End of script
```

The dividers and `##` headings sit flush at column zero. Everything beneath a
heading (functions, variable assignments, the operations body) is indented four
spaces, including inside nested helpers. This is a hard rule: the flush-left
headers with uniformly indented bodies create clean editor fold regions. The two
`Declare` sections may be empty, but their dividers are still present so the file
shape is uniform.

Names follow Verb-Noun in PascalCase for both files and functions
(`Get-Secret.ps1`, `Start-LocalDevServer.ps1`, `Get-CryptoBytes`,
`ConvertTo-Base64Url`). The noun phrase may concatenate several capitalized words.
Microsoft's approved-verb list is not enforced; the Verb-Noun shape is the
convention.

### Comment-based help block

Every script opens with a `<# ... #>` comment-based help block. Treat it as
load-bearing, not boilerplate; more help is better than less. Required tags:
`.SYNOPSIS` (one line), `.DESCRIPTION` (multi-paragraph, including side effects,
version and network constraints, clipboard or browser use), `.PARAMETER <Name>`
once per parameter with the single-letter alias documented inside the description
(for example `Alias: l`), and `.EXAMPLE` at least twice (the first is the most
common invocation; add more for meaningful flag combinations and piping). Add
`.NOTES`, `.OUTPUTS`, and `.LINK` where they help. Never put a literal `#>` inside
the help text; it closes the block early.

### Top-level CmdletBinding

Immediately after the help block, declare the explicit attribute. For read-only and
otherwise non-destructive scripts:

```powershell
[CmdletBinding(SupportsShouldProcess=$false,ConfirmImpact='None',DefaultParameterSetName='Default')]
```

The bare `[CmdletBinding()]` form is reserved for internal helper functions, which
each carry their own bare binding and a typed `Param(...)` block.

For scripts that delete or overwrite files, mutate database rows, terminate
processes, rotate secrets, force-push, or take any other hard-to-reverse action,
set `SupportsShouldProcess=$true` and a real `ConfirmImpact` (`'Low'`, `'Medium'`,
or `'High'`), and wrap every destructive call in a `$PSCmdlet.ShouldProcess(...)`
check (see the gate in `assets/fixtures.md`). The declaration alone does nothing:
an unguarded change still runs under `-WhatIf`, which is worse than not supporting
it. Never declare `-WhatIf` or `-Confirm` in `Param(...)`; they come for free.

### Param block

Use `Param(` with a capital P. Two parameter sets are conventional: `Default` (the
normal working set) and `HelpText` (exactly one parameter, the `-Help` switch,
`Mandatory=$true`). Each parameter carries a `[Parameter(...)]` with `Mandatory` and
`ParameterSetName` set explicitly, an `[Alias("x")]` single-letter shorthand,
validation attributes (`[ValidateSet(...)]`, `[ValidateRange(...)]`) where
applicable, and a typed declaration with a default value.

### Help dispatch

Capture the path once in `## Declare Variables and Arrays`:

```powershell
    $ThisScriptPath = $MyInvocation.MyCommand.Path
```

Make the help gate the first action in `## Execute Operations`, dispatching on
either the `-Help` switch or the `HelpText` parameter set, using `Get-Help
-Detailed` and an explicit `exit 0`:

```powershell
    if (($Help) -or ($PSCmdlet.ParameterSetName -eq 'HelpText')) {
        Get-Help $ThisScriptPath -Detailed
        exit 0
    }
```

Help is invokable three ways on purpose: `-Help`, its `-h` alias, and the
`HelpText` parameter set. Reserve `-h` exclusively for help.

### Verbosity, logging, and no emojis

Scripts default to active, informative output. Suppress it with a Unix-style flag
family: `-q` / `-Quiet` suppress informational chatter (Info, Success, Debug) while
warnings and errors still emit; `-Silent` suppresses all log output including
warnings, but genuine errors still reach the error stream. Never use `-s` as a
suppression alias. For a script whose stdout is a structured payload, `-Quiet` also
means emit only the payload with no decoration (for example, no trailing newline).

Scripts that report progress use the `Write-Log` fixture (in `assets/fixtures.md`):
it colorizes by level, timestamps every line, and tags the emitting sub-process via
`-Source`. Declare `$script:LogQuiet` and `$script:LogSilent` in the variables
section and wire the flags to them before any logging.

Scripts never emit emojis, anywhere. Status is conveyed through the level label and
color of the logging helper (`OK`, `FAIL`, `WARN`), never through pictographs.

### Error handling and exit codes

During development and for anything not yet thoroughly tested, errors are loud and
fatal: leave `$ErrorActionPreference` at its default or set it to `'Stop'`. Only
mature, internally owned, thoroughly tested tooling may soften this, and only as a
deliberate, scoped exception (a targeted `-ErrorAction SilentlyContinue`), never to
make error messages go away. `Set-StrictMode -Version Latest` is an optional
development aid.

The exit-code contract has exactly three values: `0` success, `1` assertion failure
(the work ran but a check failed), `2` environment precondition failure (the work
could not start: version too low, server unreachable, required binding missing).
Extend the contract in `assets/powershell-conventions.md` before adding new codes.

### Operator-facing diagnostics

For scripts whose audience is an interactive operator (smoke tests, lifecycle
wrappers, health probes), reserve `Write-Error` for genuine script-internal
failures (parse errors, contract violations, unrecoverable state); its multi-line
frame correctly reads as "this script broke". For environmental failures (an HTTP
404, a server unreachable, a missing secret) use a color `Write-Host` with explicit
attribution and a remediation hint, for example `Write-Host "FAIL: server
unreachable. Is the dev server running on port 8787?" -ForegroundColor Red`. Emit a
single `OK:` or `FAIL:` line per check.

### Path handling

Default to `-LiteralPath` for filesystem operations so names containing wildcard
metacharacters (`[`, `]`, `*`, `?`) or awkward whitespace are read verbatim. Reach
for `-Path` only when wildcard expansion is the actual intent. A few cmdlets do not
expose `-LiteralPath` (`New-Item`, `Join-Path`, `Start-Process`, `Import-Module`);
for those, use the `System.IO` .NET methods or pass the literal string as
documented in `assets/powershell-conventions.md`, which also covers the full
supported-cmdlet list and the Windows `\\?\` extended-length note.

### Version guard

Do not use `#Requires -Version`. When a script genuinely depends on a minimum
engine version (for example it uses `Invoke-WebRequest -SkipHttpErrorCheck`, which
is 7+ only), include the `Assert-PSVersion` fixture from `assets/fixtures.md` in
`## Declare Functions` and call it as the first operation; it exits with code 2. A
script that uses nothing version-sensitive does not need it.

### Output hygiene

Every file the skill writes complies with the repo conventions: UTF-8 with no BOM,
LF line endings, no trailing whitespace on any line (blank lines are genuinely
empty), a single trailing newline at end of file, and zero emojis. On PowerShell
7+, `Set-Content -Encoding utf8` and `Out-File -Encoding utf8` are BOM-free. When a
script must write BOM-free on 5.1, use `[System.IO.File]::WriteAllText(...)`.

### Build procedure

When ready to emit the file:

1. Read `assets/script-template.ps1` and `assets/fixtures.md`. Skim
   `assets/powershell-conventions.md` for any rule you are unsure about, and read a
   matching example in `assets/examples/`.
2. Pick a Verb-Noun PascalCase file name.
3. Choose the CmdletBinding: non-destructive default, or
   `SupportsShouldProcess=$true` with a real `ConfirmImpact` if the script takes any
   hard-to-reverse action.
4. Write the comment-based help block: synopsis, full description with constraints,
   one `.PARAMETER` per parameter with its alias, and at least two examples.
5. Declare the `Param(...)` block with the `Default` set and the `HelpText`
   `-Help` / `-h` switch, aliases, and validation attributes.
6. Add only the fixtures the script needs: `Write-Log` (and the suppression flags)
   if it reports progress; `Assert-PSVersion` if it uses version-specific features.
7. Fill the four sections under their dividers with four-space body indentation,
   with the help gate first in `## Execute Operations` and every destructive call
   wrapped in `$PSCmdlet.ShouldProcess(...)`.
8. Run the pre-output checklist, write the file UTF-8-no-BOM with LF, then run the
   compliance checker and fix anything it flags.

### Pre-output checklist

Before declaring the script done, verify:

- The four section dividers are present, in order, each `#` plus 79 underscores;
  body content is indented four spaces.
- The help block precedes `[CmdletBinding`, has the required tags, and contains no
  stray `#>`.
- CmdletBinding is the explicit top-level form; destructive actions are gated by
  `$PSCmdlet.ShouldProcess(...)`.
- `Default` and `HelpText` parameter sets exist; `-h` maps to help only.
- The help gate is the first operation; exit codes follow the 0/1/2 contract.
- No emojis anywhere; no `-s` suppression alias.
- UTF-8 no BOM, LF endings, no trailing whitespace, single trailing newline.
- Run `scripts/Test-ScriptCompliance.ps1 -Path <file>` (or
  `scripts/test-script-compliance.sh <file>` on a non-Windows host) and confirm it
  exits 0.

## Examples

### Example: a non-destructive utility

**User input:**

```
Write me a PowerShell script that generates a secure random API token, with
options for length and output format.
```

**Expected output:**

A single `.ps1` named with a Verb-Noun (for example `New-ApiToken.ps1`), shaped
exactly like `assets/examples/Get-Secret.ps1`: full help block with multiple
examples, the non-destructive top-level CmdletBinding, a `Default` set with typed
and validated parameters plus single-letter aliases, the `HelpText` set, the four
dividers with four-space body indentation, the help gate first, and clean payload
output honoring `-Quiet`. Crypto bytes come from
`System.Security.Cryptography.RandomNumberGenerator`, never `Get-Random`. Passes
the bundled checker.

### Example: a destructive script

**User input:**

```
Make me a .ps1 that deletes build artifacts older than two weeks from a folder.
```

**Expected output:**

A `.ps1` shaped like `assets/examples/Remove-StaleArtifact.ps1`:
`SupportsShouldProcess=$true` with `ConfirmImpact='High'`, every `Remove-Item`
wrapped in `$PSCmdlet.ShouldProcess(...)` and called with `-LiteralPath`, progress
through `Write-Log`, an environment-precondition check (`exit 2`) when the target
directory is missing, and `exit 1` if a deletion fails. The operator gets `-WhatIf`
and `-Confirm` for free.

For operator tooling that wraps a long-running local process, follow the start and
stop pair in `assets/examples/Start-LocalDevServer.ps1` and
`Stop-LocalDevServer.ps1`: a default port with a `-Port` override, a startup
banner, health-poll loop, attributed `Write-Host` on environmental failure, and the
three exit codes.

## Additional Resources

- [`assets/powershell-conventions.md`](assets/powershell-conventions.md): the
  authoritative standard. Read it for exact values, the full `-LiteralPath` cmdlet
  list and exceptions, the `System.IO` and `\\?\` notes, the 5.1-versus-7+ nuances,
  and the rationale behind every rule.
- [`assets/fixtures.md`](assets/fixtures.md): copy-paste-exact fixtures for the
  `Assert-PSVersion` version guard, the `Write-Log` logging helper, and the
  `$PSCmdlet.ShouldProcess(...)` destructive gate.
- [`assets/script-template.ps1`](assets/script-template.ps1): blank scaffold with
  the help block skeleton, the explicit CmdletBinding, the `Default` and `HelpText`
  parameter sets, the four dividers, the `$ThisScriptPath` capture, and the help
  gate. Copy it and fill it in.
- [`assets/examples/Get-Secret.ps1`](assets/examples/Get-Secret.ps1): the canonical
  non-destructive, payload-to-stdout exemplar.
- [`assets/examples/Remove-StaleArtifact.ps1`](assets/examples/Remove-StaleArtifact.ps1):
  the destructive exemplar with ShouldProcess gating.
- [`assets/examples/Start-LocalDevServer.ps1`](assets/examples/Start-LocalDevServer.ps1)
  and [`Stop-LocalDevServer.ps1`](assets/examples/Stop-LocalDevServer.ps1): the
  operator-tooling lifecycle pair.
- [`scripts/Test-ScriptCompliance.ps1`](scripts/Test-ScriptCompliance.ps1) and
  [`scripts/test-script-compliance.sh`](scripts/test-script-compliance.sh):
  deterministic compliance checkers (PowerShell and Bash twins) that verify
  encoding, line endings, trailing whitespace, emojis, the section dividers, and the
  help block. Run one after writing a script.
