# PowerShell Script Conventions and Standards

This document defines the PowerShell scripting conventions used across ShruggieTech work. It is self-contained and serves as the authoritative foundation for the `shruggie-powershell` skill. The canonical worked example is the secret-generation utility reproduced at the end of this document; every new PowerShell script follows the shape described here.

PowerShell is the primary shell on the ShruggieTech Windows development workstation. Several rules below exist because shell behavior on Windows differs from Linux runners (encoding defaults, line endings, path normalization), and because AI coding agents authoring scripts have predictable failure modes that these conventions head off at the source.

When updating an existing script, bring it to compliance with this document rather than carrying forward divergent forms.

## Target Runtime

Scripts target the latest PowerShell (7 or higher, invoked as `pwsh`). Assume PowerShell 7+ semantics unless a specific script has a stated requirement for broad backward compatibility with Windows PowerShell 5.1. When 5.1 compatibility is genuinely in scope, document the version-sensitive constructs and their fallbacks inline at the point of use, using judgment about how much detail is warranted. Do not litter a 7+ script with 5.1 caveats it will never encounter.

Two examples of version-sensitive behavior that matter in practice:

- `Invoke-WebRequest -SkipHttpErrorCheck` exists only in 7+. On 5.1 a non-2xx response throws, so a 5.1-compatible probe must wrap the call in try/catch instead.
- `Set-Content -Encoding utf8` writes UTF-8 without BOM on 7+, but writes UTF-8 with BOM on 5.1. See the File Encoding section.

### Version Guard Fixture

Do not use `#Requires -Version` statements. They have caused editor and tooling friction, and they fail at parse time with an opaque message rather than surfacing an actionable alert at the command line. When a script genuinely depends on a minimum version, enforce it logically with a guard that prints a loud, colorized alert and exits with the environment-precondition code (2). Use this verbatim shape as a stable fixture, declared in `## Declare Functions` and called as the first operation:

```powershell
    function Assert-PSVersion {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$false)]
            [version]$Minimum = '7.0'
        )
        $current = $PSVersionTable.PSVersion
        if ($current -lt $Minimum) {
            Write-Host ("ALERT: PowerShell {0}+ required; running {1}. Relaunch with 'pwsh'." -f $Minimum, $current) -ForegroundColor Red
            exit 2
        }
    }
```

Include the guard only when a script actually uses version-specific operators or cmdlets that misbehave on older engines. A script that uses nothing version-sensitive does not need it.

## Comment-Based Help Block

Every script opens with a `<# ... #>` comment-based help block, and that block is treated as load-bearing rather than as boilerplate. The help block is the interface boundary between an AI coding agent that authored the script and the human operator who runs it. For that reason, more help is better than less. Invest in it every time, for every script.

Required tags:

- `.SYNOPSIS` is a one-line description of what the script does.
- `.DESCRIPTION` is a multi-paragraph explanation including notable behaviors, side effects, and constraints (PowerShell version, network reachability, browser interaction, clipboard use, anything an operator should know before running it).
- `.PARAMETER <Name>` appears once per parameter. The single-letter alias is documented inside the parameter description (for example, `Alias: l`), not as a separate tag.
- `.EXAMPLE` appears at least twice, and generously beyond that. The first example is the most common invocation; subsequent examples cover meaningful flag combinations, edge cases, and piping patterns an operator is likely to want.

Optional tags are welcome and encouraged when they add operator value: `.NOTES` for caveats and provenance, `.OUTPUTS` for what the script emits to the pipeline, `.LINK` for related scripts or references. Err toward including useful context rather than omitting it.

The comment-based help block precedes everything else in the file. PowerShell associates it with the script automatically; no explicit `[CmdletBinding(HelpUri=...)]` is needed.

## CmdletBinding Attribute

Immediately after the comment block, top-level scripts declare the full `[CmdletBinding(...)]` attribute with explicit settings:

```powershell
[CmdletBinding(SupportsShouldProcess=$false,ConfirmImpact='None',DefaultParameterSetName='Default')]
```

