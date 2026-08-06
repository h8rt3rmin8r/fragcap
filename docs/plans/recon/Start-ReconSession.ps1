# SPDX-License-Identifier: Apache-2.0
<#
.SYNOPSIS
    Record a fragcap reconnaissance session for one focal title.

.DESCRIPTION
    Starts the four recorders described in docs/plans/reconnaissance.md and runs
    them until interrupted, so that open questions Q-1 through Q-6 can be
    answered from the resulting artifacts. Start this BEFORE launching the
    platform client, and stop it after the game has exited cleanly.

    Four recorders run concurrently:

      1. Process tree      ETW-backed process start and stop events, enriched
                           with image path and command line, so the launch chain
                           is recorded at creation time rather than
                           reconstructed afterward. Reconstruction does not
                           work; that is the problem fragcap exists to solve.
      2. Primary capture   dumpcap on the selected network adapter.
      3. Loopback capture  dumpcap on the npcap loopback adapter, which is what
                           answers Q-5.
      4. Socket table      Delta-encoded snapshots of the TCP and UDP socket
                           tables, joined against packets later to answer Q-1
                           and Q-3.

    The socket sampler calls GetExtendedTcpTable and GetExtendedUdpTable
    directly through P/Invoke rather than using Get-NetTCPConnection. This is
    deliberate and load-bearing: the CIM path costs 1400 to 2000 milliseconds
    per snapshot on a busy machine, while the direct call costs 1 to 3
    milliseconds. The direct call is also the mechanism fragcap itself will use,
    so measuring against it measures the real thing.

    Snapshots are delta encoded. A socket is written once when first observed
    and once when it disappears, rather than on every sample, which keeps the
    log small enough to reason about while still bracketing every socket
    lifetime to within one poll interval.

    PRIVACY. The artifacts contain addresses, and for some titles session
    identifiers. They are written under a gitignored directory and MUST NOT be
    committed. Only derived findings go into specification Appendix D, scrubbed
    of account identifiers, session tokens, and operator-attributable
    addresses.

    REQUIREMENTS. Administrative privilege is required for the process tree
    recorder. npcap must be installed with loopback traffic capture support and
    WinPcap API compatible mode; the script verifies both and exits 2 if either
    is missing. Wireshark supplies dumpcap.

    This script observes only. It performs no action on the constitution
    denylist: packet capture, socket table enumeration, and process enumeration
    are all read-only system queries.

.PARAMETER Title
    Short identifier for the focal title being recorded. Becomes part of the
    output directory name and is recorded in the session manifest.
    Default: 'eso'.
    Alias: t

.PARAMETER Interface
    Capture interface for the primary adapter, as reported by 'dumpcap -D'.
    Accepts the device path or the interface number. When omitted the script
    selects the connected adapter carrying a default route.
    Alias: i

.PARAMETER IntervalMs
    Socket table poll interval in milliseconds. The direct P/Invoke path
    sustains far lower values than the CIM path, so the default is aggressive
    on purpose: a shorter interval bounds the race window in specification
    section 11.3 more tightly and produces a better Q-3 measurement.
    Default: 250.
    Alias: n

.PARAMETER SnapLength
    Bytes captured per packet. The default keeps headers plus enough payload to
    characterize encryption posture for Q-6, while keeping capture files small.
    Pass 0 to capture full packets.
    Default: 262.
    Alias: s

.PARAMETER OutputRoot
    Directory under which the timestamped session directory is created.
    Default: 'captures/recon' relative to the repository root, which is
    gitignored.
    Alias: o

.PARAMETER DurationMinutes
    Stop automatically after this many minutes. Zero means run until
    interrupted with Ctrl+C, which is the normal mode.
    Default: 0.
    Alias: d

.PARAMETER Quiet
    Suppress informational output. Warnings and errors still emit.
    Alias: q

.PARAMETER Silent
    Suppress all log output including warnings. Genuine errors still reach the
    error stream.

.PARAMETER Help
    Print this help text to the terminal.
    Alias: h

