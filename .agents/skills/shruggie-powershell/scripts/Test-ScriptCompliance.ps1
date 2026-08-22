<#
.SYNOPSIS
    Verify that a PowerShell script conforms to the ShruggieTech scripting
    standard's structural and encoding rules.

.DESCRIPTION
    Runs a set of deterministic, language-agnostic checks against a target
    .ps1 file and reports one OK or FAIL line per check. The checks are:

      - UTF-8 with no byte-order mark (no leading EF BB BF)
      - LF line endings (no CR bytes)
      - No trailing whitespace on any line
      - Exactly one trailing newline at end of file
      - No emoji or pictographic characters anywhere in the file
      - The four named section dividers present, in order, each a '#' followed
        by exactly 79 underscores
      - A leading comment-based help block before the CmdletBinding attribute

    It also prints advisory WARN lines (never affecting the exit code) for
    two further concerns:

      - Possible leaked PII: a Windows or Unix/macOS home-directory path with a
        captured username segment, or an email-address-shaped string
        (a heuristic, best-effort scan of the script's own text)
      - A declared function whose verb is unapproved, or whose full name
        already collides with an inbox or installed module's command (both
        via PSScriptAnalyzer's PSUseApprovedVerbs and
        PSAvoidOverwritingBuiltInCmdlets rules, run through
        Invoke-ScriptAnalyzer; if PSScriptAnalyzer is not installed, prints
        one WARN saying the scan was skipped rather than silently omitting it)

    A POSIX-shell twin (test-script-compliance.sh) performs the same structural
    checks and the PII scan so remote agents on non-Windows hosts can verify
    without pwsh; the PSScriptAnalyzer-backed scan needs a live PowerShell
    session with that module installed and is PowerShell-twin-only.

    Exit codes: 0 every check passed, 1 at least one check failed, 2 the target
    file could not be read. The advisory WARN lines never change the exit code.

.PARAMETER Path
    Path to the .ps1 file to check. Read literally.
    Alias: p

.PARAMETER Quiet
    Suppress the per-check OK lines and print only failures and the summary.
    FAIL lines always print.
    Alias: q

.PARAMETER Silent
    Suppress all output. The exit code still reports the verdict.

.PARAMETER SkipPiiCheck
    Skip the advisory scan for possible leaked PII (username-bearing paths,
    email-address-shaped strings).

.PARAMETER SkipVerbCheck
    Skip the advisory scan for function names using a verb outside
    PowerShell's approved-verb list.

.PARAMETER SkipCollisionCheck
    Skip the advisory scan for function names already claimed by an inbox or
    installed module.

.PARAMETER Help
    Print this help text to the terminal.
    Alias: h

.EXAMPLE
    .\Test-ScriptCompliance.ps1 -Path ..\assets\examples\Get-Secret.ps1
    Checks Get-Secret.ps1 and prints one OK or FAIL line per rule.

.EXAMPLE
    .\Test-ScriptCompliance.ps1 -Path .\Build-Thing.ps1 -Quiet
    Prints only failures; exit code carries the verdict for scripting.

.NOTES
    This script is authored to the standard it checks, so running it against
    itself is a useful self-test.
