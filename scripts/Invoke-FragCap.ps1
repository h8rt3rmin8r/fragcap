# SPDX-License-Identifier: Apache-2.0
<#
.SYNOPSIS
    Shell wrapper for the fragcap capture tool on Windows.

.DESCRIPTION
    A thin PowerShell wrapper around the native fragcap binary (specification
    section 18.2). It handles the environment concerns that belong outside the
    binary: it verifies the session is elevated and relaunches itself elevated
    when it is not, it detects whether the capture driver is installed and reports
    where to download it when it is absent, it enumerates interfaces and filters
    virtual adapters from the presented list, and it expands an output-path
    template and prepares the output directory.

    The wrapper contains no capture logic and does not parse fragcap's
    human-readable output. It reacts to the structured event stream fragcap emits
    under --json (specification section 17.5), which this wrapper adds to every
    invocation. Unrecognized options are passed through to fragcap unchanged.

    It installs, downloads, and modifies nothing about the capture driver:
    detection only (constitution P-1, the Licensing rule).

.PARAMETER Profile
    The profile to capture with.
    Alias: p

.PARAMETER Out
    Output-path template. The tokens {profile}, {date}, and {time} are expanded
    before capture, and the target directory is prepared.
    Alias: o

.PARAMETER DryRun
    Print the assembled fragcap invocation and exit without capturing, elevating,
    or detecting the driver.
    Alias: d

.PARAMETER Quiet
    Suppress informational output; keep warnings and errors.
    Alias: q

.PARAMETER Silent
    Suppress warnings too; errors still emit.

.PARAMETER NoColor
    Disable colored output.

.PARAMETER Passthrough
    Any further options, passed through to fragcap unchanged.

.PARAMETER Help
    Print this help text and exit.
    Alias: h

.EXAMPLE
    .\Invoke-FragCap.ps1 -Profile eso -Out "caps\{profile}-{date}.fcapng"
    Capture with a templated output path, elevating and checking the driver.

.EXAMPLE
    .\Invoke-FragCap.ps1 -DryRun -Profile eso -Out "caps\{profile}.fcapng"
    Preview the assembled fragcap invocation without capturing.
#>
[CmdletBinding(SupportsShouldProcess=$false,ConfirmImpact='None',DefaultParameterSetName='Default')]
Param(
    [Parameter(Mandatory=$true,ParameterSetName='Default')]
    [Alias("p")]
    [string]$Profile,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("o")]
    [string]$Out,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("d")]
    [Switch]$DryRun,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("q")]
    [Switch]$Quiet,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Switch]$Silent,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Switch]$NoColor,

    [Parameter(Mandatory=$false,ParameterSetName='Default',ValueFromRemainingArguments=$true)]
    [string[]]$Passthrough,

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

    # Expand the {profile}, {date}, and {time} tokens in an output template.
    function Expand-Template {
        Param([string]$Template)
        $result = $Template
        $result = $result.Replace('{profile}', $Profile)
        $result = $result.Replace('{date}', (Get-Date -Format 'yyyy-MM-dd'))
        $result = $result.Replace('{time}', (Get-Date -Format 'HHmmss'))
        return $result
    }

    # Whether the current session is elevated.
    function Test-Elevated {
        $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = New-Object Security.Principal.WindowsPrincipal($identity)
        return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    }

#_______________________________________________________________________________
## Declare Variables and Arrays

    $ThisScriptPath = $MyInvocation.MyCommand.Path
    $script:LogQuiet = [bool]$Quiet
    $script:LogSilent = [bool]$Silent
    $script:LogNoColor = ($NoColor -or ($null -ne $env:NO_COLOR))

    # The download location reported when the capture driver is absent. This
    # wrapper installs nothing (constitution P-1, the Licensing rule).
    $NpcapUrl = 'https://npcap.com/#download'

#_______________________________________________________________________________
## Execute Operations

    # Catch help text requests first, before any work.
    if (($Help) -or ($PSCmdlet.ParameterSetName -eq 'HelpText')) {
        Get-Help -LiteralPath $ThisScriptPath -Detailed
        exit 0
    }

    # Assemble the fragcap invocation: the run subcommand, the profile, the
    # expanded output path, the --json event stream this wrapper consumes, and
    # any passed-through options.
    $command = [System.Collections.Generic.List[string]]::new()
    $command.Add('fragcap')
    $command.Add('run')
    $command.Add('--profile')
    $command.Add($Profile)
    $outPath = $null
    if ($Out) {
        $outPath = Expand-Template -Template $Out
        $command.Add('--out')
        $command.Add($outPath)
    }
    $command.Add('--json')
    if ($Passthrough) {
        foreach ($item in $Passthrough) { $command.Add($item) }
    }

    # The dry-run seam prints the assembled invocation and exits, with no
    # elevation, driver detection, or capture.
    if ($DryRun) {
        Write-Output ($command -join ' ')
        exit 0
    }

    # Elevation: relaunch elevated when the session is not, preserving arguments.
    if (-not (Test-Elevated)) {
        Write-Log 'session is not elevated; relaunching elevated' 'Info'
        try {
            $argList = @('-NoProfile', '-File', $ThisScriptPath) + $args
            Start-Process -FilePath 'pwsh' -Verb RunAs -ArgumentList $argList -ErrorAction Stop
            exit 0
        } catch {
            Write-Log 'elevation was declined; cannot capture without it' 'Error'
            exit 2
        }
    }

    # Driver detection, read-only. The capture driver's own wpcap.dll lives in the
    # Npcap directory; its absence means capture is not possible.
    $npcapDll = Join-Path -Path $env:SystemRoot -ChildPath 'System32\Npcap\wpcap.dll'
    if (-not (Test-Path -LiteralPath $npcapDll)) {
        Write-Log "the capture driver is not installed; download it from $NpcapUrl" 'Error'
        exit 1
    }

    # Interface enumeration assistance: filter virtual adapters from the list.
    try {
        $adapters = Get-NetAdapter -ErrorAction Stop |
            Where-Object { -not $_.Virtual } |
            Select-Object -ExpandProperty Name
        Write-Log ("interfaces: " + ($adapters -join ', ')) 'Info'
    } catch {
        Write-Log 'could not enumerate interfaces; fragcap will select its own' 'Warn'
    }

    # Prepare the output directory before capture.
    if ($outPath) {
        $dir = Split-Path -LiteralPath $outPath -Parent
        if ($dir -and -not (Test-Path -LiteralPath $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
        }
    }

    Write-Log ("invoking: " + ($command -join ' ')) 'Info'
    & $command[0] @($command[1..($command.Count - 1)])
    exit $LASTEXITCODE

#_______________________________________________________________________________
## End of script