The bare `[CmdletBinding()]` form is not used at the top level. The explicit shape documents intent (no `-WhatIf` or `-Confirm` semantics, default parameter set named) and matches the canonical example. The bare form is reserved for internal helper functions.

The `SupportsShouldProcess=$false, ConfirmImpact='None'` pairing shown above is the correct default for read-only and otherwise non-destructive scripts.

### When to Enable SupportsShouldProcess

When a script performs destructive or hard-to-reverse state changes, enable `SupportsShouldProcess=$true`. This is not merely permitted; it is encouraged whenever the logic warrants it. Operators reasonably expect dangerous tooling to support a dry run, and enabling this gives them the `-WhatIf` (preview without executing) and `-Confirm` (prompt before each action) common parameters for free, with no manual parameter declarations.

Treat it as warranted when a script deletes or overwrites files, drops or mutates database rows, tears down or reconfigures infrastructure, terminates processes, rotates or overwrites secrets, performs a force push, or takes any other action the operator would want to preview before committing to it.

When enabling it, set a meaningful `ConfirmImpact` rather than leaving it `'None'`. Use `'Low'`, `'Medium'`, or `'High'` to reflect the real blast radius. An operation marked `'High'` prompts for confirmation automatically under the default `$ConfirmPreference`, which is the desired behavior for genuinely dangerous actions:

```powershell
[CmdletBinding(SupportsShouldProcess=$true,ConfirmImpact='High',DefaultParameterSetName='Default')]
```

The declaration alone does nothing. This is the trap to avoid: setting `SupportsShouldProcess=$true` without gating the destructive calls means `-WhatIf` silently performs the change anyway, which is worse than not supporting it at all. Every destructive operation MUST be wrapped in a `$PSCmdlet.ShouldProcess(...)` check. The call returns `$false` under `-WhatIf` (and prints the automatic "What if:" line), and prompts the operator under `-Confirm` or when the impact meets the confirmation threshold:

```powershell
    foreach ($file in $targets) {
        if ($PSCmdlet.ShouldProcess($file.FullName, 'Remove file')) {
            Remove-Item -LiteralPath $file.FullName -Force
        }
    }
```

The first argument is the target the action affects; the second is a short description of the action. Do not declare `-WhatIf` or `-Confirm` in the `Param(...)` block; they are supplied automatically once `SupportsShouldProcess=$true` is set.

## Param Block

Use `Param(` with a capital P. Each parameter carries:

- A `[Parameter(...)]` attribute with `Mandatory` and `ParameterSetName` set explicitly.
- An `[Alias("x")]` for the single-letter shorthand.
- Validation attributes (`[ValidateSet(...)]`, `[ValidateRange(...)]`, and similar) where applicable.
- A typed declaration with a default value, for example `[int]$Length = 32` or `[string]$Format = 'Base64'`.

Two parameter sets are conventional:

- `Default` is the working parameter set the script is normally invoked under.
- `HelpText` contains exactly one parameter, the `-Help` switch, marked `Mandatory=$true`.

```powershell
Param(
    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("l")]
    [ValidateRange(1, 4096)]
    [int]$Length = 32,

    [Parameter(Mandatory=$true,ParameterSetName='HelpText')]
    [Alias("h")]
    [Switch]$Help
)
```

## Naming Convention

All script files and all functions follow a Verb-Noun shape: a verb-like word first, a single hyphen, then a noun phrase. Both halves use PascalCase. The noun phrase may be several words concatenated with no separators, each word capitalized.

- Script files: `Get-SomeYummyPizza.ps1`, `Start-LocalDevServer.ps1`, `Convert-LegacyManifest.ps1`.
- Functions: `Get-CryptoBytes`, `ConvertTo-Base64Url`, `Assert-PSVersion`.

This is the single naming standard going forward. Earlier work used mixed schemes (PascalCase for utilities, kebab-case for operator scripts) only because no standard existed yet. That era is over; everything uses the form above.

