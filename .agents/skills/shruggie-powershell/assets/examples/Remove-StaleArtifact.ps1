<#
.SYNOPSIS
    Delete build artifacts older than a cutoff age from a target directory.

.DESCRIPTION
    Enumerates files under a target directory that match a glob pattern and
    were last written before a cutoff (now minus OlderThanDays), then deletes
    them. This is a destructive operation, so the script declares
    SupportsShouldProcess and gates every deletion behind
    $PSCmdlet.ShouldProcess. Operators get -WhatIf (preview without deleting)
    and -Confirm (prompt per file) for free, and because ConfirmImpact is High
    the script prompts for confirmation by default unless -Confirm:$false is
    passed.

    Paths are handled with -LiteralPath so filenames containing wildcard
    metacharacters (square brackets, asterisks) are read verbatim.

    Exit codes: 0 success, 1 a deletion failed after it was attempted, 2 the
    target directory does not exist.

.PARAMETER Path
    Directory to scan for stale artifacts. Must exist.
    Alias: p

.PARAMETER OlderThanDays
    Age cutoff in days. Files last written more than this many days ago are
    candidates for deletion.
    Default: 14.
    Alias: d

.PARAMETER Pattern
    Glob pattern used to enumerate candidate files (passed to Get-ChildItem
    -Filter). Deliberate wildcard use; the literal-path rule applies to the
    deletion of each resolved file, not to this enumeration filter.
    Default: '*'.
    Alias: f

.PARAMETER Quiet
    Suppress informational chatter (Info, Success, Debug). Warnings and errors
    still emit.
    Alias: q

.PARAMETER Silent
    Suppress all log output including warnings. Genuine errors still reach the
    error stream.

.PARAMETER Help
    Print this help text to the terminal.
    Alias: h

.EXAMPLE
    .\Remove-StaleArtifact.ps1 -Path .\dist
    Previews nothing; prompts before deleting each artifact older than 14 days
    under .\dist (High impact prompts by default).

.EXAMPLE
    .\Remove-StaleArtifact.ps1 -Path .\dist -OlderThanDays 30 -WhatIf
    Shows what would be deleted (artifacts older than 30 days) without deleting
    anything.

.EXAMPLE
    .\Remove-StaleArtifact.ps1 -Path .\dist -Pattern '*.zip' -Confirm:$false -Quiet
    Deletes stale .zip artifacts without prompting and without informational
    output. Use only when the path is known-safe.

.NOTES
    Demonstrates the destructive-script pattern from the ShruggieTech PowerShell
    standard: SupportsShouldProcess=$true, a real ConfirmImpact, and a
    ShouldProcess gate around every state change.
#>
[CmdletBinding(SupportsShouldProcess=$true,ConfirmImpact='High',DefaultParameterSetName='Default')]
Param(
    [Parameter(Mandatory=$true,ParameterSetName='Default')]
    [Alias("p")]
    [string]$Path,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("d")]
    [ValidateRange(0, 36500)]
    [int]$OlderThanDays = 14,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("f")]
    [string]$Pattern = '*',

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

#_______________________________________________________________________________
## Declare Variables and Arrays

    $script:LogQuiet  = $false
    $script:LogSilent = $false

    $ThisScriptPath = $MyInvocation.MyCommand.Path

#_______________________________________________________________________________
## Execute Operations

    # Catch help text requests
    if (($Help) -or ($PSCmdlet.ParameterSetName -eq 'HelpText')) {
        Get-Help $ThisScriptPath -Detailed
        exit 0
    }

    # Wire suppression flags to the logging state
    if ($Quiet)  { $script:LogQuiet  = $true }
    if ($Silent) { $script:LogSilent = $true }

    # Environment precondition: the target directory must exist
    if (-not (Test-Path -LiteralPath $Path -PathType Container)) {
        Write-Host ("FAIL: target directory not found: {0}" -f $Path) -ForegroundColor Red
        exit 2
    }

    $cutoff = (Get-Date).AddDays(-$OlderThanDays)
    Write-Log ("Scanning {0} for '{1}' artifacts older than {2} ({3} days)." -f $Path, $Pattern, $cutoff.ToString('yyyy-MM-dd HH:mm:ss'), $OlderThanDays) -Level Info -Source 'Scan'

    $targets = Get-ChildItem -LiteralPath $Path -Filter $Pattern -File -Recurse |
        Where-Object { $_.LastWriteTime -lt $cutoff }

    if (-not $targets) {
        Write-Log "No stale artifacts found. Nothing to remove." -Level Success -Source 'Scan'
        exit 0
    }

    Write-Log ("Found {0} stale artifact(s)." -f $targets.Count) -Level Info -Source 'Scan'

    $removed = 0
    $failed  = 0
    foreach ($file in $targets) {
        if ($PSCmdlet.ShouldProcess($file.FullName, 'Remove stale artifact')) {
            try {
                Remove-Item -LiteralPath $file.FullName -Force
                $removed++
                Write-Log ("Removed: {0}" -f $file.FullName) -Level Success -Source 'Remove'
            } catch {
                $failed++
                Write-Log ("Could not remove {0}: {1}" -f $file.FullName, $_.Exception.Message) -Level Error -Source 'Remove'
            }
        }
    }

    Write-Log ("Done. Removed {0}, failed {1}." -f $removed, $failed) -Level Info -Source 'Remove'

    if ($failed -gt 0) {
        exit 1
    }

    exit 0

#_______________________________________________________________________________
## End of script
