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

    A POSIX-shell twin (test-script-compliance.sh) performs the same checks so
    remote agents on non-Windows hosts can verify without pwsh.

    Exit codes: 0 every check passed, 1 at least one check failed, 2 the target
    file could not be read.

.PARAMETER Path
    Path to the .ps1 file to check. Read literally.
    Alias: p

.PARAMETER Quiet
    Suppress the per-check OK lines and print only failures and the summary.
    FAIL lines always print.
    Alias: q

.PARAMETER Silent
    Suppress all output. The exit code still reports the verdict.

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