ShruggieTech does not validate verb choice against PowerShell's approved-verb list (`Get-Verb`). The Verb-Noun shape is the convention; Microsoft's specific verb taxonomy is not enforced and its unapproved-verb warning is ignored. If a script is ever imported as a module, suppress that warning at import with `Import-Module -DisableNameChecking` rather than renaming functions to satisfy the taxonomy.

## Help Dispatch

Help is invokable three ways, by deliberate design: the `-Help` switch, its `-h` alias, and the `HelpText` parameter set. All three are wired up on purpose. The redundancy reserves the `-h` namespace exclusively for help invocation. `-h` carries decades of muscle memory as the help flag, and locking it to help prevents any script from quietly repurposing it for something else.

Capture the script path once in `## Declare Variables and Arrays`:

```powershell
    $ThisScriptPath = $MyInvocation.MyCommand.Path
```

Then make the help gate the first action in `## Execute Operations`, dispatching on either the `-Help` switch or the `HelpText` parameter set:

```powershell
    if (($Help) -or ($PSCmdlet.ParameterSetName -eq 'HelpText')) {
        Get-Help $ThisScriptPath -Detailed
        exit 0
    }
```

Use `Get-Help -Detailed` (not `-Full`). The detailed form prints synopsis, syntax, parameters, and examples without the verbose remarks block. Exit explicitly with `exit 0` so the help path agrees with the documented exit-code contract.

## Named Section Dividers

The script body is divided into four named sections, each preceded by a divider line. The divider is a `#` followed by exactly 79 underscores, for a total length of 80 characters (a deliberate nod to classic 80-column terminals):

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

The four headings appear in this exact order. `## End of script` is the final content line of the file. `## Declare Functions` and `## Declare Variables and Arrays` may be empty if the script does not need them, but the dividers are still present so the file shape is uniform across the whole script collection.

## Body Indentation and Code Folding

All content beneath each section divider and heading is indented by four spaces. The divider lines and the `##` headings themselves sit flush at column zero; everything else (functions, variable assignments, the operations body) is indented one level under them.

This is a hard convention, not a cosmetic preference. The flush-left headers with uniformly indented bodies create clean, predictable fold regions in the editor. An operator or agent can collapse `## Declare Functions` and read the operations flow at a glance, then expand only the section in play. Inconsistent indentation breaks fold-region detection and defeats the entire structure, so the four-space body indent is maintained throughout, including inside nested helpers.

## Internal Function Pattern

Every helper function declared inside the script carries its own `[CmdletBinding()]` attribute and a typed `Param(...)` block, even when the function is short. This makes helpers introspectable via `Get-Help` and ensures consistent parameter validation.

Internal helpers use the bare `[CmdletBinding()]` form (no explicit `SupportsShouldProcess` or `ConfirmImpact` settings). The full attribute form is reserved for the top-level script.

```powershell
    function Get-CryptoBytes {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [int]$ByteCount
        )
        # ...
    }
```

## Verbosity, Logging, and Colorized Output

Robust progress reporting is a feature, not noise. Scripts default to active, informative output: what the script is doing, which sub-process emitted a given line, and when. Operators can suppress that output explicitly, but the default is on.

### Suppression Flags

Verbosity is suppressed through a Unix-style flag family:

- `-q` and `-Quiet` suppress informational chatter (Info, Success, Debug). Warnings and errors still emit, because a silenced failure helps no one.
- `-Silent` suppresses all log output including warnings. Genuine errors still reach the error stream so that failures are never fully hidden.

Do not use `-s` as a suppression alias. `-s` is too commonly bound to "string" or similar across the broader scripting world; reserving it for verbosity would clash with that convention.

For a script whose stdout is a structured payload (the secret-generation utility is the canonical case), `-Quiet` additionally means "emit only the payload, no decoration" (for example, suppressing a trailing newline so the value pipes cleanly). The intent is consistent: `-Quiet` means give me clean machine output and skip the human-facing extras.

### Logging Helper Fixture