#>
[CmdletBinding(SupportsShouldProcess=$false,ConfirmImpact='None',DefaultParameterSetName='Default')]
Param(
    [Parameter(Mandatory=$true,ParameterSetName='Default')]
    [Alias("p")]
    [string]$Path,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("q")]
    [Switch]$Quiet,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Switch]$Silent,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Switch]$SkipPiiCheck,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Switch]$SkipVerbCheck,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Switch]$SkipCollisionCheck,

    [Parameter(Mandatory=$true,ParameterSetName='HelpText')]
    [Alias("h")]
    [Switch]$Help
)
#_______________________________________________________________________________
## Declare Functions

    function Write-Result {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [bool]$Pass,

            [Parameter(Mandatory=$true)]
            [string]$Message
        )
        if ($script:Silent) { return }
        if ($Pass) {
            if ($script:Quiet) { return }
            Write-Host ("OK:   {0}" -f $Message) -ForegroundColor Green
        } else {
            Write-Host ("FAIL: {0}" -f $Message) -ForegroundColor Red
        }
    }

    function Test-NoEmoji {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$Text
        )
        # Flag pictographic code points numerically so this source needs no
        # non-ASCII characters and no escape sequences. Ranges: misc symbols
        # and dingbats (0x2600-0x27BF), misc symbols and arrows
        # (0x2B00-0x2BFF), variation selectors (0xFE00-0xFE0F), the zero-width
        # joiner (0x200D), and every astral-plane code point (0x1F000 and up,
        # which covers the emoji blocks).
        $enum = [System.Globalization.StringInfo]::GetTextElementEnumerator($Text)
        while ($enum.MoveNext()) {
            $cp = [System.Char]::ConvertToUtf32($enum.GetTextElement(), 0)
            if ($cp -ge 0x1F000) { return $false }
            if ($cp -ge 0x2600 -and $cp -le 0x27BF) { return $false }
            if ($cp -ge 0x2B00 -and $cp -le 0x2BFF) { return $false }
            if ($cp -ge 0xFE00 -and $cp -le 0xFE0F) { return $false }
            if ($cp -eq 0x200D) { return $false }
        }
        return $true
    }

    function Get-PiiWarning {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$Text
        )
        # Advisory, best-effort scan. Never fails the run; the caller decides
        # what to do with the findings. Deliberately does not attempt a
        # phone-number pattern (collides too readily with version strings and
        # port numbers) or free-text personal-name detection (not
        # deterministically tractable).
        $findings = New-Object System.Collections.Generic.List[string]

        $winHits = [regex]::Matches($Text, 'C:\\Users\\([^\\]+)\\').Count
        if ($winHits -gt 0) {
            $findings.Add(("Windows user-profile path with a captured username segment ({0} hit(s))" -f $winHits))
        }

        $unixHits = [regex]::Matches($Text, '(?:/home/|/Users/)([^/\s]+)/').Count
        if ($unixHits -gt 0) {
            $findings.Add(("Unix/macOS home-directory path with a captured username segment ({0} hit(s))" -f $unixHits))
        }

        $emailHits = [regex]::Matches($Text, '[\w.+-]+@[\w-]+\.[\w.-]+').Count
        if ($emailHits -gt 0) {
            $findings.Add(("email-address-shaped string ({0} hit(s))" -f $emailHits))
        }

        return $findings
    }

    function Get-ScriptAnalyzerWarning {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$ScriptText,

            [Parameter(Mandatory=$true)]
            [string[]]$RuleName
        )
        # Shells out to the real PSScriptAnalyzer instead of reimplementing
        # its rules. An earlier version of this function hand-rolled the verb
        # check with an AST scan against (Get-Verb).Verb and the collision
        # check with a live Get-Command lookup; both looked reasonable and
        # both were wrong in a way that mattered. The live Get-Command check
        # in particular only sees modules discoverable in THIS session, and
        # PSDesiredStateConfiguration (the inbox module Write-Log collided
        # with, the exact bug this check exists to catch) is a Windows
        # PowerShell 5.1 module not on PowerShell 7's module path, so the
        # hand-rolled check silently passed a fixture that reproduced the
        # real bug. PSScriptAnalyzer ships a versioned, bundled snapshot of
        # inbox and common module exports for exactly this reason, and
        # Invoke-ScriptAnalyzer is the only reliable way to consult it.
        #
        # Returns an array always, never $null: a clean scan produces no
        # output, which PowerShell collapses to $null on capture, so an
        # unwrapped return here would be indistinguishable from "no findings"
        # at the call site. The caller checks tool availability separately
        # (Get-Command Invoke-ScriptAnalyzer) before deciding whether an
        # empty result here means "clean" or "could not run."
        return @(Invoke-ScriptAnalyzer -ScriptDefinition $ScriptText -IncludeRule $RuleName -ErrorAction SilentlyContinue)
    }

#_______________________________________________________________________________
## Declare Variables and Arrays

    $script:Quiet  = [bool]$Quiet
    $script:Silent = [bool]$Silent

    $ThisScriptPath = $MyInvocation.MyCommand.Path

    $script:Dividers = @(
        '## Declare Functions',
        '## Declare Variables and Arrays',
        '## Execute Operations',
        '## End of script'
    )

