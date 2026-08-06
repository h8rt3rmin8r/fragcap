<#
.SYNOPSIS
    Start a local dev server in a separate window and wait until it is healthy.

.DESCRIPTION
    Spawns a long-running dev-server process in its own window so the operator's
    prompt stays usable, then polls a health endpoint on the target port until
    the server responds or a timeout elapses. On success it prints next-step
    guidance and returns control; on a failed health probe it prints an
    attributed, actionable failure line and exits with the environment
    precondition code.

    This is the start half of a start/stop pair (see Stop-LocalDevServer.ps1).
    Both default to the same port and accept a -Port override.

    Uses Invoke-WebRequest -SkipHttpErrorCheck, which exists only on PowerShell
    7+, so the script guards its engine version with Assert-PSVersion before
    doing any work.

    Exit codes: 0 the server became healthy, 2 the server did not respond
    within the timeout (environment precondition).

.PARAMETER Port
    TCP port the dev server listens on.
    Default: 8787.
    Alias: p

.PARAMETER Command
    Command line to launch the dev server. Run in a new pwsh window.
    Default: 'npm run dev'.
    Alias: c

.PARAMETER HealthPath
    Request path appended to http://localhost:<Port> when polling for health.
    Default: '/'.

.PARAMETER TimeoutSeconds
    How long to poll before giving up.
    Default: 60.
    Alias: t

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
    .\Start-LocalDevServer.ps1
    Launches 'npm run dev' in a new window and waits for http://localhost:8787/
    to respond.

.EXAMPLE
    .\Start-LocalDevServer.ps1 -Port 3000 -Command 'pnpm dev' -HealthPath '/api/health'
    Launches 'pnpm dev' and polls http://localhost:3000/api/health until ready.

.NOTES
    Operator-facing tooling. Reserves Write-Error for script-internal failures
    and uses attributed Write-Host for environmental failures, per the
    ShruggieTech PowerShell standard.
#>
[CmdletBinding(SupportsShouldProcess=$false,ConfirmImpact='None',DefaultParameterSetName='Default')]
Param(
    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("p")]
    [ValidateRange(1, 65535)]
    [int]$Port = 8787,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("c")]
    [string]$Command = 'npm run dev',

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [string]$HealthPath = '/',

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("t")]
    [ValidateRange(1, 3600)]
    [int]$TimeoutSeconds = 60,

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

    function Test-Health {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$Url
        )
        try {
            $resp = Invoke-WebRequest -Uri $Url -SkipHttpErrorCheck -TimeoutSec 5 -UseBasicParsing
            return ($resp.StatusCode -lt 500)
        } catch {
            return $false
        }
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

    # Enforce the minimum engine version (this script uses 7+-only features)
    Assert-PSVersion -Minimum '7.0'

    # Wire suppression flags to the logging state
    if ($Quiet)  { $script:LogQuiet  = $true }
    if ($Silent) { $script:LogSilent = $true }

    $healthUrl = "http://localhost:$Port$HealthPath"

    Write-Host ""
    Write-Host "  Local Dev Server" -ForegroundColor Cyan
    Write-Host ("  port {0}  command '{1}'  health {2}" -f $Port, $Command, $healthUrl) -ForegroundColor DarkGray
    Write-Host ""

    # Spawn the dev server in its own window so this prompt stays usable
    Write-Log ("Launching: {0}" -f $Command) -Level Info -Source 'Spawn'
    Start-Process -FilePath 'pwsh' -ArgumentList @('-NoExit', '-Command', $Command) | Out-Null

    # Poll the health endpoint until ready or timeout
    Write-Log ("Waiting up to {0}s for {1}" -f $TimeoutSeconds, $healthUrl) -Level Info -Source 'HealthPoll'
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    $healthy = $false
    while ((Get-Date) -lt $deadline) {
        if (Test-Health -Url $healthUrl) {
            $healthy = $true
            break
        }
        Start-Sleep -Seconds 1
    }

    if (-not $healthy) {
        Write-Host ("FAIL: server did not respond on {0} within {1}s. Is the command correct and the port free?" -f $healthUrl, $TimeoutSeconds) -ForegroundColor Red
        exit 2
    }

    Write-Log ("Server healthy on {0}" -f $healthUrl) -Level Success -Source 'HealthPoll'
    Write-Host ""
    Write-Host ("  Ready. Open http://localhost:{0}{1}" -f $Port, $HealthPath) -ForegroundColor Green
    Write-Host ("  Stop it with: .\Stop-LocalDevServer.ps1 -Port {0}" -f $Port) -ForegroundColor DarkGray
    Write-Host ""

    exit 0

#_______________________________________________________________________________
## End of script
