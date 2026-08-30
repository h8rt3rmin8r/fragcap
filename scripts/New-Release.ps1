# SPDX-License-Identifier: Apache-2.0
<#
.SYNOPSIS
    Prepare a fragcap release branch (Phase A), on Windows.

.DESCRIPTION
    The PowerShell twin of scripts/cut-release.sh. It consolidates the local
    half of cutting a release into one command: it bumps the workspace version,
    folds the changelog.d/ fragments into CHANGELOG.md, corrects the two
    embedded-version assertions and the golden corpus that the bump moves, and
    runs the full check set, leaving a green release/X.Y.Z branch ready to open
    as a pull request.

    It deliberately stops there. Two authorizations are required by the
    constitution and are not automated: pushing the version tag (which fires the
    release workflow) and approving the crates-io environment (which lets the
    workflow publish). This script performs neither, and never tags, pushes, or
    publishes. What it removes is the fiddly, error-prone local dance, not the
    human gates.

    The heavy lifting (changelog assembly, release-notes derivation) lives in
    cargo xtask, not here, so this stays a thin orchestrator over git, cargo
    release, and the task runner.

.PARAMETER Level
    The bump: minor (default), patch, major, or an explicit X.Y.Z version.
    Alias: l

.PARAMETER DryRun
    Print the plan and preview the assembled changelog without creating a
    branch, bumping, or writing anything.
    Alias: d

.PARAMETER Date
    Override the release date stamped into the changelog section. Default: today.

.PARAMETER Quiet
    Suppress informational output; keep warnings and errors.
    Alias: q

.PARAMETER Silent
    Suppress warnings too; errors still emit.

.PARAMETER NoColor
    Disable colored output.

.PARAMETER Help
    Print this help text and exit.
    Alias: h

.EXAMPLE
    .\New-Release.ps1 minor -DryRun
    Preview the version bump and assembled changelog, changing nothing.

.EXAMPLE
    .\New-Release.ps1 minor
    Prepare release/X.Y.Z: bump, assemble changelog, fix goldens, run the checks.
