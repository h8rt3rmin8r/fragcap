<#
.SYNOPSIS
    Answer reconnaissance questions Q-1 to Q-6 from a recorded session.

.DESCRIPTION
    Consumes a session directory produced by Start-ReconSession.ps1 and derives
    the evidence needed to resolve the open questions in specification section
    29, then writes a findings report and a draft Appendix D entry.

    The analysis proceeds in the order the protocol prescribes, because each
    stage narrows the next:

      Q-4  Process topology, from the process tree recording. Answered first:
           once the owning process identifiers are known, everything else is a
           filter rather than a search.
      Q-1  5-tuple attributability, by joining capture conversations against
           the socket table log.
      Q-2  Relay tunneling, from endpoint ownership. Ownership is resolved
           from DNS answers and TLS server names observed in the capture
           itself, so no external lookup is performed and no query discloses
           what was captured.
      Q-3  Connection lifetime distribution, from socket open and close
           brackets. Reports the packet-weighted fraction on short
           connections, which is the number that decides A-2, rather than the
           connection count, which does not.
      Q-5  Loopback handoff visibility, from the loopback capture across the
           launch window.
      Q-6  Encryption posture, by Shannon entropy over sampled payloads per
           traffic class, cross-checked against observed TLS handshakes.

    ATTRIBUTION ASYMMETRY. TCP conversations join on the full 5-tuple. UDP
    conversations join on the local endpoint alone, because the UDP socket
    table carries no remote endpoint (finding PF-3 in README.md). UDP sockets
    bound to a wildcard address are matched against the wildcard as well as the
    specific interface address.

    OUTPUT. Intermediate tables are written as CSV under an 'analysis'
    subdirectory so any conclusion can be traced back to its evidence.
    FINDINGS.md carries the report and the draft Appendix D entry.

    SCRUBBING. The report is derived, but it is NOT automatically scrubbed:
    remote addresses appear in the endpoint tables by necessity. Review
    FINDINGS.md and remove operator-attributable detail before promoting any of
    it into the specification. Pass -Scrub to mask the local address and
    redact remote host octets in the report.

    This script reads recorded artifacts only. It starts no capture, contacts
    no network host, and touches no running process.

.PARAMETER SessionPath
    Directory produced by Start-ReconSession.ps1, containing session.json.
    When omitted the most recently modified session under captures/recon is
    used.
    Alias: p

.PARAMETER ShortConnectionMs
    Connections shorter than this are counted as short for the A-2 assessment.
    Set this to the socket poll interval actually used, which is recorded in
    session.json and used automatically when this is not given.
    Alias: c

.PARAMETER SampleCount
    Packets sampled per traffic class for the Q-6 entropy measurement.
    Default: 400.
    Alias: n

.PARAMETER Scrub
    Mask the local address and redact the low octets of remote addresses in
    FINDINGS.md. The CSV tables under analysis/ are never scrubbed, since they
    are the evidence and stay local.

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
    .\Invoke-ReconAnalysis.ps1
    Analyze the most recent session and write FINDINGS.md into it.

.EXAMPLE
    .\Invoke-ReconAnalysis.ps1 -SessionPath captures/recon/eso-20260806-201500
    Analyze a specific session.

.EXAMPLE
    .\Invoke-ReconAnalysis.ps1 -Scrub
    Produce a report with addresses masked, suitable for sharing before a
    manual review pass.

.NOTES
    Exit codes: 0 success, 1 the session is incomplete or unreadable, 2
    environment precondition failure. An inconclusive result is reported as
    inconclusive; see the closing note in reconnaissance.md.
