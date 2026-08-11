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

    # Assemble the option list once: the profile, the expanded output path, the
    # --json event stream this wrapper consumes, and any passed-through options.
    $outPath = $null
    if ($Out) {
        $outPath = Expand-Template -Template $Out
    }
    $options = [System.Collections.Generic.List[string]]::new()
    $options.Add('run')
    $options.Add('--profile')
    $options.Add($Profile)
    if ($outPath) {
        $options.Add('--out')
        $options.Add($outPath)
    }
    $options.Add('--json')
    if ($Passthrough) {
        foreach ($item in $Passthrough) { $options.Add($item) }
    }

    # The dry-run seam prints the logical invocation and exits, with no elevation,
    # driver detection, or capture.
    if ($DryRun) {
        Write-Output ('fragcap ' + ($options -join ' '))
        exit 0
    }

    # Elevation: relaunch elevated when the session is not. The child is rebuilt
    # from the bound parameters (an elevated `$args` does not carry values already
    # bound to declared parameters), waited on, and its exit code propagated.
    if (-not (Test-Elevated)) {
        Write-Log 'session is not elevated; relaunching elevated' 'Info'
        $childArgs = [System.Collections.Generic.List[string]]::new()
        $childArgs.Add('-NoProfile')
        $childArgs.Add('-File')
        $childArgs.Add($ThisScriptPath)
        $childArgs.Add('-Profile')
        $childArgs.Add($Profile)
        if ($Out)     { $childArgs.Add('-Out'); $childArgs.Add($Out) }
        if ($Quiet)   { $childArgs.Add('-Quiet') }
        if ($Silent)  { $childArgs.Add('-Silent') }
        if ($NoColor) { $childArgs.Add('-NoColor') }
        if ($Passthrough) { foreach ($item in $Passthrough) { $childArgs.Add($item) } }
        try {
            $child = Start-Process -FilePath 'pwsh' -Verb RunAs `
                -ArgumentList $childArgs.ToArray() -Wait -PassThru -ErrorAction Stop
            exit $child.ExitCode
        } catch {
            Write-Log 'elevation was declined; cannot capture without it' 'Error'
            exit 2
        }
    }

    # Driver detection, read-only. The capture driver's own wpcap.dll lives in the
    # Npcap directory; its absence means capture is not possible, and its version
    # is reported so an unsuitable installation is distinguishable from a good one.
    $npcapDll = Join-Path -Path $env:SystemRoot -ChildPath 'System32\Npcap\wpcap.dll'
    if (-not (Test-Path -LiteralPath $npcapDll)) {
        Write-Log "the capture driver is not installed; download it from $NpcapUrl" 'Error'
        exit 1
    }
    $driverVersion = (Get-Item -LiteralPath $npcapDll).VersionInfo.ProductVersion
    if (-not $driverVersion) {
        $driverVersion = (Get-Item -LiteralPath $npcapDll).VersionInfo.FileVersion
    }
    Write-Log "capture driver present (npcap wpcap.dll version $driverVersion)" 'Info'

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

    # Resolve the executable: prefer a fragcap.exe bundled beside the wrapper in
    # the release archive (the wrapper lives under scripts/, the binary at the
    # archive root), then fall back to fragcap on the PATH.
    $binary = 'fragcap'
    $bundled = Join-Path -Path $PSScriptRoot -ChildPath '..\fragcap.exe'
    if (Test-Path -LiteralPath $bundled) {
        $binary = (Resolve-Path -LiteralPath $bundled).Path
    }

    Write-Log ("invoking: $binary " + ($options -join ' ')) 'Info'
    & $binary @($options.ToArray())
    exit $LASTEXITCODE

#_______________________________________________________________________________
## End of script