.EXAMPLE
    .\Start-ReconSession.ps1 -Title eso
    Record a session for The Elder Scrolls Online. Start this, then launch
    Steam, then play. Press Ctrl+C after the game has exited.

.EXAMPLE
    .\Start-ReconSession.ps1 -Title div2 -IntervalMs 100
    Record The Division 2 with a tighter socket poll, for a more precise
    connection lifetime distribution.

.EXAMPLE
    .\Start-ReconSession.ps1 -Title eso -SnapLength 0 -Interface 8
    Capture full packet payloads on interface 8. Produces much larger files;
    use when payload inspection beyond the first 262 bytes is needed.

.NOTES
    Exit codes: 0 success, 1 a recorder failed, 2 environment precondition
    failure. See docs/plans/reconnaissance.md for what to do with the output.
#>
[CmdletBinding(SupportsShouldProcess=$false,ConfirmImpact='None',DefaultParameterSetName='Default')]
Param(
    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("t")]
    [string]$Title = 'eso',

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("i")]
    [string]$Interface = '',

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("n")]
    [ValidateRange(25,10000)]
    [int]$IntervalMs = 250,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("s")]
    [ValidateRange(0,65535)]
    [int]$SnapLength = 262,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("o")]
    [string]$OutputRoot = '',

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("d")]
    [ValidateRange(0,1440)]
    [int]$DurationMinutes = 0,

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

    function Assert-Environment {
        [CmdletBinding()]
        Param()

        # Elevation is no longer required. Capture works unprivileged when npcap
        # is installed with AdminOnly = 0 (PF-2), and the process recorder now
        # falls back to a polling source that needs no privilege. Elevation only
        # buys the ETW source, which is more precise, so it is reported rather
        # than demanded.
        $identity  = [Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = [Security.Principal.WindowsPrincipal]$identity
        $script:IsElevated = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

        $npcap = Get-ItemProperty 'HKLM:\SOFTWARE\WOW6432Node\Npcap' -ErrorAction SilentlyContinue
        if (-not $npcap) {
            $npcap = Get-ItemProperty 'HKLM:\SOFTWARE\Npcap' -ErrorAction SilentlyContinue
        }
        if (-not $npcap) {
            Write-Host "FAIL: npcap is not installed. Install it from https://npcap.com" -ForegroundColor Red
            exit 2
        }
        if ($npcap.WinPcapCompatible -ne 1) {
            Write-Host "FAIL: npcap lacks WinPcap API compatible mode. Reinstall npcap with 'Install Npcap in WinPcap API-compatible Mode' checked." -ForegroundColor Red
            exit 2
        }

        if (-not (Test-Path -LiteralPath $script:DumpcapPath)) {
            Write-Host "FAIL: dumpcap not found at $script:DumpcapPath. Install Wireshark." -ForegroundColor Red
            exit 2
        }

        # dumpcap -D returns an array. Use -not (array -match ...) rather than
        # (array -notmatch ...): the latter is a filter that returns the
        # non-matching elements, so it is truthy whenever any line differs,
        # which is always.
        $interfaces = @(& $script:DumpcapPath -D 2>&1)
        if (-not ($interfaces -match 'NPF_Loopback')) {
            Write-Host "FAIL: no npcap loopback adapter. Reinstall npcap with 'Support loopback traffic capture' checked. Q-5 cannot be answered without it." -ForegroundColor Red
            exit 2
        }

        return $interfaces
    }

    function Get-DefaultInterface {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string[]]$InterfaceList
        )

        $route = Get-NetRoute -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue |
                 Sort-Object RouteMetric |
                 Select-Object -First 1
        if (-not $route) { return $null }

        $adapter = Get-NetAdapter -InterfaceIndex $route.ifIndex -ErrorAction SilentlyContinue
        if (-not $adapter) { return $null }

        foreach ($line in $InterfaceList) {
            if ($line -match '^\s*\d+\.\s+(\S+)\s+\((.+)\)\s*$') {
                if ($Matches[2] -eq $adapter.Name) { return $Matches[1] }
            }
        }
        return $null
    }

    function Add-SocketSampler {
        [CmdletBinding()]
        Param()

        if ('FragcapRecon.SocketTable' -as [type]) { return }

        Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Net;
using System.Runtime.InteropServices;

namespace FragcapRecon {
  public static class SocketTable {
    [DllImport("iphlpapi.dll", SetLastError=true)]
    static extern uint GetExtendedTcpTable(IntPtr t, ref int size, bool sort, int af, int cls, int res);
    [DllImport("iphlpapi.dll", SetLastError=true)]
    static extern uint GetExtendedUdpTable(IntPtr t, ref int size, bool sort, int af, int cls, int res);

    const int AF_INET = 2, AF_INET6 = 23;
    const int TCP_OWNER_PID = 5, UDP_OWNER_PID = 1;

    static readonly string[] States = {
      "?", "CLOSED", "LISTEN", "SYN_SENT", "SYN_RCVD", "ESTABLISHED",
      "FIN_WAIT1", "FIN_WAIT2", "CLOSE_WAIT", "CLOSING", "LAST_ACK",
      "TIME_WAIT", "DELETE_TCB"
    };

    static int Port(int netOrder) {
      return ((netOrder & 0xFF) << 8) | ((netOrder >> 8) & 0xFF);
    }
    static string V4(uint a) { return new IPAddress((long)a).ToString(); }
    static string V6(byte[] a, uint scope) {
      return new IPAddress(a, scope).ToString();
    }
    static string StateName(int s) {
      return (s >= 0 && s < States.Length) ? States[s] : s.ToString();
    }

    // Rows are emitted as pipe-delimited fields so the caller does no
    // structure marshalling:
    //   proto|localAddr|localPort|remoteAddr|remotePort|state|pid
    public static List<string> Snapshot() {
      var rows = new List<string>(4096);
      ReadTcp(AF_INET, rows);
      ReadTcp(AF_INET6, rows);
      ReadUdp(AF_INET, rows);
      ReadUdp(AF_INET6, rows);
      return rows;
    }

    static IntPtr Fetch(bool tcp, int af, out int count, out int stride) {
      int size = 0;
      int cls = tcp ? TCP_OWNER_PID : UDP_OWNER_PID;
      if (tcp) { GetExtendedTcpTable(IntPtr.Zero, ref size, false, af, cls, 0); }
      else     { GetExtendedUdpTable(IntPtr.Zero, ref size, false, af, cls, 0); }
      IntPtr buf = Marshal.AllocHGlobal(size);
      uint rc = tcp
        ? GetExtendedTcpTable(buf, ref size, false, af, cls, 0)
        : GetExtendedUdpTable(buf, ref size, false, af, cls, 0);
      if (rc != 0) { Marshal.FreeHGlobal(buf); count = 0; stride = 0; return IntPtr.Zero; }
      count = Marshal.ReadInt32(buf);
      if (tcp) { stride = (af == AF_INET) ? 24 : 56; }
      else     { stride = (af == AF_INET) ? 12 : 28; }
      return buf;
    }

    static void ReadTcp(int af, List<string> rows) {
      int count, stride;
      IntPtr buf = Fetch(true, af, out count, out stride);
      if (buf == IntPtr.Zero) return;
      try {
        IntPtr p = new IntPtr(buf.ToInt64() + 4);
        for (int i = 0; i < count; i++) {
          if (af == AF_INET) {
            int st = Marshal.ReadInt32(p, 0);
            string la = V4((uint)Marshal.ReadInt32(p, 4));
            int lp = Port(Marshal.ReadInt32(p, 8));
            string ra = V4((uint)Marshal.ReadInt32(p, 12));
            int rp = Port(Marshal.ReadInt32(p, 16));
            int pid = Marshal.ReadInt32(p, 20);
            rows.Add(string.Format("tcp|{0}|{1}|{2}|{3}|{4}|{5}",
              la, lp, ra, rp, StateName(st), pid));
          } else {
            byte[] lb = new byte[16]; Marshal.Copy(p, lb, 0, 16);
            uint lsc = (uint)Marshal.ReadInt32(p, 16);
            int lp = Port(Marshal.ReadInt32(p, 20));
            byte[] rb = new byte[16];
            Marshal.Copy(new IntPtr(p.ToInt64() + 24), rb, 0, 16);
            uint rsc = (uint)Marshal.ReadInt32(p, 40);
            int rp = Port(Marshal.ReadInt32(p, 44));
            int st = Marshal.ReadInt32(p, 48);
            int pid = Marshal.ReadInt32(p, 52);
            rows.Add(string.Format("tcp6|{0}|{1}|{2}|{3}|{4}|{5}",
              V6(lb, lsc), lp, V6(rb, rsc), rp, StateName(st), pid));
          }
          p = new IntPtr(p.ToInt64() + stride);
        }
      } finally { Marshal.FreeHGlobal(buf); }
    }

    static void ReadUdp(int af, List<string> rows) {
      int count, stride;
      IntPtr buf = Fetch(false, af, out count, out stride);
      if (buf == IntPtr.Zero) return;
      try {
        IntPtr p = new IntPtr(buf.ToInt64() + 4);
        for (int i = 0; i < count; i++) {
          if (af == AF_INET) {
            string la = V4((uint)Marshal.ReadInt32(p, 0));
            int lp = Port(Marshal.ReadInt32(p, 4));
            int pid = Marshal.ReadInt32(p, 8);
            // The UDP socket table exposes no remote endpoint. This is not an
            // omission in this script: GetExtendedUdpTable does not carry one,
            // so UDP attribution keys on the local endpoint alone.
            rows.Add(string.Format("udp|{0}|{1}|||{2}|{3}", la, lp, "-", pid));
          } else {
            byte[] lb = new byte[16]; Marshal.Copy(p, lb, 0, 16);
            uint lsc = (uint)Marshal.ReadInt32(p, 16);
            int lp = Port(Marshal.ReadInt32(p, 20));
            int pid = Marshal.ReadInt32(p, 24);
            rows.Add(string.Format("udp6|{0}|{1}|||{2}|{3}",
              V6(lb, lsc), lp, "-", pid));
          }
          p = new IntPtr(p.ToInt64() + stride);
        }
      } finally { Marshal.FreeHGlobal(buf); }
    }
  }
}
'@ -ErrorAction Stop
    }