Scripts that report operator-facing progress use a dedicated logging helper rather than scattered `Write-Host` calls. The helper colorizes by level, timestamps every line, and tags the emitting sub-process. Use this verbatim shape as a stable fixture, adjusting levels and colors only with reason. Declare the suppression state in `## Declare Variables and Arrays` and the helper in `## Declare Functions`:

```powershell
#_______________________________________________________________________________
## Declare Variables and Arrays

    $script:LogQuiet  = $false
    $script:LogSilent = $false

#_______________________________________________________________________________
## Declare Functions

    function Write-Log {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true,Position=0)]
            [string]$Message,

            [Parameter(Mandatory=$false)]
            [ValidateSet('Info','Success','Warn','Error','Debug')]
            [string]$Level = 'Info',

            [Parameter(Mandatory=$false)]
            [string]$Source = $null
        )
        if ($script:LogSilent -and $Level -ne 'Error') { return }
        if ($script:LogQuiet -and (@('Info','Success','Debug') -contains $Level)) { return }

        $stamp = (Get-Date).ToString('yyyy-MM-dd HH:mm:ss.fff')
        $tag   = if ($Source) { "[$Source] " } else { '' }
        $label = $Level.ToUpper().PadRight(7)
        $color = switch ($Level) {
            'Info'    { 'Gray' }
            'Success' { 'Green' }
            'Warn'    { 'Yellow' }
            'Error'   { 'Red' }
            'Debug'   { 'DarkGray' }
        }
        Write-Host ("{0} {1}{2} {3}" -f $stamp, $tag, $label, $Message) -ForegroundColor $color
    }
```

Sample output, with the `-Source` tag identifying which sub-process emitted each line:

```
2026-05-31 14:03:22.481 [BuildBundle] INFO    Compiling content bundle
2026-05-31 14:03:23.107 [BuildBundle] SUCCESS Bundle written: 412 entries
2026-05-31 14:03:23.109 [HealthPoll]  WARN    Endpoint slow to respond, retrying
```

## No Emojis in Script Output

Scripts do not emit emojis. Not in log lines, not in banners, not in status markers, not anywhere. Emoji-decorated output reads as unprofessional, renders inconsistently across terminals and encodings, and is a frequent tell of unreviewed AI-generated code. Status is conveyed through the level label and color of the logging helper (`OK`, `FAIL`, `WARN`, and color), never through pictographs. This rule is absolute and applies to all generated and authored scripts.

## Error Handling Policy

The right `$ErrorActionPreference` depends entirely on a script's maturity and audience, so this is a policy rather than a single mandated value.

During development and iteration, and for anything not yet thoroughly tested, errors are loud and fatal. Let the engine throw the full red error block and stop the script dead. The scary error frame and the instant halt are exactly what the iterative construction process needs; they surface the problem immediately at the line that caused it. Either leave `$ErrorActionPreference` at its default and let terminating errors propagate, or set it to `'Stop'` to make non-terminating errors fatal too.

Only for mature, internally owned tooling that has completed thorough testing, and only after a deliberate decision, may a script soften this (for example, a targeted `-ErrorAction SilentlyContinue` on a specific call, or a scoped `$ErrorActionPreference = 'SilentlyContinue'`). Silent continuation is an earned exception for known-safe paths, never a default reached for to make error messages go away.

`Set-StrictMode -Version Latest` is a useful development-time aid for catching uninitialized variables and similar mistakes. It is optional, not mandated, and most valuable while a script is being built.

## Operator-Facing Diagnostic Patterns

These conventions apply specifically to scripts whose primary audience is an interactive operator (smoke tests, dev-server lifecycle wrappers, deploy verifiers, health probes) rather than CI or other scripts. The signal-to-noise needs of an interactive operator differ from those of a CI runner. An environmental failure that reads as "the script broke" sends the operator to debug the wrong layer.

### Reserve Write-Error for Script-Internal Failures