#_______________________________________________________________________________
## Execute Operations

    # Catch help text requests
    if (($Help) -or ($PSCmdlet.ParameterSetName -eq 'HelpText')) {
        Get-Help $ThisScriptPath -Detailed
        exit 0
    }

    # Environment precondition: the target file must be readable
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        Write-Host ("FAIL: target file not found: {0}" -f $Path) -ForegroundColor Red
        exit 2
    }

    try {
        $bytes = [System.IO.File]::ReadAllBytes($Path)
    } catch {
        Write-Host ("FAIL: could not read {0}: {1}" -f $Path, $_.Exception.Message) -ForegroundColor Red
        exit 2
    }

    $failures = 0

    # UTF-8 with no BOM
    $hasBom = ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF)
    Write-Result -Pass (-not $hasBom) -Message 'UTF-8 with no byte-order mark'
    if ($hasBom) { $failures++ }

    # LF line endings (no CR)
    $hasCr = ($bytes -contains 0x0D)
    Write-Result -Pass (-not $hasCr) -Message 'LF line endings (no CR bytes)'
    if ($hasCr) { $failures++ }

    $text  = [System.Text.Encoding]::UTF8.GetString($bytes)
    $lines = $text.Split("`n")

    # No trailing whitespace on any line
    $trailing = @($lines | Where-Object { $_ -match '[ \t]+$' })
    Write-Result -Pass ($trailing.Count -eq 0) -Message ("No trailing whitespace ({0} offending line(s))" -f $trailing.Count)
    if ($trailing.Count -ne 0) { $failures++ }

    # Exactly one trailing newline at end of file
    $singleEof = ($text.Length -gt 0 -and $text[-1] -eq "`n" -and -not ($text.EndsWith("`n`n")))
    Write-Result -Pass $singleEof -Message 'Exactly one trailing newline at end of file'
    if (-not $singleEof) { $failures++ }

    # No emoji or pictographs
    $noEmoji = Test-NoEmoji -Text $text
    Write-Result -Pass $noEmoji -Message 'No emoji or pictographic characters'
    if (-not $noEmoji) { $failures++ }

    # Four section dividers present, each '#' + 79 underscores
    $dividerCount = @($lines | Where-Object { $_ -match '^#_{79}$' }).Count
    Write-Result -Pass ($dividerCount -eq 4) -Message ("Four 80-column section dividers present (found {0})" -f $dividerCount)
    if ($dividerCount -ne 4) { $failures++ }

    # The four named headings present, in order
    $idx = 0
    $inOrder = $true
    foreach ($line in $lines) {
        if ($idx -lt $script:Dividers.Count -and $line.Trim() -eq $script:Dividers[$idx]) {
            $idx++
        }
    }
    if ($idx -lt $script:Dividers.Count) { $inOrder = $false }
    Write-Result -Pass $inOrder -Message 'Named section headings present in canonical order'
    if (-not $inOrder) { $failures++ }

    # Comment-based help block before the first [CmdletBinding
    $openIdx  = $text.IndexOf('<#')
    $closeIdx = $text.IndexOf('#>')
    $bindIdx  = $text.IndexOf('[CmdletBinding')
    $helpOk = ($openIdx -ge 0 -and $closeIdx -gt $openIdx -and ($bindIdx -lt 0 -or $closeIdx -lt $bindIdx))
    Write-Result -Pass $helpOk -Message 'Comment-based help block precedes [CmdletBinding'
    if (-not $helpOk) { $failures++ }

    # Advisory only: possible leaked PII. Never affects $failures or the exit
    # code; opt out with -SkipPiiCheck. Suppressed by -Silent, not by -Quiet
    # (matches how Write-ShruggieLog's Warn level already survives -Quiet).
    if (-not $SkipPiiCheck -and -not $script:Silent) {
        foreach ($finding in (Get-PiiWarning -Text $text)) {
            Write-Host ("WARN: possible leaked PII: {0}" -f $finding) -ForegroundColor Yellow
        }
    }

    # Advisory only: PSScriptAnalyzer's approved-verb rule and its
    # built-in/inbox-cmdlet-collision rule. Never affects $failures or the
    # exit code. Requires the PSScriptAnalyzer module; if it is not
    # installed, prints one WARN saying so rather than silently skipping the
    # scan. Opt out of either half independently with -SkipVerbCheck /
    # -SkipCollisionCheck.
    if ((-not $SkipVerbCheck -or -not $SkipCollisionCheck) -and -not $script:Silent) {
        if (-not (Get-Command Invoke-ScriptAnalyzer -ErrorAction SilentlyContinue)) {
            Write-Host 'WARN: PSScriptAnalyzer is not installed; skipped the approved-verb and built-in-cmdlet-collision scans' -ForegroundColor Yellow
        } else {
            $rules = New-Object System.Collections.Generic.List[string]
            if (-not $SkipVerbCheck)      { $rules.Add('PSUseApprovedVerbs') }
            if (-not $SkipCollisionCheck) { $rules.Add('PSAvoidOverwritingBuiltInCmdlets') }
            foreach ($r in (Get-ScriptAnalyzerWarning -ScriptText $text -RuleName $rules)) {
                Write-Host ("WARN: [{0}] {1}" -f $r.RuleName, $r.Message) -ForegroundColor Yellow
            }
        }
    }

    if (-not $script:Silent) {
        Write-Host ''
        if ($failures -eq 0) {
            Write-Host ("OK:   {0} is compliant." -f $Path) -ForegroundColor Green
        } else {
            Write-Host ("FAIL: {0} has {1} compliance issue(s)." -f $Path, $failures) -ForegroundColor Red
        }
    }

    if ($failures -gt 0) {
        exit 1
    }

    exit 0

#_______________________________________________________________________________
## End of script