#_______________________________________________________________________________
## Declare Variables and Arrays

    $script:LogQuiet   = $false
    $script:LogSilent  = $false
    $ThisScriptPath    = $MyInvocation.MyCommand.Path
    $script:DumpcapPath = 'C:\Program Files\Wireshark\dumpcap.exe'

    $script:Recorders  = @()
    $script:Subs       = @()
    $script:IsElevated = $false
    $script:SocketLog  = $null

#_______________________________________________________________________________
## Execute Operations

    # Catch help text requests
    if (($Help) -or ($PSCmdlet.ParameterSetName -eq 'HelpText')) {
        Get-Help $ThisScriptPath -Detailed
        exit 0
    }

    if ($Quiet)  { $script:LogQuiet  = $true }
    if ($Silent) { $script:LogSilent = $true }

    Assert-PSVersion -Minimum '7.0'
    $interfaceList = Assert-Environment
    Write-Log "Environment checks passed" -Level Success -Source preflight

    if (-not $Interface) {
        $Interface = Get-DefaultInterface -InterfaceList $interfaceList
        if (-not $Interface) {
            Write-Host "FAIL: could not determine the default interface. Pass -Interface explicitly; run 'dumpcap -D' to list them." -ForegroundColor Red
            exit 2
        }
        Write-Log "Selected primary interface $Interface" -Level Info -Source preflight
    }

    if (-not $OutputRoot) {
        # This script lives at <repo>/docs/plans/recon/, so reaching the
        # repository root takes four steps up from the file: recon, plans,
        # docs, root. Three steps lands in docs/ and writes sessions to
        # docs/captures/, outside the gitignore rule that protects them.
        $repoRoot   = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $ThisScriptPath)))
        $OutputRoot = Join-Path $repoRoot 'captures/recon'
    }
    $stamp     = (Get-Date).ToString('yyyyMMdd-HHmmss')
    $sessionId = "$Title-$stamp"
    $outDir    = Join-Path $OutputRoot $sessionId
    $null = New-Item -ItemType Directory -Path $outDir -Force
    Write-Log "Session directory $outDir" -Level Info -Source preflight

    Add-SocketSampler
    Write-Log "Socket sampler loaded (direct IP Helper calls)" -Level Success -Source preflight

    $startUtc = (Get-Date).ToUniversalTime()
    $manifest = [ordered]@{
        sessionId       = $sessionId
        title           = $Title
        startUtc        = $startUtc.ToString('o')
        startLocal      = (Get-Date).ToString('o')
        intervalMs      = $IntervalMs
        snapLength      = $SnapLength
        primaryInterface= $Interface
        loopbackDevice  = '\Device\NPF_Loopback'
        tshark          = (& $script:DumpcapPath -v 2>&1 | Select-Object -First 1)
        powershell      = $PSVersionTable.PSVersion.ToString()
        os              = (Get-CimInstance Win32_OperatingSystem).Version
        note            = 'Contains addresses. Do not commit. Derived findings only go to Appendix D.'
    }
    $manifest | ConvertTo-Json -Depth 4 |
        Set-Content -LiteralPath (Join-Path $outDir 'session.json') -Encoding utf8

    try {
        $snapArg = if ($SnapLength -gt 0) { @('-s', "$SnapLength") } else { @() }

        $primaryOut  = Join-Path $outDir 'primary.pcapng'
        $loopbackOut = Join-Path $outDir 'loopback.pcapng'

        $script:Recorders += Start-Process -FilePath $script:DumpcapPath -PassThru -WindowStyle Hidden `
            -ArgumentList (@('-i', $Interface, '-w', $primaryOut, '-q') + $snapArg)
        Write-Log "Primary capture started -> primary.pcapng" -Level Success -Source capture

        $script:Recorders += Start-Process -FilePath $script:DumpcapPath -PassThru -WindowStyle Hidden `
            -ArgumentList (@('-i', '\Device\NPF_Loopback', '-w', $loopbackOut, '-q') + $snapArg)
        Write-Log "Loopback capture started -> loopback.pcapng" -Level Success -Source capture

        $procPath = Join-Path $outDir 'processes.jsonl'
        [System.IO.File]::WriteAllText($procPath, '', [System.Text.UTF8Encoding]::new($false))

        # An -Action scriptblock runs in its own scope and CANNOT see this
        # script's $script: variables. A shared StreamWriter handle therefore
        # arrives as $null, every write throws into the event job's error
        # stream where nothing surfaces it, and the session completes reporting
        # success with an empty log. Pass the path through -MessageData and
        # append per event instead. Process creation is low rate, so opening
        # per write costs nothing that matters.
        # Every action writes through $Event.MessageData rather than a captured
        # variable. An -Action scriptblock runs in its own scope and cannot see
        # this script's $script: state, so a shared handle arrives as null and
        # every write throws into the job error stream where nothing surfaces
        # it. That failure is silent and the session completes reporting
        # success.
        $etwStartAction = {
            $e = $Event.SourceEventArgs.NewEvent
            $rec = [ordered]@{
                ts     = (Get-Date).ToUniversalTime().ToString('o')
                event  = 'start'
                source = 'etw'
                pid    = [int]$e.ProcessID
                ppid   = [int]$e.ParentProcessID
                name   = [string]$e.ProcessName
                path   = $null
                cmd    = $null
            }
            $p = Get-CimInstance Win32_Process -Filter "ProcessId=$([int]$e.ProcessID)" -ErrorAction SilentlyContinue
            if ($p) { $rec.path = $p.ExecutablePath; $rec.cmd = $p.CommandLine }
            [System.IO.File]::AppendAllText($Event.MessageData,
                ($rec | ConvertTo-Json -Compress -Depth 3) + "`n")
        }
        $stopAction = {
            $e = $Event.SourceEventArgs.NewEvent
            $rec = [ordered]@{
                ts       = (Get-Date).ToUniversalTime().ToString('o')
                event    = 'stop'
                source   = 'etw'
                pid      = [int]$e.ProcessID
                name     = [string]$e.ProcessName
                exitCode = [int]$e.ExitStatus
            }
            [System.IO.File]::AppendAllText($Event.MessageData,
                ($rec | ConvertTo-Json -Compress -Depth 3) + "`n")
        }
        # The WMI source carries the executable path and full command line on
        # the instance itself, so no second query is needed and no race exists
        # against a process that has already exited.
        $wmiStartAction = {
            $t = $Event.SourceEventArgs.NewEvent.TargetInstance
            $rec = [ordered]@{
                ts     = (Get-Date).ToUniversalTime().ToString('o')
                event  = 'start'
                source = 'wmi'
                pid    = [int]$t.ProcessId
                ppid   = [int]$t.ParentProcessId
                name   = [string]$t.Name
                path   = [string]$t.ExecutablePath
                cmd    = [string]$t.CommandLine
            }
            [System.IO.File]::AppendAllText($Event.MessageData,
                ($rec | ConvertTo-Json -Compress -Depth 3) + "`n")
        }
        $wmiStopAction = {
            $t = $Event.SourceEventArgs.NewEvent.TargetInstance
            $rec = [ordered]@{
                ts     = (Get-Date).ToUniversalTime().ToString('o')
                event  = 'stop'
                source = 'wmi'
                pid    = [int]$t.ProcessId
                name   = [string]$t.Name
            }
            [System.IO.File]::AppendAllText($Event.MessageData,
                ($rec | ConvertTo-Json -Compress -Depth 3) + "`n")
        }

        # Two independent sources, because each has a failure mode the other
        # covers.
        #
        # ETW (Win32_ProcessStartTrace) is push-based and catches processes far
        # shorter-lived than any poll interval, but it requires elevation.
        #
        # WMI instance creation (__InstanceCreationEvent WITHIN 1) needs no
        # privilege and carries the executable path and full command line, which
        # the ETW trace does not, at the cost of a one second poll that can miss
        # a process living less than that. Launcher-chain stages live for
        # seconds to minutes, so this is an acceptable floor for Q-4, but it IS
        # a floor and short-lived helpers may be missed.
        #
        # Whichever fire, both write to the same log with a 'source' field, and
        # the analysis deduplicates on process identifier.
        $live = @()

        if ($script:IsElevated) {
            try {
                $script:Subs += Register-CimIndicationEvent -ClassName Win32_ProcessStartTrace `
                    -SourceIdentifier 'FragcapEtwStart' -Action $etwStartAction -MessageData $procPath -ErrorAction Stop
                $script:Subs += Register-CimIndicationEvent -ClassName Win32_ProcessStopTrace `
                    -SourceIdentifier 'FragcapEtwStop' -Action $stopAction -MessageData $procPath -ErrorAction Stop
                $live += 'etw'
            } catch {
                Write-Log "ETW process source unavailable: $($_.Exception.Message)" -Level Warn -Source process
            }
        } else {
            Write-Log "Not elevated: ETW process source skipped, polling source only" -Level Warn -Source process
        }

        try {
            $script:Subs += Register-CimIndicationEvent `
                -Query "SELECT * FROM __InstanceCreationEvent WITHIN 1 WHERE TargetInstance ISA 'Win32_Process'" `
                -SourceIdentifier 'FragcapWmiStart' -Action $wmiStartAction -MessageData $procPath -ErrorAction Stop
            $script:Subs += Register-CimIndicationEvent `
                -Query "SELECT * FROM __InstanceDeletionEvent WITHIN 1 WHERE TargetInstance ISA 'Win32_Process'" `
                -SourceIdentifier 'FragcapWmiStop' -Action $wmiStopAction -MessageData $procPath -ErrorAction Stop
            $live += 'wmi'
        } catch {
            Write-Log "WMI process source unavailable: $($_.Exception.Message)" -Level Warn -Source process
        }

        if ($live.Count -eq 0) {
            Write-Log "No process source could be registered. Q-4 cannot be answered." -Level Error -Source process
            exit 1
        }

        # Prove the recorder writes before trusting it for a whole session, and
        # give delivery time to actually happen: WMI indication delivery runs
        # around a second, so a sub-second check reports failure on a recorder
        # that works. Poll for growth instead of sleeping once and guessing.
        # Snapshot everything already running. Creation events only name
        # processes that start during the session, so persistent platform
        # services (the Steam client, a publisher launcher left running) would
        # hold sockets under a process identifier with no name attached, which
        # is exactly the lifecycle class section 10.4 cares about.
        $baseStamp = (Get-Date).ToUniversalTime().ToString('o')
        $baseline = Get-CimInstance Win32_Process -ErrorAction SilentlyContinue | ForEach-Object {
            [ordered]@{
                ts     = $baseStamp
                event  = 'baseline'
                source = 'snapshot'
                pid    = [int]$_.ProcessId
                ppid   = [int]$_.ParentProcessId
                name   = [string]$_.Name
                path   = [string]$_.ExecutablePath
                cmd    = [string]$_.CommandLine
            } | ConvertTo-Json -Compress -Depth 3
        }
        [System.IO.File]::AppendAllText($procPath, (($baseline -join "`n") + "`n"))
        Write-Log "Baseline snapshot: $($baseline.Count) processes already running" -Level Info -Source process

        # Measure GROWTH past the baseline, not whether the file is non-empty.
        # The baseline alone makes it non-empty, so an emptiness test would pass
        # with the event recorder completely dead, which is the false success
        # this check exists to prevent.
        $sizeBeforeProbe = (Get-Item -LiteralPath $procPath).Length

        $probe = Start-Process -FilePath 'cmd.exe' -ArgumentList '/c','exit' -PassThru -WindowStyle Hidden
        $probe.WaitForExit()
        $deadline = (Get-Date).AddSeconds(8)
        while (((Get-Item -LiteralPath $procPath).Length -le $sizeBeforeProbe) -and ((Get-Date) -lt $deadline)) {
            Start-Sleep -Milliseconds 250
        }

        if ((Get-Item -LiteralPath $procPath).Length -le $sizeBeforeProbe) {
            Write-Log "Process recorder registered ($($live -join ', ')) but wrote nothing for a test process in 8 seconds." -Level Error -Source process
            # Action-block failures land in the subscriber's job error stream,
            # where nothing surfaces them by default. Surface them: this is the
            # only place the actual cause is visible.
            foreach ($id in @('FragcapEtwStart','FragcapWmiStart')) {
                $sub = Get-EventSubscriber -SourceIdentifier $id -ErrorAction SilentlyContinue
                if ($sub -and $sub.Action) {
                    Write-Log "  $id job state $($sub.Action.State), $($sub.Action.Error.Count) errors" -Level Error -Source process
                    $sub.Action.Error | Select-Object -First 3 | ForEach-Object {
                        Write-Log "    $_" -Level Error -Source process
                    }
                }
            }
            Write-Log "Aborting rather than recording a session that cannot answer Q-4." -Level Error -Source process
            exit 1
        }
        Write-Log "Process recorder verified, sources: $($live -join ', ') -> processes.jsonl" -Level Success -Source process

        $sockPath = Join-Path $outDir 'sockets.jsonl'
        $script:SocketLog = [System.IO.StreamWriter]::new($sockPath, $false, [System.Text.UTF8Encoding]::new($false))
        $script:SocketLog.AutoFlush = $false

        Write-Log "Socket sampler started -> sockets.jsonl (every ${IntervalMs}ms)" -Level Success -Source socket
        Write-Log "RECORDING. Launch the platform client now, then the title." -Level Success -Source session
        Write-Log "Press Ctrl+C after the game has exited cleanly." -Level Info -Source session

        $seen     = @{}
        $deadline = if ($DurationMinutes -gt 0) { (Get-Date).AddMinutes($DurationMinutes) } else { [datetime]::MaxValue }
        $samples  = 0
        $lastBeat = Get-Date

        while ((Get-Date) -lt $deadline) {
            $now  = (Get-Date).ToUniversalTime().ToString('o')
            $rows = [FragcapRecon.SocketTable]::Snapshot()
            $current = @{}

            foreach ($row in $rows) {
                $current[$row] = $true
                if (-not $seen.ContainsKey($row)) {
                    $seen[$row] = $now
                    $f = $row.Split('|')
                    $rec = [ordered]@{
                        ts = $now; event = 'open'; proto = $f[0]
                        la = $f[1]; lp = [int]$f[2]; ra = $f[3]
                        rp = $(if ($f[4]) { [int]$f[4] } else { 0 })
                        state = $f[5]; pid = [int]$f[6]
                    }
                    $script:SocketLog.WriteLine(($rec | ConvertTo-Json -Compress))
                }
            }

            foreach ($key in @($seen.Keys)) {
                if (-not $current.ContainsKey($key)) {
                    $f = $key.Split('|')
                    $rec = [ordered]@{
                        ts = $now; event = 'close'; proto = $f[0]
                        la = $f[1]; lp = [int]$f[2]; ra = $f[3]
                        rp = $(if ($f[4]) { [int]$f[4] } else { 0 })
                        state = $f[5]; pid = [int]$f[6]
                        openedAt = $seen[$key]
                    }
                    $script:SocketLog.WriteLine(($rec | ConvertTo-Json -Compress))
                    $seen.Remove($key)
                }
            }

            $samples++
            if (((Get-Date) - $lastBeat).TotalSeconds -ge 30) {
                $script:SocketLog.Flush()
                Write-Log "$samples samples, $($seen.Count) sockets open" -Level Debug -Source socket
                $lastBeat = Get-Date
            }

            Start-Sleep -Milliseconds $IntervalMs
        }
    }
    finally {
        Write-Log "Stopping recorders" -Level Info -Source session

        foreach ($id in @('FragcapEtwStart','FragcapEtwStop','FragcapWmiStart','FragcapWmiStop')) {
            Unregister-Event -SourceIdentifier $id -ErrorAction SilentlyContinue
        }
        foreach ($r in $script:Recorders) {
            if ($r -and -not $r.HasExited) {
                Stop-Process -Id $r.Id -Force -ErrorAction SilentlyContinue
            }
        }
        if ($script:SocketLog) { $script:SocketLog.Flush(); $script:SocketLog.Dispose() }

        if ($outDir -and (Test-Path -LiteralPath $outDir)) {
            $endUtc = (Get-Date).ToUniversalTime()
            $mp = Join-Path $outDir 'session.json'
            $m  = Get-Content -LiteralPath $mp -Raw | ConvertFrom-Json
            $m | Add-Member -NotePropertyName 'endUtc' -NotePropertyValue $endUtc.ToString('o') -Force
            $m | Add-Member -NotePropertyName 'durationMinutes' `
                 -NotePropertyValue ([math]::Round(($endUtc - $startUtc).TotalMinutes, 2)) -Force
            $m | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $mp -Encoding utf8

            Write-Log "Session complete: $outDir" -Level Success -Source session
            Get-ChildItem -LiteralPath $outDir | ForEach-Object {
                Write-Log ("  {0,-18} {1,10:N0} bytes" -f $_.Name, $_.Length) -Level Info -Source session
            }
            Write-Log "Next: run Invoke-ReconAnalysis.ps1 against this directory." -Level Info -Source session
        }
    }

    exit 0

#_______________________________________________________________________________
## End of script