#>
[CmdletBinding(SupportsShouldProcess=$false,ConfirmImpact='None',DefaultParameterSetName='Default')]
Param(
    [Parameter(Mandatory=$false,Position=0,ParameterSetName='Default')]
    [Alias("l")]
    [string]$Level = 'minor',

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("d")]
    [Switch]$DryRun,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [string]$Date,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("q")]
    [Switch]$Quiet,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Switch]$Silent,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Switch]$NoColor,

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

            [Parameter(Mandatory=$false,Position=1)]
            [ValidateSet('Info','Warn','Error')]
            [string]$Level = 'Info'
        )
        if ($script:LogSilent -and $Level -ne 'Error') { return }
        if ($script:LogQuiet -and $Level -eq 'Info') { return }
        $label = $Level.ToUpper().PadRight(5)
        $color = switch ($Level) {
            'Info'  { 'Gray' }
            'Warn'  { 'Yellow' }
            'Error' { 'Red' }
        }
        if ($script:LogNoColor) {
            [Console]::Error.WriteLine("$label $Message")
        } else {
            Write-Host "$label $Message" -ForegroundColor $color
        }
    }

    # Fail early on an unsupported PowerShell, so a version-specific idiom does
    # not surface as a confusing error midway through a release.
    function Assert-PSVersion {
        if ($PSVersionTable.PSVersion.Major -lt 7) {
            Write-Log "PowerShell 7 or newer is required (found $($PSVersionTable.PSVersion))" 'Error'
            exit 2
        }
    }

    # Run a native command and stop on a non-zero exit, so a failing step is
    # legible and never silently continues.
    function Invoke-Native {
        Param(
            [Parameter(Mandatory=$true,Position=0)]
            [string]$Exe,
            [Parameter(Mandatory=$false,ValueFromRemainingArguments=$true)]
            [string[]]$Arguments = @()
        )
        Write-Log ("run: $Exe " + ($Arguments -join ' ')) 'Info'
        & $Exe @Arguments
        if ($LASTEXITCODE -ne 0) {
            throw "command failed (exit $LASTEXITCODE): $Exe $($Arguments -join ' ')"
        }
    }

    # The current workspace version: the first version key in the root manifest,
    # which sits under [workspace.package].
    function Get-WorkspaceVersion {
        $line = Select-String -LiteralPath (Join-Path $RepoRoot 'Cargo.toml') -Pattern '^version' |
            Select-Object -First 1
        if ($line -and $line.Line -match '"([^"]+)"') { return $Matches[1] }
        return ''
    }

    # Compute the target version from the current one and a level. An explicit
    # X.Y.Z is returned unchanged.
    function Get-SemverBump {
        Param([string]$Old,[string]$BumpLevel)
        if ($BumpLevel -match '^\d+\.\d+\.\d+$') { return $BumpLevel }
        $parts = $Old.Split('.')
        $major = [int]$parts[0]; $minor = [int]$parts[1]; $patch = [int]$parts[2]
        switch ($BumpLevel) {
            'major' { return "$($major + 1).0.0" }
            'minor' { return "$major.$($minor + 1).0" }
            'patch' { return "$major.$minor.$($patch + 1)" }
            default { throw "invalid level or version: $BumpLevel" }
        }
    }

    # Verify the tools and repository state a real cut requires. Exits 2 when a
    # tool is absent, 1 when a precondition is unmet.
    function Test-Preflight {
        if (-not (Get-Command git -ErrorAction SilentlyContinue) -or
            -not (Get-Command cargo -ErrorAction SilentlyContinue)) {
            Write-Log 'git and cargo are required' 'Error'; exit 2
        }
        & cargo release --version *> $null
        if ($LASTEXITCODE -ne 0) {
            Write-Log 'cargo-release is required (install: cargo install cargo-release)' 'Error'
            exit 2
        }
        $branch = (& git rev-parse --abbrev-ref HEAD).Trim()
        if ($branch -ne 'main') {
            Write-Log "must be on main to cut a release (on: $branch)" 'Error'; exit 1
        }
        if (& git status --porcelain) {
            Write-Log 'working tree is not clean; commit or stash first' 'Error'; exit 1
        }
        Invoke-Native git fetch --quiet origin main
        if ((& git rev-parse HEAD).Trim() -ne (& git rev-parse origin/main).Trim()) {
            Write-Log 'local main is not in sync with origin/main; pull or push first' 'Error'; exit 1
        }
    }

    # Replace the embedded version string fragcap/<old> with fragcap/<new> in the
    # two source assertions the bump moves. The profile-format comment
    # (fragcap:profile=...) uses a colon, not a slash, and is intentionally left
    # alone: it versions the profile embedding, not the release.
    function Update-EmbeddedVersion {
        Param([string]$Old,[string]$New)
        $files = @(
            (Join-Path $RepoRoot 'crates/fragcap-sink/src/pcapng/mod.rs'),
            (Join-Path $RepoRoot 'crates/fragcap-sink/src/json/mod.rs')
        )
        foreach ($file in $files) {
            $text = [System.IO.File]::ReadAllText($file)
            $text = $text.Replace("fragcap/$Old", "fragcap/$New")
            [System.IO.File]::WriteAllText($file, $text)
            Write-Log "updated embedded version in $file" 'Info'
        }
        # Applies-To moves with the workspace version. It is bound to that version
        # by cargo xtask spec (constitution P-11), which runs in the check set, so
        # leaving it stale would fail every release preparation deterministically.
        $spec = Join-Path $RepoRoot 'docs/fragcap-specification.md'
        $text = [System.IO.File]::ReadAllText($spec)
        $text = [regex]::Replace($text, '(?m)^\*\*Applies-To:\*\* [0-9][0-9.]*', "**Applies-To:** $New")
        [System.IO.File]::WriteAllText($spec, $text)
        Write-Log "updated Applies-To in $spec" 'Info'
    }

    # Print the sequence the operator runs after this script, so the two
    # remaining authorizations are never a surprise.
    function Write-NextSteps {
        Param([string]$Version)
        $steps = @"

Next steps (each is a deliberate, authorized act this script does not perform):

  1. Review the release/$Version branch, then open a pull request:
       git push -u origin release/$Version
       gh pr create --fill

  2. After the operator merges it, tag the release from main:
       git switch main; git pull
       git tag v$Version; git push origin v$Version

  3. The release workflow builds artifacts and creates the GitHub release, then
     the publish job waits on the crates-io environment. Approve it in GitHub to
     publish the eight crates.
"@
        [Console]::Error.WriteLine($steps)
    }

#_______________________________________________________________________________
## Declare Variables and Arrays

    $ThisScriptPath = $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
    $script:LogQuiet = [bool]$Quiet
    $script:LogSilent = [bool]$Silent
    $script:LogNoColor = ($NoColor -or ($null -ne $env:NO_COLOR))
    $ErrorActionPreference = 'Stop'