#>
[CmdletBinding(SupportsShouldProcess=$false,ConfirmImpact='None',DefaultParameterSetName='Default')]
Param(
    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("p")]
    [string]$SessionPath = '',

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("c")]
    [ValidateRange(0,60000)]
    [int]$ShortConnectionMs = 0,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("n")]
    [ValidateRange(20,5000)]
    [int]$SampleCount = 400,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Switch]$Scrub,

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

    function ConvertFrom-HumanBytes {
        # tshark humanizes byte columns: "5633 bytes", "12 kB", "97 MB".
        # Treating these as integers silently under-reports volume by three or
        # six orders of magnitude, which is why this exists.
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [AllowEmptyString()]
            [string]$Value,

            [Parameter(Mandatory=$true)]
            [AllowEmptyString()]
            [string]$Unit
        )
        $n = 0.0
        if (-not [double]::TryParse($Value, [ref]$n)) { return 0 }
        switch ($Unit) {
            'bytes' { return [long]$n }
            'kB'    { return [long]($n * 1000) }
            'MB'    { return [long]($n * 1000000) }
            'GB'    { return [long]($n * 1000000000) }
            'TB'    { return [long]($n * 1000000000000) }
            default { return [long]$n }
        }
    }

    function Split-Endpoint {
        # "1.2.3.4:443" or "2600:1901:0:9e23:::443". Split on the LAST colon:
        # IPv6 literals are full of colons and a naive split mangles them.
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$Endpoint
        )
        $i = $Endpoint.LastIndexOf(':')
        if ($i -lt 0) { return @{ Address = $Endpoint; Port = 0 } }
        $port = 0
        [void][int]::TryParse($Endpoint.Substring($i + 1), [ref]$port)
        return @{ Address = $Endpoint.Substring(0, $i); Port = $port }
    }

    function Get-Conversation {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$CaptureFile,

            [Parameter(Mandatory=$true)]
            [ValidateSet('tcp','udp')]
            [string]$Protocol
        )
        $raw = & $script:TsharkPath -r $CaptureFile -q -z "conv,$Protocol" 2>$null
        $rows = @()
        foreach ($line in $raw) {
            if ($line -notmatch '<->') { continue }
            $parts = $line -split '\s*<->\s*', 2
            if ($parts.Count -ne 2) { continue }
            $a = Split-Endpoint -Endpoint $parts[0].Trim()
            $t = $parts[1].Trim() -split '\s+'
            # peer, then three (frames, bytes, unit) triples, then start and
            # duration: twelve tokens exactly.
            if ($t.Count -lt 12) { continue }
            $b = Split-Endpoint -Endpoint $t[0]
            $rows += [pscustomobject]@{
                Protocol    = $Protocol
                AddressA    = $a.Address
                PortA       = $a.Port
                AddressB    = $b.Address
                PortB       = $b.Port
                FramesBtoA  = [long]$t[1]
                BytesBtoA   = ConvertFrom-HumanBytes -Value $t[2] -Unit $t[3]
                FramesAtoB  = [long]$t[4]
                BytesAtoB   = ConvertFrom-HumanBytes -Value $t[5] -Unit $t[6]
                FramesTotal = [long]$t[7]
                BytesTotal  = ConvertFrom-HumanBytes -Value $t[8] -Unit $t[9]
                RelStart    = [double]$t[10]
                Duration    = [double]$t[11]
            }
        }
        return $rows
    }

    function Get-DnsMap {
        # Address to hostname, built from DNS answers inside the capture.
        # Using observed answers rather than reverse lookups keeps the analysis
        # offline and reflects what the client actually resolved.
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$CaptureFile
        )
        $map = @{}
        $raw = & $script:TsharkPath -r $CaptureFile -Y 'dns.flags.response==1' `
               -T fields -e dns.qry.name -e dns.a -e dns.aaaa 2>$null
        foreach ($line in $raw) {
            $f = $line -split "`t"
            if ($f.Count -lt 2 -or -not $f[0]) { continue }
            $name = $f[0]
            foreach ($col in @($f[1], $f[2])) {
                if (-not $col) { continue }
                foreach ($addr in ($col -split ',')) {
                    if ($addr -and -not $map.ContainsKey($addr)) { $map[$addr] = $name }
                }
            }
        }
        return $map
    }

    function Get-TlsServerName {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$CaptureFile
        )
        $map = @{}
        $raw = & $script:TsharkPath -r $CaptureFile -Y 'tls.handshake.type==1' `
               -T fields -e ip.dst -e ipv6.dst -e tls.handshake.extensions_server_name 2>$null
        foreach ($line in $raw) {
            $f = $line -split "`t"
            if ($f.Count -lt 3) { continue }
            $dst = if ($f[0]) { $f[0] } else { $f[1] }
            if ($dst -and $f[2] -and -not $map.ContainsKey($dst)) { $map[$dst] = $f[2] }
        }
        return $map
    }

    function Measure-Entropy {
        # Shannon entropy in bits per byte. Above about 7.5 indicates encrypted
        # or compressed content; below about 6.0 indicates structured or
        # cleartext content. The band between is genuinely ambiguous and is
        # reported as such rather than rounded to a verdict.
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [AllowEmptyCollection()]
            [byte[]]$Bytes
        )
        if ($Bytes.Length -eq 0) { return 0.0 }
        $freq = New-Object 'int[]' 256
        foreach ($b in $Bytes) { $freq[$b]++ }
        $entropy = 0.0
        foreach ($count in $freq) {
            if ($count -eq 0) { continue }
            $p = $count / $Bytes.Length
            $entropy -= $p * [math]::Log($p, 2)
        }
        return [math]::Round($entropy, 3)
    }

    function Get-PayloadEntropy {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$CaptureFile,

            [Parameter(Mandatory=$true)]
            [string]$Filter,

            [Parameter(Mandatory=$true)]
            [string]$Field,

            [Parameter(Mandatory=$true)]
            [int]$Count
        )
        # Use tcp.payload / udp.payload, not data.data. Once tshark dissects a
        # payload as a protocol (TLS, HTTP, QUIC) the generic data.data field is
        # empty, so sampling it returns nothing for exactly the encrypted
        # traffic this measurement exists to characterize.
        #
        # Deliberately no -c: that caps packets READ, not packets MATCHED, so on
        # a capture dominated by one high-volume flow it returns almost nothing
        # for every other class. Scan the file and stop accumulating once the
        # sample is large enough.
        $bytes   = New-Object System.Collections.Generic.List[byte]
        $matched = 0
        foreach ($hex in (& $script:TsharkPath -r $CaptureFile -Y $Filter -T fields -e $Field 2>$null)) {
            if (-not $hex) { continue }
            $clean = $hex -replace '[^0-9a-fA-F]', ''
            if ($clean.Length -lt 2) { continue }
            for ($i = 0; $i + 1 -lt $clean.Length; $i += 2) {
                $bytes.Add([Convert]::ToByte($clean.Substring($i, 2), 16))
            }
            $matched++
            if ($matched -ge $Count) { break }
        }
        return [pscustomobject]@{
            SampleBytes = $bytes.Count
            Entropy     = Measure-Entropy -Bytes $bytes.ToArray()
        }
    }

    function Format-Address {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [AllowEmptyString()]
            [string]$Address,

            [Parameter(Mandatory=$false)]
            [bool]$IsLocal = $false
        )
        if (-not $script:DoScrub) { return $Address }
        if ($IsLocal) { return '<local>' }
        if ($Address -match '^(\d+)\.(\d+)\.\d+\.\d+$') { return "$($Matches[1]).$($Matches[2]).x.x" }
        if ($Address -match '^([0-9a-fA-F]*:[0-9a-fA-F]*):') { return "$($Matches[1])::redacted" }
        return '<redacted>'
    }