The `Write-Error` cmdlet renders a multi-line frame that includes the script's file path, the line number, and tildes pointing at the failing call site. That frame reads visually as "this script broke." The attribution is correct when the failure is internal (a parse error, a contract violation, an unrecoverable invariant). It is actively misleading when the failure is environmental (an HTTP 404 on a probe, a server unreachable, a missing secret). Operators who did not author the script see the file-and-line citation and start debugging the script instead of the environment.

The convention:

- Use `Write-Error` for genuine script-internal failures: parse errors, contract violations, unrecoverable internal state, bad parameter combinations. The frame's framing is correct here.
- Use `Write-Host` with color and explicit attribution for environmental failures, for example `Write-Host "FAIL: server unreachable. Is the dev server running on port 8787?" -ForegroundColor Red`. The error frame is suppressed; the message text carries both the failure label and the actionable remediation hint.
- Emit a single structured `FAIL:` or `OK:` line per check, with the label and remediation hint embedded in the message rather than spread across an error frame.

In practice this combines with the logging helper above: probes wrap a single network call in try/catch and emit color-coded `Warn` or `Error` markers, reserving `Write-Error` for cases where the script itself cannot proceed.

## Exit-Code Conventions

The convention defines three exhaustive exit codes:

- `0` is success.
- `1` is an assertion failure. The script reached its work but a check failed (a probe returned the wrong payload, a schema mismatch, and similar).
- `2` is an environment precondition failure. The script could not start its work (PowerShell version too low, a server unreachable, a required binding missing, and similar).

If a future script needs more granularity, extend this section before introducing new codes. Exit codes are an inter-script contract; ad-hoc additions break operator expectations and any CI dispatch logic that reads them.

## Path Handling: Prefer -LiteralPath

This is one of the most reliable sources of silent breakage in AI-authored PowerShell, so it gets explicit treatment.

By default, the `-Path` parameter on provider cmdlets runs its value through wildcard (glob) expansion. Any path that contains a wildcard metacharacter (`[`, `]`, `*`, `?`) is therefore misinterpreted as a pattern rather than read as a literal name. `-LiteralPath` disables that expansion and uses the string exactly as given. Filenames with square brackets, asterisks, or leading and trailing whitespace are common enough (especially files synced down from cloud storage) that defaulting to `-LiteralPath` is the standard.

The breakage is not theoretical. Reading a file literally named `report[2026].txt`:

```powershell
Get-Content -Path        '.\report[2026].txt'   # FAILS: [2026] is read as a character-class glob
Get-Content -LiteralPath '.\report[2026].txt'   # works: the name is used verbatim
Test-Path   -Path        '.\report[2026].txt'    # returns $false (confidently wrong)
Test-Path   -LiteralPath '.\report[2026].txt'    # returns $true (correct)
```

### The Rule

Default to `-LiteralPath` for filesystem operations. Reach for `-Path` only when wildcard expansion is the actual intent (for example, deliberately enumerating `*.log`).

Most provider cmdlets an agent reaches for support `-LiteralPath`. The following are confirmed to accept it, and should be called with it:

`Get-Item`, `Get-ChildItem`, `Get-Content`, `Set-Content`, `Add-Content`, `Clear-Content`, `Copy-Item`, `Move-Item`, `Remove-Item`, `Rename-Item`, `Test-Path`, `Resolve-Path`, `Convert-Path`, `Split-Path`, `Out-File`, `Select-String`, `Get-FileHash`, `Unblock-File`, `Import-Csv`, `Export-Csv`, `Import-Clixml`, `Export-Clixml`, `Get-ItemProperty`, `Set-ItemProperty`, `Clear-ItemProperty`, `New-ItemProperty`, `Rename-ItemProperty`, `Push-Location`, `Set-Location`, `Invoke-Item`.

### The Exceptions

A few commonly used cmdlets do not expose `-LiteralPath`. Passing `-LiteralPath` to them is itself a binding error, so know the alternative:

- `New-Item` takes `-Path` only. For a target whose name contains wildcard metacharacters or awkward whitespace, create it through .NET instead: `[System.IO.Directory]::CreateDirectory($p)` for a directory, `[System.IO.File]::WriteAllText($p, $content)` for a file.
- `Join-Path` takes `-Path` and `-ChildPath` with no `-LiteralPath`. It is pure string composition and is safe as long as `-Resolve` is not used. For full control, `[System.IO.Path]::Combine($a, $b)` composes without touching the filesystem.
- `Start-Process` takes `-FilePath`, which is treated literally; quote the path and pass arguments through the `-ArgumentList` array rather than concatenating a command string.
- `Import-Module` takes `-Name`; pass a quoted literal path string for a file-based module.

### The Robust Fallback

When path handling has to be bulletproof, the .NET `System.IO` types (`[System.IO.File]`, `[System.IO.Directory]`, `[System.IO.Path]`) always treat their string arguments literally. No provider, no globbing, no surprises. For pathological inputs (names with leading or trailing whitespace), construct and pass the exact string and use these methods directly.

### Windows-Specific Note

On Windows, the file APIs normalize away trailing spaces and trailing dots from path segments, which makes a file literally named `"trailing ".txt"` unreachable by ordinary means. The extended-length path prefix `\\?\` (for example `\\?\C:\dir\trailing .txt`) disables that normalization and is the escape hatch for such files. It requires a fully-qualified path with backslash separators and is most reliable through the .NET `System.IO` methods. This is established Windows behavior and applies only on Windows.

## No Trailing Whitespace

No line in any script ends with stray whitespace. This applies to every line: code, comments, and blank lines alike. A line used for visual spacing is genuinely empty, containing no spaces or tabs before its newline. In regex terms, blank separation is `\n\n`, never `\n[ \t]+\n`.

Trailing whitespace produces noisy diffs, trips linters and pre-commit hooks, and is invisible in most editors until it causes a problem. Strip it before saving, including blank lines inside nested helper bodies.

## File Encoding

PowerShell scripts are saved as UTF-8 without BOM. PowerShell on Windows historically defaulted to UTF-16 LE with BOM, and the wrong encoding breaks Linux runners and `git diff` rendering.

A version nuance applies when scripts write files: on PowerShell 7+, `Set-Content -Encoding utf8` and `Out-File -Encoding utf8` produce UTF-8 without a BOM. On Windows PowerShell 5.1, those same calls emit a BOM, and a BOM-free write requires `[System.Text.UTF8Encoding]::new($false)` or `[System.IO.File]::WriteAllText(...)`. Since the target runtime is 7+, the `-Encoding utf8` form is BOM-free and preferred.

Verify a script's own encoding from PowerShell without leaving the shell:

```powershell
# Returns $true if the file begins with a UTF-8 BOM (EF BB BF). Expected: $false.
$bytes = [System.IO.File]::ReadAllBytes($ThisScriptPath) | Select-Object -First 3
($bytes[0] -eq 0xEF) -and ($bytes[1] -eq 0xBB) -and ($bytes[2] -eq 0xBF)
```

The equivalent check on a Linux runner is `file <script>.ps1`, which should report `ASCII text` or `UTF-8 Unicode text` and never `UTF-8 Unicode (with BOM) text`.

## Line Endings and End of File

Normalize line endings to LF. CRLF in scripts produces cross-platform diff noise and can confuse tooling that runs on both the Windows workstation and Linux runners. A `.gitattributes` entry such as `*.ps1 text eol=lf` keeps this stable across checkouts.

The file ends with the `## End of script` divider section followed by exactly one trailing newline (LF). No content follows it and no blank lines accumulate after it.

## Canonical Example: Secret Generation Utility

The following is the full source of a script that exercises every part of the convention: rich comment-based help with multiple examples, the explicit top-level `CmdletBinding` attribute, two parameter sets including the `HelpText` set, validation attributes and aliases, the four named 80-column section dividers, four-space body indentation, internal helpers using the bare `[CmdletBinding()]`, the `$ThisScriptPath` capture, the help dispatch, and clean exit handling. The cryptographic substance is incidental; the structural shape is the point.