#_______________________________________________________________________________
## Execute Operations

    # Catch help text requests first, before any work. Get-Help resolves a
    # script's comment-based help by its full path (there is no -LiteralPath on
    # Get-Help, and ErrorActionPreference is Stop here, so the path must resolve).
    if (($Help) -or ($PSCmdlet.ParameterSetName -eq 'HelpText')) {
        Get-Help -Name (Resolve-Path -LiteralPath $ThisScriptPath).Path -Detailed
        exit 0
    }

    Assert-PSVersion

    $oldVersion = Get-WorkspaceVersion
    if (-not $oldVersion) {
        Write-Log 'could not read the workspace version from Cargo.toml' 'Error'; exit 2
    }
    try {
        $targetVersion = Get-SemverBump -Old $oldVersion -BumpLevel $Level
    } catch {
        Write-Log $_.Exception.Message 'Error'; exit 2
    }
    $releaseDate = if ($Date) { $Date } else { (Get-Date -Format 'yyyy-MM-dd') }

    # Reject a malformed version or date here, before creating a branch or
    # writing anything. Get-SemverBump already rejects a bad explicit version,
    # but a -Date typo would otherwise flow through to the changelog heading.
    # cargo xtask changelog --release validates the same two fields again before
    # it consumes fragments; this guard is the earlier of the two.
    if ($targetVersion -notmatch '^\d+\.\d+\.\d+$') {
        Write-Log "invalid target version: $targetVersion (expected X.Y.Z)" 'Error'; exit 2
    }
    if ($releaseDate -notmatch '^\d{4}-\d{2}-\d{2}$') {
        Write-Log "invalid date: $releaseDate (expected YYYY-MM-DD)" 'Error'; exit 2
    }

    Write-Log "current version: $oldVersion" 'Info'
    Write-Log "target version:  $targetVersion" 'Info'
    Write-Log "release date:    $releaseDate" 'Info'

    if ($DryRun) {
        Write-Log 'dry run: nothing will be created, bumped, or written' 'Warn'
        Write-Log "would create branch: release/$targetVersion" 'Info'
        Write-Log "would run: cargo release $targetVersion --workspace --execute --no-confirm" 'Info'
        Write-Log 'changelog preview (cargo xtask changelog --check):' 'Info'
        Push-Location $RepoRoot
        try {
            & cargo run --quiet --package xtask -- changelog --check
            if ($LASTEXITCODE -ne 0) { Write-Log 'changelog preview failed' 'Error'; exit 1 }
        } finally { Pop-Location }
        Write-NextSteps -Version $targetVersion
        Write-Log 'dry run complete' 'Info'
        exit 0
    }

    Test-Preflight

    Push-Location $RepoRoot
    try {
        Invoke-Native git switch -c "release/$targetVersion"

        # Bump and commit the version. release.toml pins this to move the number
        # only: no tag, no push, no publish.
        Invoke-Native cargo release $targetVersion --workspace --execute --no-confirm

        $newVersion = Get-WorkspaceVersion
        if ($newVersion -ne $targetVersion) {
            throw "version after bump is $newVersion, expected $targetVersion"
        }

        # The bump moved fragcap/<version>, in two assertions and every golden.
        Update-EmbeddedVersion -Old $oldVersion -New $targetVersion
        # Target the three regenerating test binaries specifically rather than
        # the whole workspace: the corpus conservation checks read these same
        # goldens and refuse to regenerate on principle, so running them in the
        # same pass would race the rewrite and fail.
        #
        # The three own every golden carrying the embedded fragcap/<version>
        # string: fragcap's goldens binary owns the fixture corpus,
        # fragcap-cli's cli_capture owns capture.fcapng and capture.jsonl, and
        # its cli_extcap owns run.fcapng. Keep this list in step with the
        # goldens; a binary named here that no longer exists fails the cut, and
        # one omitted leaves a stale version in a golden.
        Write-Log 'regenerating the golden corpus for the new version' 'Info'
        $env:FRAGCAP_UPDATE_GOLDENS = '1'
        try {
            Invoke-Native -Exe cargo -Arguments @('test', '-p', 'fragcap', '--test', 'goldens', '--quiet')
            Invoke-Native -Exe cargo -Arguments @('test', '-p', 'fragcap-cli', '--test', 'cli_capture', '--quiet')
            Invoke-Native -Exe cargo -Arguments @('test', '-p', 'fragcap-cli', '--test', 'cli_extcap', '--quiet')
        } finally {
            Remove-Item Env:FRAGCAP_UPDATE_GOLDENS -ErrorAction SilentlyContinue
        }

        # Assemble the changelog and fold everything into the one release commit.
        Invoke-Native cargo run --quiet --package xtask -- changelog --release $targetVersion $releaseDate
        Invoke-Native git add -A
        Invoke-Native git commit --amend --no-edit

        Write-Log 'running the full check set (cargo xtask ci)' 'Info'
        & cargo xtask ci
        if ($LASTEXITCODE -ne 0) {
            Write-Log "cargo xtask ci failed on release/$targetVersion; the branch is left for inspection" 'Error'
            exit 1
        }

        Write-Log "prepared release/$targetVersion" 'Info'
        Write-NextSteps -Version $targetVersion
    } catch {
        Write-Log $_.Exception.Message 'Error'
        exit 1
    } finally {
        Pop-Location
    }

#_______________________________________________________________________________
## End of script