#_______________________________________________________________________________
## Declare Variables and Arrays

    $script:LogQuiet    = $false
    $script:LogSilent   = $false
    $script:DoScrub     = $false
    $ThisScriptPath     = $MyInvocation.MyCommand.Path
    $script:TsharkPath  = 'C:\Program Files\Wireshark\tshark.exe'

    # Loud and fatal. The default of Continue let a per-item assignment failure
    # empty every hostname in the report while the script still exited 0, which
    # is precisely the silent-success failure this analysis exists to detect.
    $ErrorActionPreference = 'Stop'

    # Shannon entropy bands, bits per byte.
    $script:EntropyHigh = 7.5
    $script:EntropyLow  = 6.0

#_______________________________________________________________________________
## Execute Operations

    # Catch help text requests
    if (($Help) -or ($PSCmdlet.ParameterSetName -eq 'HelpText')) {
        Get-Help $ThisScriptPath -Detailed
        exit 0
    }

    if ($Quiet)  { $script:LogQuiet  = $true }
    if ($Silent) { $script:LogSilent = $true }
    if ($Scrub)  { $script:DoScrub   = $true }

    Assert-PSVersion -Minimum '7.0'

    if (-not (Test-Path -LiteralPath $script:TsharkPath)) {
        Write-Host "FAIL: tshark not found at $script:TsharkPath. Install Wireshark." -ForegroundColor Red
        exit 2
    }

    if (-not $SessionPath) {
        # Four steps up from <repo>/docs/plans/recon/<file>: recon, plans,
        # docs, root. Three lands in docs/ and misses the sessions entirely.
        $repoRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $ThisScriptPath)))
        $root     = Join-Path $repoRoot 'captures/recon'
        if (-not (Test-Path -LiteralPath $root)) {
            Write-Host "FAIL: no sessions found under $root. Run Start-ReconSession.ps1 first." -ForegroundColor Red
            exit 1
        }
        $latest = Get-ChildItem -LiteralPath $root -Directory |
                  Sort-Object LastWriteTime -Descending | Select-Object -First 1
        if (-not $latest) {
            Write-Host "FAIL: no session directories under $root." -ForegroundColor Red
            exit 1
        }
        $SessionPath = $latest.FullName
    }

    $manifestPath = Join-Path $SessionPath 'session.json'
    if (-not (Test-Path -LiteralPath $manifestPath)) {
        Write-Host "FAIL: $SessionPath is not a session directory (no session.json)." -ForegroundColor Red
        exit 1
    }
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    Write-Log "Session $($manifest.sessionId), title '$($manifest.title)'" -Level Success -Source load

    if ($ShortConnectionMs -eq 0) {
        $ShortConnectionMs = [int]$manifest.intervalMs
        Write-Log "Short-connection threshold set to the poll interval, ${ShortConnectionMs}ms" -Level Info -Source load
    }

    $primary   = Join-Path $SessionPath 'primary.pcapng'
    $loopback  = Join-Path $SessionPath 'loopback.pcapng'
    $sockPath  = Join-Path $SessionPath 'sockets.jsonl'
    $procPath  = Join-Path $SessionPath 'processes.jsonl'
    $analysis  = Join-Path $SessionPath 'analysis'
    $null = New-Item -ItemType Directory -Path $analysis -Force

    foreach ($required in @($primary, $sockPath)) {
        if (-not (Test-Path -LiteralPath $required)) {
            Write-Host "FAIL: missing required artifact $required" -ForegroundColor Red
            exit 1
        }
    }

    # ---- Load recorded state -------------------------------------------------

    $sockets = @()
    foreach ($line in [System.IO.File]::ReadLines($sockPath)) {
        if ($line) { $sockets += ($line | ConvertFrom-Json) }
    }
    Write-Log "Loaded $($sockets.Count) socket records" -Level Info -Source load

    $procs = @()
    if ((Test-Path -LiteralPath $procPath) -and (Get-Item -LiteralPath $procPath).Length -gt 0) {
        foreach ($line in [System.IO.File]::ReadLines($procPath)) {
            if ($line) { $procs += ($line | ConvertFrom-Json) }
        }
    }
    if ($procs.Count -eq 0) {
        Write-Log "processes.jsonl is empty. Q-4 cannot be answered; the session was probably not run elevated." -Level Warn -Source load
    } else {
        Write-Log "Loaded $($procs.Count) process events" -Level Info -Source load
    }

    $localAddrs = @{}
    foreach ($s in $sockets) { if ($s.la) { $localAddrs[$s.la] = $true } }

    # ---- Q-4: process topology ----------------------------------------------

    Write-Log "Q-4: building process topology" -Level Info -Source analyze
    # Baseline records name processes that were already running when the
    # session began, which is how persistent platform services get a name
    # instead of a bare identifier. Creation events supersede them, and the two
    # process sources can both report the same start, so deduplicate on
    # identifier keeping the earliest real start.
    $stops  = @($procs | Where-Object { $_.event -eq 'stop' })
    $stopBy = @{}
    foreach ($s in $stops) { $stopBy[[string]$s.pid] = $s.ts }

    $byPid = [ordered]@{}
    foreach ($p in ($procs | Where-Object { $_.event -eq 'baseline' })) {
        $byPid[[string]$p.pid] = @{ Rec = $p; Pre = $true }
    }
    foreach ($p in ($procs | Where-Object { $_.event -eq 'start' })) {
        $k = [string]$p.pid
        if ((-not $byPid.Contains($k)) -or $byPid[$k].Pre) {
            $byPid[$k] = @{ Rec = $p; Pre = $false }
        }
    }

    $tree = foreach ($k in $byPid.Keys) {
        $p = $byPid[$k].Rec
        [pscustomobject]@{
            Pid            = $p.pid
            Ppid           = $p.ppid
            Name           = $p.name
            Path           = $p.path
            Started        = $p.ts
            Stopped        = $stopBy[$k]
            Persisted      = -not $stopBy.ContainsKey($k)
            RunningAtStart = $byPid[$k].Pre
        }
    }
    $tree | Export-Csv -LiteralPath (Join-Path $analysis 'process-tree.csv') -NoTypeInformation

    # Processes that actually held sockets are the ones that matter.
    $socketPids = @{}
    foreach ($s in $sockets) { $socketPids[[string]$s.pid] = $true }
    $netProcs = @($tree | Where-Object { $socketPids.ContainsKey([string]$_.Pid) })

    # ---- Conversations and ownership ----------------------------------------

    Write-Log "Extracting TCP conversations" -Level Info -Source analyze
    $tcpConv = Get-Conversation -CaptureFile $primary -Protocol tcp
    Write-Log "Extracting UDP conversations" -Level Info -Source analyze
    $udpConv = Get-Conversation -CaptureFile $primary -Protocol udp
    Write-Log "Found $($tcpConv.Count) TCP and $($udpConv.Count) UDP conversations" -Level Success -Source analyze

    # Index sockets for the join. TCP keys on the full 5-tuple; UDP keys on the
    # local endpoint alone, because the UDP socket table carries no remote
    # endpoint (PF-3).
    $tcpIndex = @{}
    $udpIndex = @{}
    foreach ($s in $sockets) {
        if ($s.event -ne 'open') { continue }
        if ($s.proto -like 'tcp*') {
            $tcpIndex["$($s.la)|$($s.lp)|$($s.ra)|$($s.rp)"] = $s
        } else {
            $udpIndex["$($s.la)|$($s.lp)"] = $s
        }
    }

    $flows = foreach ($c in ($tcpConv + $udpConv)) {
        $aLocal = $localAddrs.ContainsKey($c.AddressA)
        $bLocal = $localAddrs.ContainsKey($c.AddressB)
        if ($aLocal -and -not $bLocal) {
            $lAddr = $c.AddressA; $lPort = $c.PortA
            $rAddr = $c.AddressB; $rPort = $c.PortB
        } elseif ($bLocal -and -not $aLocal) {
            $lAddr = $c.AddressB; $lPort = $c.PortB
            $rAddr = $c.AddressA; $rPort = $c.PortA
        } else {
            $lAddr = $c.AddressA; $lPort = $c.PortA
            $rAddr = $c.AddressB; $rPort = $c.PortB
        }

        $match = $null
        if ($c.Protocol -eq 'tcp') {
            $match = $tcpIndex["$lAddr|$lPort|$rAddr|$rPort"]
        } else {
            foreach ($cand in @("$lAddr|$lPort", "0.0.0.0|$lPort", "::|$lPort")) {
                if ($udpIndex.ContainsKey($cand)) { $match = $udpIndex[$cand]; break }
            }
        }

        $ownerName = $null
        if ($match) {
            $hit = $tree | Where-Object { $_.Pid -eq $match.pid } | Select-Object -First 1
            $ownerName = if ($hit) { $hit.Name } else { "pid $($match.pid)" }
        }

        # A conversation with neither endpoint in this machine's socket table is
        # not ours: LAN multicast and broadcast from other hosts (SSDP, mDNS,
        # DHCPv6) is captured because the adapter sees it, not because anything
        # here sent it. Counting those as attribution failures understates the
        # mechanism, so they are classified separately rather than lumped in.
        $isOurs = ($aLocal -or $bLocal)

        [pscustomobject]@{
            Protocol      = $c.Protocol
            LocalAddress  = $lAddr
            LocalPort     = $lPort
            RemoteAddress = $rAddr
            RemotePort    = $rPort
            Frames        = $c.FramesTotal
            Bytes         = $c.BytesTotal
            DurationSec   = $c.Duration
            OwnerPid      = if ($match) { $match.pid } else { $null }
            OwnerName     = $ownerName
            Attributed    = [bool]$match
            OurTraffic    = $isOurs
        }
    }
    $flows | Export-Csv -LiteralPath (Join-Path $analysis 'flows.csv') -NoTypeInformation

    $totalFrames = ($flows | Measure-Object -Property Frames -Sum).Sum
    $attrFrames  = ($flows | Where-Object { $_.Attributed } | Measure-Object -Property Frames -Sum).Sum
    if (-not $totalFrames) { $totalFrames = 1 }
    $attrPct = [math]::Round(100.0 * $attrFrames / $totalFrames, 2)

    # Judge attribution only on traffic this machine actually originated or
    # received. Flow counts including foreign multicast measure the LAN, not
    # the mechanism.
    $ours     = @($flows | Where-Object { $_.OurTraffic })
    $foreign  = @($flows | Where-Object { -not $_.OurTraffic })
    $tcpFlows = @($ours | Where-Object { $_.Protocol -eq 'tcp' })
    $udpFlows = @($ours | Where-Object { $_.Protocol -eq 'udp' })
    $tcpAttr  = @($tcpFlows | Where-Object { $_.Attributed }).Count
    $udpAttr  = @($udpFlows | Where-Object { $_.Attributed }).Count
    $udpUnattr = @($udpFlows | Where-Object { -not $_.Attributed })
    $udpUnattrBytes = ($udpUnattr | Measure-Object -Property Bytes -Sum).Sum
    if (-not $udpUnattrBytes) { $udpUnattrBytes = 0 }
    $udpBytes = ($udpFlows | Measure-Object -Property Bytes -Sum).Sum
    if (-not $udpBytes) { $udpBytes = 1 }
    $udpMissBytePct = [math]::Round(100.0 * $udpUnattrBytes / $udpBytes, 4)
    Write-Log "Q-1: $attrPct% of frames attributed; UDP misses carry $udpMissBytePct% of UDP bytes" -Level Success -Source analyze

    # ---- Q-2: endpoint ownership --------------------------------------------

    Write-Log "Q-2: resolving endpoint ownership from observed DNS and TLS" -Level Info -Source analyze
    $dnsMap = Get-DnsMap -CaptureFile $primary
    $sniMap = Get-TlsServerName -CaptureFile $primary
    Write-Log "Resolved $($dnsMap.Count) addresses from DNS, $($sniMap.Count) from TLS server names" -Level Info -Source analyze

    $endpoints = $flows | Group-Object RemoteAddress | ForEach-Object {
        $addr = $_.Name
        # Not $host: that is a read-only PowerShell automatic variable, and
        # assigning to it fails per item while leaving the pipeline running, so
        # every hostname silently comes back empty and the report still looks
        # complete.
        $hostName = if ($dnsMap.ContainsKey($addr)) { $dnsMap[$addr] }
                    elseif ($sniMap.ContainsKey($addr)) { $sniMap[$addr] }
                    else { $null }
        [pscustomobject]@{
            RemoteAddress = $addr
            Hostname      = $hostName
            Flows         = $_.Count
            Frames        = ($_.Group | Measure-Object -Property Frames -Sum).Sum
            Bytes         = ($_.Group | Measure-Object -Property Bytes -Sum).Sum
            Owners        = (($_.Group.OwnerName | Where-Object { $_ } | Sort-Object -Unique) -join ';')
            Ports         = (($_.Group.RemotePort | Sort-Object -Unique) -join ';')
        }
    } | Sort-Object Bytes -Descending
    $endpoints | Export-Csv -LiteralPath (Join-Path $analysis 'endpoints.csv') -NoTypeInformation

    $unnamed = @($endpoints | Where-Object { -not $_.Hostname })

    # ---- Q-3: connection lifetime distribution ------------------------------

    Write-Log "Q-3: measuring connection lifetime distribution" -Level Info -Source analyze
    $lifetimes = foreach ($s in ($sockets | Where-Object { $_.event -eq 'close' -and $_.openedAt })) {
        $ms = ([datetime]$s.ts - [datetime]$s.openedAt).TotalMilliseconds
        [pscustomobject]@{
            Protocol = $s.proto
            Pid      = $s.pid
            Local    = "$($s.la):$($s.lp)"
            Remote   = "$($s.ra):$($s.rp)"
            Ms       = [math]::Round($ms, 1)
            Short    = ($ms -lt $ShortConnectionMs)
        }
    }
    $lifetimes | Export-Csv -LiteralPath (Join-Path $analysis 'lifetimes.csv') -NoTypeInformation

    $closed     = @($lifetimes).Count
    $shortCount = @($lifetimes | Where-Object { $_.Short }).Count
    $shortPct   = if ($closed) { [math]::Round(100.0 * $shortCount / $closed, 2) } else { 0 }

    # The packet-weighted fraction is what decides A-2. Many short connections
    # carrying almost no traffic is a different situation from a few carrying a
    # lot, and the connection count alone cannot tell them apart.
    $shortKeys = @{}
    foreach ($l in ($lifetimes | Where-Object { $_.Short })) { $shortKeys[$l.Local] = $true }
    $shortFrames = ($flows | Where-Object {
        $shortKeys.ContainsKey("$($_.LocalAddress):$($_.LocalPort)")
    } | Measure-Object -Property Frames -Sum).Sum
    if (-not $shortFrames) { $shortFrames = 0 }
    $shortFramePct = [math]::Round(100.0 * $shortFrames / $totalFrames, 2)

    $sorted = @($lifetimes.Ms | Sort-Object)
    $median = if ($sorted.Count) { $sorted[[int]($sorted.Count / 2)] } else { 0 }
    $p10    = if ($sorted.Count) { $sorted[[int]($sorted.Count * 0.10)] } else { 0 }

    # ---- Q-5: loopback handoff ----------------------------------------------

    Write-Log "Q-5: examining the loopback capture" -Level Info -Source analyze
    $loopSummary = 'loopback.pcapng missing'
    $loopFlows   = @()
    if (Test-Path -LiteralPath $loopback) {
        $loopFlows = @(Get-Conversation -CaptureFile $loopback -Protocol tcp) +
                     @(Get-Conversation -CaptureFile $loopback -Protocol udp)
        $loopFlows | Export-Csv -LiteralPath (Join-Path $analysis 'loopback-flows.csv') -NoTypeInformation
        $loopSummary = "$($loopFlows.Count) loopback conversations"
    }

    # ---- Q-6: encryption posture --------------------------------------------

    Write-Log "Q-6: sampling payload entropy" -Level Info -Source analyze
    $classes = @(
        @{ Name = 'TCP payload'; Filter = 'tcp.len>0';    Field = 'tcp.payload' },
        @{ Name = 'UDP payload'; Filter = 'udp.length>8'; Field = 'udp.payload' }
    )
    $entropy = foreach ($c in $classes) {
        $r = Get-PayloadEntropy -CaptureFile $primary -Filter $c.Filter `
             -Field $c.Field -Count $SampleCount
        $verdict = if ($r.SampleBytes -eq 0) { 'no samples' }
                   elseif ($r.Entropy -ge $script:EntropyHigh) { 'encrypted or compressed' }
                   elseif ($r.Entropy -le $script:EntropyLow) { 'structured or cleartext' }
                   else { 'INCONCLUSIVE' }
        [pscustomobject]@{
            Class       = $c.Name
            SampleBytes = $r.SampleBytes
            Entropy     = $r.Entropy
            Reading     = $verdict
        }
    }
    $entropy | Export-Csv -LiteralPath (Join-Path $analysis 'entropy.csv') -NoTypeInformation
    $tlsCount = $sniMap.Count

    # ---- Report --------------------------------------------------------------

    Write-Log "Writing FINDINGS.md" -Level Info -Source report
    $topFlows = $flows | Sort-Object Bytes -Descending | Select-Object -First 15
    $topEnds  = $endpoints | Select-Object -First 15
    $nl       = [Environment]::NewLine
    $sb       = New-Object System.Text.StringBuilder

    [void]$sb.AppendLine("# Reconnaissance findings: $($manifest.title)")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("Session ``$($manifest.sessionId)``, started $($manifest.startUtc), " +
                         "duration $($manifest.durationMinutes) minutes.")
    [void]$sb.AppendLine("Socket poll interval $($manifest.intervalMs) ms. " +
                         "Short-connection threshold $ShortConnectionMs ms.")
    [void]$sb.AppendLine()
    if ($script:DoScrub) {
        [void]$sb.AppendLine("Addresses in this report are masked. The unmasked evidence is in " +
                             "``analysis/`` and stays local.")
    } else {
        [void]$sb.AppendLine("**This report is derived but NOT scrubbed.** Review and remove " +
                             "operator-attributable detail before promoting any of it into the " +
                             "specification.")
    }
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("## Verdict summary")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("| Question | Assumption | Reading |")
    [void]$sb.AppendLine("| --- | --- | --- |")

    # A-1 asks whether a packet resolves to an owning process, which is a
    # socket-table join and does not need the process tree. A missing tree
    # costs the process NAMES (and therefore Q-4 and role separation), not the
    # attribution itself, so it must not be reported as an attribution failure.
    $q1 = if ($attrPct -ge 95) { 'A-1 holds' }
          elseif ($attrPct -ge 70) { 'A-1 partially holds' }
          else { 'A-1 fails' }
    if ($procs.Count -eq 0) { $q1 += ' by PID; names unavailable' }
    $q2 = if ($unnamed.Count -eq 0) { 'A-3 holds (all endpoints named)' }
          else { "review: $($unnamed.Count) unnamed endpoints" }
    $q3 = if (-not $closed) { 'INCONCLUSIVE (no closed connections)' }
          elseif ($shortFramePct -lt 1) { 'A-2 holds' }
          elseif ($shortFramePct -lt 5) { 'A-2 holds, marginal' }
          else { 'A-2 at risk' }
    $q4 = if ($procs.Count -eq 0) { 'INCONCLUSIVE (session not elevated)' } else { 'see topology' }
    # Loopback traffic existing does not establish that the launcher-to-client
    # handoff is what produced it: plenty of unrelated software talks over
    # loopback. Confirming the handoff needs the conversations attributed to the
    # launcher and client processes, which needs the process tree. Without it
    # this is inconclusive, and saying otherwise would be the convenient answer
    # rather than the true one.
    $q5 = if (-not (Test-Path -LiteralPath $loopback)) { 'INCONCLUSIVE (no loopback capture)' }
          elseif ($loopFlows.Count -eq 0) { 'A-5 fails (no loopback traffic at all)' }
          elseif ($procs.Count -eq 0) { "INCONCLUSIVE ($($loopFlows.Count) loopback conv, unattributable without process tree)" }
          else { "A-5 plausible ($($loopFlows.Count) loopback conv; confirm the launcher and client own them)" }

    [void]$sb.AppendLine("| Q-1 5-tuple attributable | A-1 | $q1 ($attrPct% of frames) |")
    [void]$sb.AppendLine("| Q-2 relay tunneled | A-3 | $q2 |")
    [void]$sb.AppendLine("| Q-3 connection lifetimes | A-2 | $q3 ($shortFramePct% frames on short) |")
    [void]$sb.AppendLine("| Q-4 process topology | A-4 | $q4 |")
    [void]$sb.AppendLine("| Q-5 loopback handoff | A-5 | $q5 |")
    [void]$sb.AppendLine("| Q-6 encryption posture | n/a | $tlsCount TLS server names observed |")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("An INCONCLUSIVE reading is a real result. Record it as inconclusive " +
                         "rather than rounding it to whichever answer is more convenient.")
    [void]$sb.AppendLine()

    [void]$sb.AppendLine("## Q-4: process topology")
    [void]$sb.AppendLine()
    if ($netProcs.Count) {
        [void]$sb.AppendLine("Processes that held sockets during the session, in creation order.")
        [void]$sb.AppendLine()
        [void]$sb.AppendLine("``Pre`` marks a process already running when the session began, " +
                             "which is the persistent platform-service lifecycle class. The " +
                             "others were created during the session and are the launch chain.")
        [void]$sb.AppendLine()
        [void]$sb.AppendLine("| PID | Parent | Image | Pre | Started | Persisted |")
        [void]$sb.AppendLine("| --- | --- | --- | --- | --- | --- |")
        foreach ($p in ($netProcs | Sort-Object RunningAtStart, Started)) {
            [void]$sb.AppendLine("| $($p.Pid) | $($p.Ppid) | ``$($p.Name)`` | $($p.RunningAtStart) | $($p.Started) | $($p.Persisted) |")
        }
    } else {
        [void]$sb.AppendLine("No process events recorded. Q-4 is unanswered, and Q-1 " +
                             "attribution below therefore reports socket ownership by " +
                             "identifier only. Re-run the session from an elevated shell.")
    }
    [void]$sb.AppendLine()

    [void]$sb.AppendLine("## Q-1: attribution")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("Counting only traffic this machine sent or received. " +
                         "$($foreign.Count) further conversations were foreign LAN multicast " +
                         "or broadcast, which the adapter sees but nothing here originated; " +
                         "those are excluded rather than counted as attribution failures.")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("- Conversations: $($tcpFlows.Count) TCP, $($udpFlows.Count) UDP.")
    [void]$sb.AppendLine("- Attributed: $tcpAttr of $($tcpFlows.Count) TCP, $udpAttr of $($udpFlows.Count) UDP.")
    [void]$sb.AppendLine("- Frame-weighted attribution: **$attrPct%**.")
    [void]$sb.AppendLine("- **UDP conversations that missed carry $udpMissBytePct% of UDP bytes.**")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("TCP joins on the full 5-tuple. UDP joins on the local endpoint alone, " +
                         "because the UDP socket table carries no remote endpoint (PF-3).")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("Read the byte figure, not the conversation count. UDP misses cluster on " +
                         "ephemeral request-response sockets, chiefly DNS, which open and close " +
                         "inside one poll interval and carry almost nothing. A low conversation " +
                         "rate paired with a negligible byte share means the race window is real " +
                         "but lands on traffic that does not matter. A high byte share is the " +
                         "result that would threaten A-1.")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("Largest conversations by volume:")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("| Proto | Local port | Remote | Frames | Bytes | Owner |")
    [void]$sb.AppendLine("| --- | --- | --- | --- | --- | --- |")
    foreach ($f in $topFlows) {
        $r = Format-Address -Address $f.RemoteAddress
        $o = if ($f.OwnerName) { $f.OwnerName } else { 'UNATTRIBUTED' }
        [void]$sb.AppendLine("| $($f.Protocol) | $($f.LocalPort) | ${r}:$($f.RemotePort) | $($f.Frames) | $($f.Bytes) | $o |")
    }
    [void]$sb.AppendLine()

    [void]$sb.AppendLine("## Q-2: endpoint ownership")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("Hostnames come from DNS answers and TLS server names observed inside " +
                         "the capture, so no external lookup was performed. $($unnamed.Count) " +
                         "endpoints resolved to no name; those are the candidates for relay " +
                         "inspection.")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("| Remote | Hostname | Flows | Bytes | Owner |")
    [void]$sb.AppendLine("| --- | --- | --- | --- | --- |")
    foreach ($e in $topEnds) {
        $r = Format-Address -Address $e.RemoteAddress
        $h = if ($e.Hostname) { $e.Hostname } else { '(unresolved)' }
        $o = if ($e.Owners) { $e.Owners } else { 'UNATTRIBUTED' }
        [void]$sb.AppendLine("| $r | $h | $($e.Flows) | $($e.Bytes) | $o |")
    }
    [void]$sb.AppendLine()

    [void]$sb.AppendLine("## Q-3: connection lifetimes")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("- Connections observed opening and closing: $closed.")
    [void]$sb.AppendLine("- Shorter than $ShortConnectionMs ms: $shortCount ($shortPct%).")
    [void]$sb.AppendLine("- **Frames riding on those short connections: $shortFramePct%.**")
    [void]$sb.AppendLine("- Median lifetime $median ms, tenth percentile $p10 ms.")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("The frame-weighted figure is the one that decides A-2. Before " +
                         "concluding that polling is insufficient, check whether shortening the " +
                         "poll interval resolves it: the direct socket table call costs 1 to 3 " +
                         "milliseconds, so a much tighter cadence is affordable (PF-4).")
    [void]$sb.AppendLine()

    [void]$sb.AppendLine("## Q-5: loopback handoff")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("$loopSummary.")
    [void]$sb.AppendLine()
    if ($loopFlows.Count -eq 0 -and (Test-Path -LiteralPath $loopback)) {
        [void]$sb.AppendLine("No loopback conversations. If the launcher and client communicate " +
                             "by named pipe, shared memory, or command line argument, that is a " +
                             "documented scope boundary rather than a defect, and it belongs in " +
                             "the getting-started documentation so users do not expect something " +
                             "the tool cannot deliver.")
    }
    [void]$sb.AppendLine()

    [void]$sb.AppendLine("## Q-6: encryption posture")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("| Class | Sample bytes | Entropy (bits/byte) | Reading |")
    [void]$sb.AppendLine("| --- | --- | --- | --- |")
    foreach ($e in $entropy) {
        [void]$sb.AppendLine("| $($e.Class) | $($e.SampleBytes) | $($e.Entropy) | $($e.Reading) |")
    }
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("Above $($script:EntropyHigh) bits per byte indicates encrypted or " +
                         "compressed content; below $($script:EntropyLow) indicates structured " +
                         "or cleartext. The band between is genuinely ambiguous and is reported " +
                         "as inconclusive rather than rounded.")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("$tlsCount distinct TLS server names were observed. Where a title uses " +
                         "transport encryption, payloads are captured as ciphertext and fragcap " +
                         "does not decrypt them (specification section 19.6).")
    [void]$sb.AppendLine()

    [void]$sb.AppendLine("## Draft Appendix D entry")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("Review, scrub, and shorten before promoting into the specification.")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine('```text')
    [void]$sb.AppendLine("Title:                $($manifest.title)")
    [void]$sb.AppendLine("Session date:         $($manifest.startUtc)")
    [void]$sb.AppendLine("Game build:           TODO record the client version")
    [void]$sb.AppendLine("Process topology:     $(if ($netProcs.Count) { ($netProcs.Name | Sort-Object -Unique) -join ' -> ' } else { 'NOT RECORDED' })")
    [void]$sb.AppendLine("Transport protocols:  TCP $($tcpFlows.Count) conv, UDP $($udpFlows.Count) conv")
    [void]$sb.AppendLine("Endpoint ownership:   $($endpoints.Count) endpoints, $($unnamed.Count) unresolved")
    [void]$sb.AppendLine("Encryption posture:   $(($entropy | ForEach-Object { "$($_.Class)=$($_.Reading)" }) -join '; ')")
    [void]$sb.AppendLine("Lifetime median:      $median ms; frames on short connections $shortFramePct%")
    [void]$sb.AppendLine("A-1 (5-tuple):        $q1")
    [void]$sb.AppendLine("A-2 (lifetimes):      $q3")
    [void]$sb.AppendLine("A-3 (relay):          $q2")
    [void]$sb.AppendLine("A-4 (role separable): $(if ($netProcs.Count -gt 1) { 'review process table' } else { 'INCONCLUSIVE' })")
    [void]$sb.AppendLine("A-5 (loopback):       $q5")
    [void]$sb.AppendLine('```')

    $reportPath = Join-Path $SessionPath 'FINDINGS.md'
    [System.IO.File]::WriteAllText($reportPath, $sb.ToString().Replace($nl, "`n"),
        [System.Text.UTF8Encoding]::new($false))

    Write-Log "Report written: $reportPath" -Level Success -Source report
    Write-Log "Evidence tables: $analysis" -Level Info -Source report
    Write-Log "Q-1 $attrPct% attributed | Q-3 $shortFramePct% frames on short | Q-5 $($loopFlows.Count) loopback conv" -Level Success -Source report

    exit 0

#_______________________________________________________________________________
## End of script
