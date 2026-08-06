<#
.SYNOPSIS
    Stop the local dev server listening on a given port.

.DESCRIPTION
    Finds the process that owns the TCP listener on the target port and
    terminates it. This is a destructive action, so the script declares
    SupportsShouldProcess and gates the Stop-Process call behind
    $PSCmdlet.ShouldProcess, giving the operator -WhatIf and -Confirm.

    This is the stop half of a start/stop pair (see Start-LocalDevServer.ps1).
    Both default to the same port and accept a -Port override.

    If nothing is listening on the port the script treats that as already
    stopped and exits successfully.

    Exit codes: 0 the listener was stopped or was already absent, 1 a process
    was found but could not be terminated.

.PARAMETER Port
    TCP port whose listener should be stopped.
    Default: 8787.
    Alias: p

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
    .\Stop-LocalDevServer.ps1
    Stops whatever is listening on port 8787, prompting before it terminates
    the process (Medium impact).

.EXAMPLE
    .\Stop-LocalDevServer.ps1 -Port 3000 -WhatIf
    Shows which process would be terminated for port 3000 without terminating
    it.

.NOTES
    Operator-facing tooling. Demonstrates a Medium-impact destructive action
    gated by ShouldProcess, per the ShruggieTech PowerShell standard.
#>
[CmdletBinding(SupportsShouldProcess=$true,ConfirmImpact='Medium',DefaultParameterSetName='Default')]
Param(
    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("p")]
    [ValidateRange(1, 65535)]
    [int]$Port = 8787,

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

    function Get-ListenerProcessId {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [int]$ListenPort
        )
        try {
            $conns = Get-NetTCPConnection -State Listen -LocalPort $ListenPort -ErrorAction Stop
            return ($conns | Select-Object -ExpandProperty OwningProcess -Unique)
        } catch {
            return @()
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

    # Wire suppression flags to the logging state
    if ($Quiet)  { $script:LogQuiet  = $true }
    if ($Silent) { $script:LogSilent = $true }

    $pids = @(Get-ListenerProcessId -ListenPort $Port)

    if ($pids.Count -eq 0) {
        Write-Log ("Nothing listening on port {0}. Already stopped." -f $Port) -Level Success -Source 'Stop'
        exit 0
    }

    $failed = 0
    foreach ($processId in $pids) {
        $proc = Get-Process -Id $processId -ErrorAction SilentlyContinue
        $name = if ($proc) { $proc.ProcessName } else { 'unknown' }
        $target = ("PID {0} ({1}) on port {2}" -f $processId, $name, $Port)
        if ($PSCmdlet.ShouldProcess($target, 'Stop process')) {
            try {
                Stop-Process -Id $processId -Force -ErrorAction Stop
                Write-Log ("Stopped {0}" -f $target) -Level Success -Source 'Stop'
            } catch {
                $failed++
                Write-Log ("Could not stop {0}: {1}" -f $target, $_.Exception.Message) -Level Error -Source 'Stop'
            }
        }
    }

    if ($failed -gt 0) {
        exit 1
    }

    exit 0

#_______________________________________________________________________________
## End of script