```powershell
<#
.SYNOPSIS
    Generate a cryptographically secure random secret string for use in
    full-stack development.

.DESCRIPTION
    Generates random bytes from a cryptographically secure source
    (System.Security.Cryptography.RandomNumberGenerator, which pulls from
    the OS CSPRNG) and encodes them in the requested format. Suitable for
    OAuth cookie secrets, Auth.js AUTH_SECRET values, JWT signing keys,
    API tokens, CSRF secrets, and similar credentials.

    Defaults to 32 bytes encoded as standard Base64, which is the format
    expected by most Node.js, Cloudflare Workers, and Auth.js secret
    consumers.

    NOTE: This script does NOT use Get-Random, which is non-cryptographic
    on Windows PowerShell 5.1 and unsuitable for secret generation.

.PARAMETER Length
    Number of random bytes to generate before encoding. The resulting
    string length depends on the encoding (for example, 32 bytes produces
    about 44 Base64 characters, 64 hex characters, or 43 Base64Url
    characters). For the Alphanumeric format, this value is interpreted as
    the character count of the output string.
    Default: 32.
    Alias: l

.PARAMETER Format
    Output encoding format. Valid options:
      - Base64       (default; standard padded base64)
      - Base64Url    (URL-safe; no padding; for JWT and URL parameters)
      - Hex          (lowercase hexadecimal)
      - Alphanumeric (A-Z, a-z, 0-9; for secrets that must avoid symbols)
    Alias: f

.PARAMETER Count
    Number of secrets to generate. Each is printed on its own line.
    Default: 1.
    Alias: n

.PARAMETER Clipboard
    Copy the generated secret(s) to the clipboard instead of printing to
    stdout. Avoids leaving the secret in the terminal scrollback.
    Alias: c

.PARAMETER Quiet
    Emit only the payload with no trailing newline, for clean piping into
    another command. Suppresses human-facing decoration.
    Alias: q

.PARAMETER Help
    Print this help text to the terminal.
    Alias: h

.EXAMPLE
    .\Get-Secret.ps1
    Generates a single 32-byte Base64 secret (the most common case;
    suitable for AUTH_SECRET, cookie secrets, and similar).

.EXAMPLE
    .\Get-Secret.ps1 -Length 64 -Format Hex
    Generates a 64-byte secret encoded as 128 hex characters.

.EXAMPLE
    .\Get-Secret.ps1 -Format Base64Url -Clipboard
    Generates a URL-safe Base64 secret and copies it directly to the
    clipboard without echoing it to the terminal.

.EXAMPLE
    .\Get-Secret.ps1 -Count 5 -Format Alphanumeric -Length 16
    Generates five 16-character alphanumeric secrets, one per line.

.EXAMPLE
    .\Get-Secret.ps1 -Quiet | Set-Clipboard
    Generates a secret with no trailing newline and pipes it to the
    clipboard. Useful when the consumer is sensitive to whitespace.
#>
[CmdletBinding(SupportsShouldProcess=$false,ConfirmImpact='None',DefaultParameterSetName='Default')]
Param(
    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("l")]
    [ValidateRange(1, 4096)]
    [int]$Length = 32,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("f")]
    [ValidateSet('Base64','Base64Url','Hex','Alphanumeric')]
    [string]$Format = 'Base64',

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("n")]
    [ValidateRange(1, 1000)]
    [int]$Count = 1,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("c")]
    [Switch]$Clipboard,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("q")]
    [Switch]$Quiet,

    [Parameter(Mandatory=$true,ParameterSetName='HelpText')]
    [Alias("h")]
    [Switch]$Help
)
#_______________________________________________________________________________
## Declare Functions

    function Get-CryptoBytes {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [int]$ByteCount
        )
        $bytes = New-Object byte[] $ByteCount
        $rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
        try {
            $rng.GetBytes($bytes)
        } finally {
            $rng.Dispose()
        }
        return ,$bytes
    }

    function ConvertTo-Base64Url {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [byte[]]$InputBytes
        )
        $b64 = [Convert]::ToBase64String($InputBytes)
        return $b64.TrimEnd('=').Replace('+','-').Replace('/','_')
    }

    function ConvertTo-HexString {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [byte[]]$InputBytes
        )
        return -join ($InputBytes | ForEach-Object { '{0:x2}' -f $_ })
    }

    function ConvertTo-AlphanumericString {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [int]$CharCount
        )
        # 62-char alphabet. Reject bytes >= 248 (4 * 62) to avoid modulo
        # bias. Over-fetch to reduce loop iterations.
        $alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'
        $result = New-Object System.Text.StringBuilder
        while ($result.Length -lt $CharCount) {
            $batch = Get-CryptoBytes -ByteCount ($CharCount * 2)
            foreach ($b in $batch) {
                if ($b -lt 248) {
                    [void]$result.Append($alphabet[$b % 62])
                    if ($result.Length -ge $CharCount) { break }
                }
            }
        }
        return $result.ToString()
    }

    function New-Secret {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [int]$ByteLength,

            [Parameter(Mandatory=$true)]
            [string]$Encoding
        )
        switch ($Encoding) {
            'Base64' {
                $bytes = Get-CryptoBytes -ByteCount $ByteLength
                return [Convert]::ToBase64String($bytes)
            }
            'Base64Url' {
                $bytes = Get-CryptoBytes -ByteCount $ByteLength
                return ConvertTo-Base64Url -InputBytes $bytes
            }
            'Hex' {
                $bytes = Get-CryptoBytes -ByteCount $ByteLength
                return ConvertTo-HexString -InputBytes $bytes
            }
            'Alphanumeric' {
                return ConvertTo-AlphanumericString -CharCount $ByteLength
            }
        }
    }

#_______________________________________________________________________________
## Declare Variables and Arrays

    $ThisScriptPath = $MyInvocation.MyCommand.Path

#_______________________________________________________________________________
## Execute Operations

    # Catch help text requests
    if (($Help) -or ($PSCmdlet.ParameterSetName -eq 'HelpText')) {
        Get-Help $ThisScriptPath -Detailed
        exit 0
    }

    # Generate the requested number of secrets
    $secrets = @()
    for ($i = 0; $i -lt $Count; $i++) {
        $secrets += New-Secret -ByteLength $Length -Encoding $Format
    }

    $output = $secrets -join [Environment]::NewLine

    # Emit per the requested output mode
    if ($Clipboard) {
        $output | Set-Clipboard
        Write-Verbose "Copied $Count secret(s) to clipboard."
    } elseif ($Quiet) {
        [Console]::Out.Write($output)
    } else {
        Write-Output $output
    }

#_______________________________________________________________________________
## End of script
```

## Operator-Tooling Subclass: Local Process Lifecycle

A second canonical shape applies the convention to operator-facing tooling that wraps a long-running local process (a dev server, a watcher, a tunnel). It is expressed as a start and stop pair:

- A start script that builds any prerequisite artifacts, spawns the long-running process in a separate window so the operator's prompt stays usable, polls a health endpoint until the process is ready, prints next-step guidance, then returns control.
- A matching stop script that terminates the listener on the agreed port.

Both scripts default to a known port, accept a `-Port` override, and follow every convention above: rich comment-based help, the full top-level `CmdletBinding`, the parameter sets including `HelpText`, the four 80-column section dividers, four-space body indentation, the help dispatch, and the three exit codes. A script that wraps a long-running process additionally prints a startup banner that names the target, reports operator-actionable progress through the logging helper, and surfaces actionable diagnostics when a health probe fails. Per the operator-facing diagnostics convention, these scripts reserve `Write-Error` for script-internal failures and use color-coded `Write-Host` with explicit attribution for environmental failures.

## Reference and Precedence

The canonical example block above is the authoritative reference for PowerShell scripting conventions in ShruggieTech work. Standalone scripts that implement this pattern (such as the deployed `Get-Secret.ps1`) track this document. If a standalone script and this document ever drift, the standalone script is updated to match this document.
