# SPDX-License-Identifier: Apache-2.0
<#
.SYNOPSIS
    Certify final fragcap Windows release packages and installer lifecycle behavior.

.DESCRIPTION
    Validates the final portable ZIP, MSI, standalone catalog, and checksum sidecars against the versioned repository contract. It safely inspects archive entries, verifies exact shared bytes, checks PE machine and imports with dumpbin, measures Authenticode state, and runs the packaged native controlled smoke under a constrained environment.

    The script then exercises clean install, repair, exact-byte reinstall, upgrade from the digest-pinned predecessor, downgrade refusal, and uninstall. Windows Installer children are hidden, non-interactive, time-bounded, and locally logged. Runner-specific MSI logs and scratch files are always removed and are never included in the public report.

    This script changes machine installer state, the system PATH through the MSI, and potentially one exact Windows Defender exclusion. Every hard-to-reverse action is guarded by ShouldProcess. Automation must pass -Confirm:$false; -WhatIf previews actions but cannot produce a successful certification report.

.PARAMETER ArtifactDirectory
    Directory containing exactly the candidate ZIP, MSI, catalog.db, and their three .sha256 sidecars. Alias: a

.PARAMETER PredecessorPath
    Path to the separately acquired digest-pinned v0.8.0 MSI used for upgrade and downgrade-refusal cases. Alias: p

.PARAMETER ReportPath
    Destination for the bounded public-safe JSON certification report. Alias: r

.PARAMETER ContractPath
    Package contract path. Defaults to integration/windows-package-contract-v1.json. Alias: c

.PARAMETER DumpbinPath
    Optional explicit dumpbin.exe path. When omitted, Visual Studio Installer discovery is used. Alias: d

.PARAMETER Quiet
    Suppress informational, success, and debug output while preserving warnings and errors. Alias: q

.PARAMETER Silent
    Suppress all logging except genuine errors.

.PARAMETER Help
    Print detailed help. Alias: h

.EXAMPLE
    pwsh -NoLogo -NoProfile -NonInteractive -File scripts/Test-PackageCertification.ps1 -ArtifactDirectory dist -PredecessorPath target/package-certification/fragcap-0.8.0-x86_64.msi -ReportPath dist/certification-report.json -Confirm:$false
    Runs the complete certification against the final candidate artifacts.

.EXAMPLE
    pwsh -NoLogo -NoProfile -File scripts/Test-PackageCertification.ps1 -ArtifactDirectory dist -PredecessorPath predecessor.msi -ReportPath report.json -WhatIf
    Previews installer and cleanup actions. The preview intentionally does not produce a passing report.

.OUTPUTS
    Writes operator progress to the host and one bounded JSON report to ReportPath on success.

.NOTES
    Requires PowerShell 7, administrator rights for per-machine MSI operations, Windows Installer, and Visual Studio dumpbin.exe. Exit 0 means complete certification, exit 1 means an assertion failed, and exit 2 means a prerequisite prevented execution.
#>
[CmdletBinding(SupportsShouldProcess=$true,ConfirmImpact='High',DefaultParameterSetName='Default')]
Param(
    [Parameter(Mandatory=$true,ParameterSetName='Default')]
    [Alias("a")]
    [string]$ArtifactDirectory,

    [Parameter(Mandatory=$true,ParameterSetName='Default')]
    [Alias("p")]
    [string]$PredecessorPath,

    [Parameter(Mandatory=$true,ParameterSetName='Default')]
    [Alias("r")]
    [string]$ReportPath,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("c")]
    [string]$ContractPath = 'integration/windows-package-contract-v1.json',

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("d")]
    [string]$DumpbinPath = '',

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

    function Write-ShruggieLog {
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
        $tag = if ($Source) { "[$Source] " } else { '' }
        $label = $Level.ToUpper().PadRight(7)
        $color = switch ($Level) {
            'Info' { 'Gray' }
            'Success' { 'Green' }
            'Warn' { 'Yellow' }
            'Error' { 'Red' }
            'Debug' { 'DarkGray' }
        }
        Write-Host ("{0} {1}{2} {3}" -f $stamp, $tag, $label, $Message) -ForegroundColor $color
    }

    function Resolve-FullLiteralPath {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$Value
        )
        return (Resolve-Path -LiteralPath $Value -ErrorAction Stop).Path
    }

    function Get-Sha256 {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$Path
        )
        return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    }

    function Test-IsLoopbackAddress {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$Address
        )
        $parsed = $null
        if (-not [System.Net.IPAddress]::TryParse($Address, [ref]$parsed)) { return $false }
        return [System.Net.IPAddress]::IsLoopback($parsed) -or $parsed.Equals([System.Net.IPAddress]::Any) -or $parsed.Equals([System.Net.IPAddress]::IPv6Any)
    }

    function Test-IsExactLoopbackAddress {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$Address
        )
        $parsed = $null
        return [System.Net.IPAddress]::TryParse($Address, [ref]$parsed) -and [System.Net.IPAddress]::IsLoopback($parsed)
    }

    function Invoke-HiddenProcess {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$FilePath,

            [Parameter(Mandatory=$true)]
            [string[]]$ArgumentList,

            [Parameter(Mandatory=$true)]
            [ValidateRange(1, 600)]
            [int]$TimeoutSeconds,

            [Parameter(Mandatory=$false)]
            [hashtable]$Environment = @{},

            [Parameter(Mandatory=$false)]
            [string]$ObservedExecutablePath
        )
        $start = [System.Diagnostics.ProcessStartInfo]::new()
        $start.FileName = $FilePath
        $start.UseShellExecute = $false
        $start.CreateNoWindow = $true
        $start.WindowStyle = [System.Diagnostics.ProcessWindowStyle]::Hidden
        $start.RedirectStandardOutput = $true
        $start.RedirectStandardError = $true
        foreach ($argument in $ArgumentList) { [void]$start.ArgumentList.Add($argument) }
        foreach ($name in $Environment.Keys) {
            if ($null -eq $Environment[$name]) { [void]$start.Environment.Remove($name) } else { $start.Environment[$name] = [string]$Environment[$name] }
        }
        $process = [System.Diagnostics.Process]::new()
        $process.StartInfo = $start
        $watch = [System.Diagnostics.Stopwatch]::StartNew()
        if (-not $process.Start()) { throw "could not start process" }
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        $waitTask = $process.WaitForExitAsync()
        $observedPaths = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
        $observedAddresses = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
        $observationSamples = 0
        if (-not [string]::IsNullOrEmpty($ObservedExecutablePath)) { [void]$observedPaths.Add([System.IO.Path]::GetFullPath($FilePath)) }
        while (-not $waitTask.IsCompleted) {
            if ($watch.Elapsed.TotalSeconds -ge $TimeoutSeconds) {
                try { $process.Kill($true) } catch { Write-ShruggieLog "Timed-out child could not be killed cleanly: $($_.Exception.Message)" -Level Warn -Source Child }
                throw "process exceeded $TimeoutSeconds seconds"
            }
            if (-not [string]::IsNullOrEmpty($ObservedExecutablePath)) {
                $snapshot = @(Get-CimInstance -ClassName Win32_Process -Property ProcessId,ParentProcessId,ExecutablePath -ErrorAction Stop)
                $owned = [System.Collections.Generic.HashSet[uint32]]::new()
                [void]$owned.Add([uint32]$process.Id)
                do {
                    $added = $false
                    foreach ($candidate in $snapshot) {
                        if ($owned.Contains([uint32]$candidate.ParentProcessId) -and $owned.Add([uint32]$candidate.ProcessId)) { $added = $true }
                    }
                } while ($added)
                foreach ($candidate in $snapshot | Where-Object { $owned.Contains([uint32]$_.ProcessId) }) {
                    if (-not [string]::IsNullOrEmpty($candidate.ExecutablePath)) { [void]$observedPaths.Add([System.IO.Path]::GetFullPath($candidate.ExecutablePath)) }
                }
                foreach ($connection in @(Get-NetTCPConnection -ErrorAction Stop | Where-Object { $owned.Contains([uint32]$_.OwningProcess) })) {
                    [void]$observedAddresses.Add([string]$connection.LocalAddress)
                    [void]$observedAddresses.Add([string]$connection.RemoteAddress)
                }
                foreach ($endpoint in @(Get-NetUDPEndpoint -ErrorAction Stop | Where-Object { $owned.Contains([uint32]$_.OwningProcess) })) { [void]$observedAddresses.Add([string]$endpoint.LocalAddress) }
                $observationSamples++
            }
            [System.Threading.Thread]::Sleep(100)
        }
        [void]$waitTask.GetAwaiter().GetResult()
        $process.WaitForExit()
        $watch.Stop()
        $stdout = $stdoutTask.GetAwaiter().GetResult()
        $stderr = $stderrTask.GetAwaiter().GetResult()
        if ($stdout.Length -gt 262144 -or $stderr.Length -gt 262144) { throw 'child output exceeded 256 KiB' }
        $exitCode = [int]$process.ExitCode
        $process.Dispose()
        $observation = $null
        if (-not [string]::IsNullOrEmpty($ObservedExecutablePath)) {
            $expectedPath = [System.IO.Path]::GetFullPath($ObservedExecutablePath)
            $unexpectedPaths = @($observedPaths | Where-Object { $_ -ine $expectedPath })
            $nonLoopbackAddresses = @($observedAddresses | Where-Object { -not (Test-IsLoopbackAddress -Address $_) })
            $loopbackAddresses = @($observedAddresses | Where-Object { Test-IsExactLoopbackAddress -Address $_ })
            $observation = [pscustomobject]@{ samples = $observationSamples; process_paths = @($observedPaths | Sort-Object); unexpected_process_paths = $unexpectedPaths; observed_addresses = @($observedAddresses | Sort-Object); non_loopback_addresses = $nonLoopbackAddresses; loopback_observed = $loopbackAddresses.Count -gt 0; complete = $observationSamples -gt 0 }
        }
        return [pscustomobject]@{ ExitCode = $exitCode; Stdout = $stdout; Stderr = $stderr; ElapsedSeconds = [math]::Ceiling($watch.Elapsed.TotalSeconds); Observation = $observation }
    }

    function Invoke-MsiOperation {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$Case,

            [Parameter(Mandatory=$true)]
            [string[]]$Arguments,

            [Parameter(Mandatory=$true)]
            [string]$LogPath,

            [Parameter(Mandatory=$false)]
            [int[]]$AllowedExitCodes = @(0, 3010)
        )
        if (-not $script:TopLevelCmdlet.ShouldProcess(($Arguments -join ' '), "Run hidden Windows Installer case $Case")) { throw "$Case was not executed" }
        $allArguments = @($Arguments) + @('/qn', '/norestart', '/L*V', $LogPath)
        $result = Invoke-HiddenProcess -FilePath "$env:SystemRoot\System32\msiexec.exe" -ArgumentList $allArguments -TimeoutSeconds 600
        if ($AllowedExitCodes -notcontains $result.ExitCode) { throw "$Case exited $($result.ExitCode)" }
        return $result
    }

    function Get-MsiProperty {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$MsiPath,

            [Parameter(Mandatory=$true)]
            [string]$Name
        )
        $installer = New-Object -ComObject WindowsInstaller.Installer
        $database = $installer.GetType().InvokeMember('OpenDatabase', 'InvokeMethod', $null, $installer, @($MsiPath, 0))
        $query = "SELECT ``Value`` FROM ``Property`` WHERE ``Property``='$Name'"
        $view = $database.GetType().InvokeMember('OpenView', 'InvokeMethod', $null, $database, @($query))
        [void]$view.GetType().InvokeMember('Execute', 'InvokeMethod', $null, $view, $null)
        $record = $view.GetType().InvokeMember('Fetch', 'InvokeMethod', $null, $view, $null)
        if ($null -eq $record) { throw "MSI property $Name is absent" }
        return [string]$record.GetType().InvokeMember('StringData', 'GetProperty', $null, $record, 1)
    }

    function Get-ProductRegistrationCount {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$ProductCode
        )
        $paths = @("HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\$ProductCode", "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\$ProductCode")
        return @($paths | Where-Object { Test-Path -LiteralPath $_ }).Count
    }

    function Get-SystemPathCount {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$ExpectedPath
        )
        $machinePath = [Environment]::GetEnvironmentVariable('Path', 'Machine')
        $normalized = $ExpectedPath.TrimEnd('\')
        return @(($machinePath -split ';') | Where-Object { $_.TrimEnd('\') -ieq $normalized }).Count
    }

    function Assert-ExactInstalledFiles {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$InstallDirectory,

            [Parameter(Mandatory=$true)]
            [string]$ReferenceDirectory,

            [Parameter(Mandatory=$true)]
            [string[]]$ExpectedPaths
        )
        $observed = @(Get-ChildItem -LiteralPath $InstallDirectory -Recurse -File | ForEach-Object { [System.IO.Path]::GetRelativePath($InstallDirectory, $_.FullName).Replace('\', '/') } | Sort-Object)
        if (Compare-Object -ReferenceObject @($ExpectedPaths | Sort-Object) -DifferenceObject $observed) { throw "recursive installed file inventory does not match the contract" }
        foreach ($path in $ExpectedPaths) {
            $installed = [System.IO.Path]::Combine($InstallDirectory, $path)
            $reference = [System.IO.Path]::Combine($ReferenceDirectory, $path)
            if ((Get-Sha256 -Path $installed) -ne (Get-Sha256 -Path $reference)) { throw "installed $path differs from the portable package" }
        }
    }

    function Get-ZipContent {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$ZipPath,

            [Parameter(Mandatory=$true)]
            [string]$Destination,

            [Parameter(Mandatory=$true)]
            [object[]]$ExpectedEntries
        )
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        [void][System.IO.Directory]::CreateDirectory($Destination)
        $archive = [System.IO.Compression.ZipFile]::OpenRead($ZipPath)
        try {
            $observed = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
            foreach ($entry in $archive.Entries) {
                if ([string]::IsNullOrEmpty($entry.Name) -or $entry.FullName -ne $entry.Name -or [System.IO.Path]::IsPathRooted($entry.FullName) -or $entry.FullName.Contains('..') -or $entry.FullName.Contains('\')) { throw "unsafe ZIP entry $($entry.FullName)" }
                if (-not $observed.Add($entry.FullName)) { throw "duplicate ZIP entry $($entry.FullName)" }
                $expected = @($ExpectedEntries | Where-Object { $_.path -ceq $entry.FullName })
                if ($expected.Count -ne 1) { throw "undeclared ZIP entry $($entry.FullName)" }
                if ($entry.Length -gt [int64]$expected[0].size_ceiling_bytes) { throw "ZIP entry $($entry.FullName) exceeds its size ceiling" }
                $destinationPath = [System.IO.Path]::Combine($Destination, $entry.Name)
                $sourceStream = $entry.Open()
                $destinationStream = [System.IO.File]::Open($destinationPath, [System.IO.FileMode]::CreateNew, [System.IO.FileAccess]::Write, [System.IO.FileShare]::None)
                try { $sourceStream.CopyTo($destinationStream) } finally { $destinationStream.Dispose(); $sourceStream.Dispose() }
            }
            if ($observed.Count -ne $ExpectedEntries.Count) { throw 'ZIP is missing a required entry' }
        } finally {
            $archive.Dispose()
        }
    }

    function Assert-Unsigned {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$Path
        )
        $signature = Get-AuthenticodeSignature -LiteralPath $Path
        if ($signature.Status -ne 'NotSigned' -or $null -ne $signature.SignerCertificate) { throw "signature state for $([System.IO.Path]::GetFileName($Path)) is not determinately unsigned" }
    }

    function Get-PeInspection {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$ExecutablePath,

            [Parameter(Mandatory=$true)]
            [string]$Dumper,

            [Parameter(Mandatory=$true)]
            [pscustomobject]$Policy,

            [Parameter(Mandatory=$true)]
            [string]$Surface
        )
        $headers = Invoke-HiddenProcess -FilePath $Dumper -ArgumentList @('/headers', $ExecutablePath) -TimeoutSeconds 60
        $dependencies = Invoke-HiddenProcess -FilePath $Dumper -ArgumentList @('/dependents', $ExecutablePath) -TimeoutSeconds 60
        if ($headers.ExitCode -ne 0 -or $dependencies.ExitCode -ne 0) { throw 'dumpbin could not inspect the packaged executable' }
        if ($headers.Stdout -notmatch '(?im)^\s*8664 machine \(x64\)') { throw 'packaged executable is not x86_64 PE machine 8664' }
        $ordinary = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
        $delayed = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
        $section = ''
        foreach ($line in ($dependencies.Stdout -split "`r?`n")) {
            if ($line -match 'Image has the following dependencies:') { $section = 'ordinary'; continue }
            if ($line -match 'Image has the following delay load dependencies:') { $section = 'delayed'; continue }
            if ($line -match '^\s+([A-Za-z0-9_.-]+\.dll)\s*$') {
                $name = $Matches[1].ToLowerInvariant()
                if ($section -eq 'ordinary') { [void]$ordinary.Add($name) }
                if ($section -eq 'delayed') { [void]$delayed.Add($name) }
            }
        }
        $expectedOrdinary = @($Policy.ordinary | Sort-Object)
        $expectedDelayed = @($Policy.delayed | Sort-Object)
        if (Compare-Object -ReferenceObject $expectedOrdinary -DifferenceObject @($ordinary | Sort-Object)) { throw 'ordinary PE imports differ from the closed allowlist' }
        if (Compare-Object -ReferenceObject $expectedDelayed -DifferenceObject @($delayed | Sort-Object)) { throw 'delayed PE imports differ from the closed allowlist' }
        Assert-Unsigned -Path $ExecutablePath
        $version = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($ExecutablePath)
        return [pscustomobject]@{ surface = $Surface; machine = '8664'; ordinary_imports = $expectedOrdinary; delayed_imports = $expectedDelayed; file_version = $version.FileVersion; product_version = $version.ProductVersion; product_name = $version.ProductName; original_filename = $version.OriginalFilename; signature = 'not_signed'; complete = $true }
    }

    function Invoke-Fragcap {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$Executable,

            [Parameter(Mandatory=$true)]
            [string[]]$Arguments,

            [Parameter(Mandatory=$true)]
            [hashtable]$Environment,

            [Parameter(Mandatory=$false)]
            [int[]]$AllowedExitCodes = @(0),

            [Parameter(Mandatory=$false)]
            [switch]$ObserveTreeAndNetwork
        )
        $observedPath = if ($ObserveTreeAndNetwork) { $Executable } else { $null }
        $result = Invoke-HiddenProcess -FilePath $Executable -ArgumentList $Arguments -TimeoutSeconds 60 -Environment $Environment -ObservedExecutablePath $observedPath
        if ($AllowedExitCodes -notcontains $result.ExitCode) { throw "fragcap $($Arguments -join ' ') exited $($result.ExitCode): $($result.Stderr)" }
        return $result
    }

    function Test-UserFixture {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [hashtable]$Digests
        )
        foreach ($path in $Digests.Keys) {
            if (-not (Test-Path -LiteralPath $path) -or (Get-Sha256 -Path $path) -ne $Digests[$path]) { throw 'installer changed user-owned fixture state' }
        }
    }

#_______________________________________________________________________________
## Declare Variables and Arrays

    $ThisScriptPath = $MyInvocation.MyCommand.Path
    $script:LogQuiet = [bool]$Quiet
    $script:LogSilent = [bool]$Silent
    $script:TopLevelCmdlet = $PSCmdlet
    $scratch = $null
    $seededDefender = $false
    $installDirectory = $null
    $currentProductCode = $null
    $predecessorProductCode = $null
    $smokeFirewallRuleName = $null

#_______________________________________________________________________________
## Execute Operations

    if (($Help) -or ($PSCmdlet.ParameterSetName -eq 'HelpText')) {
        Get-Help $ThisScriptPath -Detailed
        exit 0
    }
    Assert-PSVersion
    $ErrorActionPreference = 'Stop'
    Set-StrictMode -Version Latest
    try {
        if (-not $IsWindows) { Write-ShruggieLog 'Package certification requires Windows.' -Level Error -Source Preflight; exit 2 }
        $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = [Security.Principal.WindowsPrincipal]::new($identity)
        if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { Write-ShruggieLog 'Package certification requires an administrator runner.' -Level Error -Source Preflight; exit 2 }
        $artifactRoot = Resolve-FullLiteralPath -Value $ArtifactDirectory
        $predecessor = Resolve-FullLiteralPath -Value $PredecessorPath
        $contractFile = Resolve-FullLiteralPath -Value $ContractPath
        $reportFile = [System.IO.Path]::GetFullPath($ReportPath)
        $contractBytes = [System.IO.File]::ReadAllBytes($contractFile)
        $contract = [System.Text.Encoding]::UTF8.GetString($contractBytes) | ConvertFrom-Json -Depth 32
        if ($contract.schema_version -ne 1) { throw 'unsupported package contract schema' }
        if ((Get-Sha256 -Path $predecessor) -ne $contract.predecessor.sha256 -or (Get-Item -LiteralPath $predecessor).Length -ne $contract.predecessor.size_bytes) { throw 'predecessor MSI does not match the pinned identity' }
        if ([string]::IsNullOrEmpty($DumpbinPath)) {
            $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
            if (-not (Test-Path -LiteralPath $vswhere)) { Write-ShruggieLog 'Visual Studio Installer discovery is unavailable.' -Level Error -Source Preflight; exit 2 }
            $found = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -find 'VC\Tools\MSVC\**\bin\Hostx64\x64\dumpbin.exe'
            $DumpbinPath = @($found | Select-Object -Last 1)[0]
        }
        $dumper = Resolve-FullLiteralPath -Value $DumpbinPath
        $candidateMsi = @(Get-ChildItem -LiteralPath $artifactRoot -File -Filter 'fragcap-*-x86_64.msi')
        $candidateZip = @(Get-ChildItem -LiteralPath $artifactRoot -File -Filter 'fragcap-*-x86_64-pc-windows-msvc.zip')
        $catalog = @(Get-ChildItem -LiteralPath $artifactRoot -File -Filter 'catalog.db')
        if ($candidateMsi.Count -ne 1 -or $candidateZip.Count -ne 1 -or $catalog.Count -ne 1) { throw 'artifact directory must contain exactly one MSI, one portable ZIP, and catalog.db' }
        $primary = @($candidateZip[0], $candidateMsi[0], $catalog[0])
        $allFiles = @(Get-ChildItem -LiteralPath $artifactRoot -File)
        if ($allFiles.Count -ne 6) { throw 'artifact directory must contain exactly three primary artifacts and three checksum sidecars before reporting' }
        foreach ($file in $primary) {
            $sidecar = "$($file.FullName).sha256"
            if (-not (Test-Path -LiteralPath $sidecar)) { throw "missing checksum sidecar for $($file.Name)" }
            $line = ([System.IO.File]::ReadAllText($sidecar, [System.Text.Encoding]::UTF8)).TrimEnd("`r", "`n")
            if ($line -cnotmatch '^([0-9a-f]{64})  ([^/\\]+)$' -or $Matches[2] -cne $file.Name -or $Matches[1] -cne (Get-Sha256 -Path $file.FullName)) { throw "invalid checksum sidecar for $($file.Name)" }
        }
        foreach ($artifact in $contract.primary_artifacts) {
            $match = switch ($artifact.id) { 'portable-zip' { $candidateZip[0] } 'windows-msi' { $candidateMsi[0] } 'standalone-catalog' { $catalog[0] } }
            if ($match.Length -gt [int64]$artifact.size_ceiling_bytes) { throw "$($artifact.id) exceeds its size ceiling" }
        }
        $scratch = Join-Path ([System.IO.Path]::GetTempPath()) ("fragcap-package-{0}" -f [guid]::NewGuid().ToString('N'))
        [void][System.IO.Directory]::CreateDirectory($scratch)
        $zipRoot = Join-Path $scratch 'zip'
        Get-ZipContent -ZipPath $candidateZip[0].FullName -Destination $zipRoot -ExpectedEntries $contract.shared_entries
        if ((Get-Sha256 -Path (Join-Path $zipRoot 'catalog.db')) -ne (Get-Sha256 -Path $catalog[0].FullName)) { throw 'standalone catalog differs from portable package catalog' }
        $zipExe = Join-Path $zipRoot 'fragcap.exe'
        Assert-Unsigned -Path $candidateMsi[0].FullName
        $zipPe = Get-PeInspection -ExecutablePath $zipExe -Dumper $dumper -Policy $contract.pe_imports -Surface 'portable-zip'
        $cleanEnvironment = @{ Path = "$env:SystemRoot\System32;$env:SystemRoot"; APPDATA = (Join-Path $scratch 'appdata'); LOCALAPPDATA = (Join-Path $scratch 'localappdata'); FRAGCAP_CONTROLLED_TARGET_EXECUTABLE = $zipExe; HTTP_PROXY = $null; HTTPS_PROXY = $null; ALL_PROXY = $null }
        [void][System.IO.Directory]::CreateDirectory($cleanEnvironment.APPDATA)
        [void][System.IO.Directory]::CreateDirectory($cleanEnvironment.LOCALAPPDATA)
        $buildResult = Invoke-Fragcap -Executable $zipExe -Arguments @('__build-identity') -Environment $cleanEnvironment
        $buildIdentity = $buildResult.Stdout | ConvertFrom-Json
        if (-not $buildIdentity.official -or $buildIdentity.target -ne $contract.release_identity.target -or $buildIdentity.architecture -ne $contract.release_identity.architecture -or $buildIdentity.deep_capture_backend -ne $contract.release_identity.deep_capture_backend -or (Compare-Object -ReferenceObject @($contract.release_identity.features | Sort-Object) -DifferenceObject @($buildIdentity.features | Sort-Object))) { throw 'packaged binary build identity differs from the release contract' }
        $expectedMsiName = "fragcap-$($buildIdentity.version)-x86_64.msi"
        $expectedZipName = "fragcap-$($buildIdentity.version)-x86_64-pc-windows-msvc.zip"
        if ($candidateMsi[0].Name -cne $expectedMsiName -or $candidateZip[0].Name -cne $expectedZipName) { throw 'artifact filename disagrees with packaged binary version' }
        $versionInfo = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($zipExe)
        if ($versionInfo.FileVersion -ne "$($buildIdentity.version).0" -or $versionInfo.ProductVersion -ne $buildIdentity.version -or $versionInfo.ProductName -ne 'fragcap' -or $versionInfo.OriginalFilename -ne 'fragcap.exe') { throw 'PE version resource disagrees with build identity' }
        $doctor = Invoke-Fragcap -Executable $zipExe -Arguments @('--json', 'doctor') -Environment $cleanEnvironment -AllowedExitCodes @(0, 1)
        if ($doctor.Stdout -notmatch 'fragcap-native') { throw 'packaged Doctor output does not expose the native backend' }
        $localDb = Join-Path $scratch 'local.db'
        $bundle = Join-Path $scratch 'bundle'
        [void](Invoke-Fragcap -Executable $zipExe -Arguments @('targets', 'add', 'Package Certification', '--db', $localDb, '--anchor', 'package:certification', '--exe', $zipExe, '--socket-holder', 'yes') -Environment $cleanEnvironment)
        if ($null -eq (Get-Command New-NetFirewallRule -ErrorAction SilentlyContinue) -or $null -eq (Get-Command Remove-NetFirewallRule -ErrorAction SilentlyContinue)) { Write-ShruggieLog 'Windows Firewall observation controls are unavailable.' -Level Error -Source Preflight; exit 2 }
        $smokeFirewallRuleName = "fragcap-package-certification-$([guid]::NewGuid().ToString('N'))"
        if (-not $PSCmdlet.ShouldProcess($zipExe, 'Block non-loopback smoke traffic for the exact packaged executable')) { throw 'smoke network containment was not established' }
        [void](New-NetFirewallRule -Name $smokeFirewallRuleName -DisplayName $smokeFirewallRuleName -Direction Outbound -Action Block -Program $zipExe -RemoteAddress @('Internet','LocalSubnet') -Profile Any -Enabled True -ErrorAction Stop)
        $smoke = Invoke-Fragcap -Executable $zipExe -Arguments @('--json', 'deep-capture', 'package_certification', '--launch', '--calibrate', 'reachability', '--calibration-protocol', 'routing', '--launch-case', 'direct-exe-warm', '--duration', '5s', '--wait', '7s', '--yes', '--controlled-target', '--local-db', $localDb, '--bundle', $bundle) -Environment $cleanEnvironment -ObserveTreeAndNetwork
        if ($smoke.Stderr -notmatch 'fragcap-native' -or $smoke.Stderr -notmatch 'reached-client') { throw 'packaged controlled native smoke did not produce expected evidence' }
        if (-not $smoke.Observation.complete -or $smoke.Observation.samples -lt 1 -or $smoke.Observation.process_paths.Count -lt 1 -or $smoke.Observation.unexpected_process_paths.Count -ne 0) { throw 'packaged controlled native smoke escaped the observed process boundary' }
        if ($PSCmdlet.ShouldProcess($smokeFirewallRuleName, 'Remove package-certification smoke firewall rule')) { Remove-NetFirewallRule -Name $smokeFirewallRuleName -ErrorAction Stop; $smokeFirewallRuleName = $null }
        $installDirectory = Join-Path $scratch 'installed'
        $userFixturePaths = [ordered]@{
            'capture' = Join-Path $cleanEnvironment.LOCALAPPDATA 'fragcap\captures\preserved.fcapng'
            'deep-capture-bundle' = Join-Path $cleanEnvironment.APPDATA 'fragcap\sessions\preserved\manifest.json'
            'extcap-registration' = Join-Path $cleanEnvironment.APPDATA 'Wireshark\extcap\fragcap.exe'
            'local-database' = Join-Path $cleanEnvironment.APPDATA 'fragcap\local.db'
            'writable-catalog' = Join-Path $cleanEnvironment.APPDATA 'fragcap\catalog.db'
        }
        $expectedFileFixtureNames = @($contract.user_owned_fixtures | Where-Object { $_ -ne 'preexisting-defender-exclusion' } | Sort-Object)
        if (Compare-Object -ReferenceObject $expectedFileFixtureNames -DifferenceObject @($userFixturePaths.Keys | Sort-Object)) { throw 'user-owned fixture path map does not cover the contract' }
        $userDigests = @{}
        foreach ($name in $userFixturePaths.Keys) {
            $path = $userFixturePaths[$name]
            [void][System.IO.Directory]::CreateDirectory([System.IO.Path]::GetDirectoryName($path))
            [System.IO.File]::WriteAllText($path, "S131 user-owned fixture: $name", [System.Text.UTF8Encoding]::new($false))
            $userDigests[$path] = Get-Sha256 -Path $path
        }
        $defenderAvailable = $null -ne (Get-Command Get-MpPreference -ErrorAction SilentlyContinue) -and $null -ne (Get-Command Add-MpPreference -ErrorAction SilentlyContinue)
        if ($defenderAvailable) {
            $alreadyExcluded = @((Get-MpPreference).ExclusionPath) -contains $installDirectory
            if (-not $alreadyExcluded -and $PSCmdlet.ShouldProcess($installDirectory, 'Seed exact pre-existing administrator-owned Defender exclusion')) {
                Add-MpPreference -ExclusionPath $installDirectory
                $seededDefender = @((Get-MpPreference).ExclusionPath) -contains $installDirectory
            }
        }
        $currentProductCode = Get-MsiProperty -MsiPath $candidateMsi[0].FullName -Name 'ProductCode'
        $predecessorProductCode = Get-MsiProperty -MsiPath $predecessor -Name 'ProductCode'
        if ((Get-MsiProperty -MsiPath $candidateMsi[0].FullName -Name 'ProductName') -cne 'fragcap' -or (Get-MsiProperty -MsiPath $candidateMsi[0].FullName -Name 'ProductVersion') -cne $buildIdentity.version -or (Get-MsiProperty -MsiPath $candidateMsi[0].FullName -Name 'Manufacturer') -cne 'ShruggieTech' -or (Get-MsiProperty -MsiPath $candidateMsi[0].FullName -Name 'UpgradeCode') -cne '{7F3A2C4E-1D9B-4A6E-9C58-2B0E7D4F6A13}') { throw 'MSI metadata disagrees with the certified release identity' }
        $lifecycle = [System.Collections.Generic.List[object]]::new()
        $clean = Invoke-MsiOperation -Case 'clean-install' -Arguments @('/i', $candidateMsi[0].FullName, "INSTALLDIR=$installDirectory\") -LogPath (Join-Path $scratch 'clean-install.log')
        Assert-ExactInstalledFiles -InstallDirectory $installDirectory -ReferenceDirectory $zipRoot -ExpectedPaths @($contract.shared_entries.path)
        if ((Get-ProductRegistrationCount -ProductCode $currentProductCode) -ne 1 -or (Get-SystemPathCount -ExpectedPath $installDirectory) -ne 1) { throw 'clean install did not create one exact product and PATH entry' }
        Test-UserFixture -Digests $userDigests
        if ($seededDefender -and -not (@((Get-MpPreference).ExclusionPath) -contains $installDirectory)) { throw 'clean install removed the pre-existing Defender exclusion' }
        $lifecycle.Add([pscustomobject]@{ id = 'clean-install'; terminal = 'passed'; cleanup = 'reconciled'; elapsed_seconds = [int]$clean.ElapsedSeconds; complete = $true })
        if ($PSCmdlet.ShouldProcess((Join-Path $installDirectory 'NOTICE'), 'Delete owned file before repair')) { Remove-Item -LiteralPath (Join-Path $installDirectory 'NOTICE') -Force }
        if ($PSCmdlet.ShouldProcess((Join-Path $installDirectory 'LICENSE'), 'Alter owned file before repair')) { [System.IO.File]::WriteAllText((Join-Path $installDirectory 'LICENSE'), 'altered', [System.Text.UTF8Encoding]::new($false)) }
        $repair = Invoke-MsiOperation -Case 'repair' -Arguments @('/fa', $candidateMsi[0].FullName) -LogPath (Join-Path $scratch 'repair.log')
        Assert-ExactInstalledFiles -InstallDirectory $installDirectory -ReferenceDirectory $zipRoot -ExpectedPaths @($contract.shared_entries.path)
        Test-UserFixture -Digests $userDigests
        $lifecycle.Add([pscustomobject]@{ id = 'repair'; terminal = 'passed'; cleanup = 'reconciled'; elapsed_seconds = [int]$repair.ElapsedSeconds; complete = $true })
        $reinstall = Invoke-MsiOperation -Case 'same-version-reinstall' -Arguments @('/i', $candidateMsi[0].FullName, "INSTALLDIR=$installDirectory\") -LogPath (Join-Path $scratch 'reinstall.log')
        Assert-ExactInstalledFiles -InstallDirectory $installDirectory -ReferenceDirectory $zipRoot -ExpectedPaths @($contract.shared_entries.path)
        if ((Get-ProductRegistrationCount -ProductCode $currentProductCode) -ne 1 -or (Get-SystemPathCount -ExpectedPath $installDirectory) -ne 1) { throw 'same-version reinstall duplicated product state' }
        Test-UserFixture -Digests $userDigests
        $lifecycle.Add([pscustomobject]@{ id = 'same-version-reinstall'; terminal = 'passed'; cleanup = 'reconciled'; elapsed_seconds = [int]$reinstall.ElapsedSeconds; complete = $true })
        [void](Invoke-MsiOperation -Case 'reset-before-upgrade' -Arguments @('/x', $candidateMsi[0].FullName) -LogPath (Join-Path $scratch 'reset.log'))
        if ((Get-ProductRegistrationCount -ProductCode $currentProductCode) -ne 0 -or (Get-SystemPathCount -ExpectedPath $installDirectory) -ne 0) { throw 'reset before upgrade left candidate-owned state' }
        if ($seededDefender) {
            if (-not (@((Get-MpPreference).ExclusionPath) -contains $installDirectory)) { throw 'uninstall removed a pre-existing administrator-owned Defender exclusion' }
            if ($PSCmdlet.ShouldProcess($installDirectory, 'Remove the certification-seeded Defender exclusion before owned-effect testing')) { Remove-MpPreference -ExclusionPath $installDirectory; $seededDefender = $false }
        }
        [void](Invoke-MsiOperation -Case 'install-predecessor' -Arguments @('/i', $predecessor, "INSTALLDIR=$installDirectory\") -LogPath (Join-Path $scratch 'predecessor.log'))
        $upgrade = Invoke-MsiOperation -Case 'upgrade' -Arguments @('/i', $candidateMsi[0].FullName, "INSTALLDIR=$installDirectory\") -LogPath (Join-Path $scratch 'upgrade.log')
        Assert-ExactInstalledFiles -InstallDirectory $installDirectory -ReferenceDirectory $zipRoot -ExpectedPaths @($contract.shared_entries.path)
        if ((Get-ProductRegistrationCount -ProductCode $currentProductCode) -ne 1 -or (Get-ProductRegistrationCount -ProductCode $predecessorProductCode) -ne 0) { throw 'upgrade did not replace the predecessor product exactly' }
        Test-UserFixture -Digests $userDigests
        $lifecycle.Add([pscustomobject]@{ id = 'upgrade'; terminal = 'passed'; cleanup = 'reconciled'; elapsed_seconds = [int]$upgrade.ElapsedSeconds; complete = $true })
        $downgradeWatch = [System.Diagnostics.Stopwatch]::StartNew()
        [void](Invoke-MsiOperation -Case 'downgrade-refusal' -Arguments @('/i', $predecessor, "INSTALLDIR=$installDirectory\") -LogPath (Join-Path $scratch 'downgrade.log') -AllowedExitCodes @(1603, 1638))
        $downgradeWatch.Stop()
        if ((Get-ProductRegistrationCount -ProductCode $currentProductCode) -ne 1 -or (Get-ProductRegistrationCount -ProductCode $predecessorProductCode) -ne 0) { throw 'older predecessor was not refused without changing the current product' }
        Assert-ExactInstalledFiles -InstallDirectory $installDirectory -ReferenceDirectory $zipRoot -ExpectedPaths @($contract.shared_entries.path)
        Test-UserFixture -Digests $userDigests
        $lifecycle.Add([pscustomobject]@{ id = 'downgrade-refusal'; terminal = 'refused_as_expected'; cleanup = 'reconciled'; elapsed_seconds = [int][math]::Ceiling($downgradeWatch.Elapsed.TotalSeconds); complete = $true })
        $installedExecutable = Join-Path $installDirectory 'fragcap.exe'
        $installedPe = Get-PeInspection -ExecutablePath $installedExecutable -Dumper $dumper -Policy $contract.pe_imports -Surface 'installed-msi'
        $installedIdentity = (Invoke-Fragcap -Executable $installedExecutable -Arguments @('__build-identity') -Environment $cleanEnvironment).Stdout | ConvertFrom-Json
        if (($installedIdentity | ConvertTo-Json -Compress) -cne ($buildIdentity | ConvertTo-Json -Compress)) { throw 'installed executable identity differs from the portable executable identity' }
        $uninstall = Invoke-MsiOperation -Case 'uninstall' -Arguments @('/x', $candidateMsi[0].FullName) -LogPath (Join-Path $scratch 'uninstall.log')
        if ((Test-Path -LiteralPath $installDirectory) -or (Get-ProductRegistrationCount -ProductCode $currentProductCode) -ne 0 -or (Get-SystemPathCount -ExpectedPath $installDirectory) -ne 0) { throw 'uninstall left installer-owned files, registration, or PATH state' }
        $markerPath = 'HKLM:\Software\ShruggieTech\fragcap'
        $marker = Get-ItemProperty -LiteralPath $markerPath -Name 'FRAGCAP_DEFENDER_EXCLUSION_OWNER' -ErrorAction SilentlyContinue
        if ($null -ne $marker -and $marker.FRAGCAP_DEFENDER_EXCLUSION_OWNER) { throw 'uninstall left the Defender ownership marker' }
        if ($defenderAvailable -and (@((Get-MpPreference).ExclusionPath) -contains $installDirectory)) { throw 'uninstall left an installer-owned Defender exclusion' }
        Test-UserFixture -Digests $userDigests
        $lifecycle.Add([pscustomobject]@{ id = 'uninstall'; terminal = 'passed'; cleanup = 'reconciled'; elapsed_seconds = [int]$uninstall.ElapsedSeconds; complete = $true })
        $artifactRows = @(
            [pscustomobject]@{ id = 'portable-zip'; filename = $candidateZip[0].Name; size_bytes = $candidateZip[0].Length; sha256 = Get-Sha256 -Path $candidateZip[0].FullName; signature = 'not_applicable'; complete = $true },
            [pscustomobject]@{ id = 'windows-msi'; filename = $candidateMsi[0].Name; size_bytes = $candidateMsi[0].Length; sha256 = Get-Sha256 -Path $candidateMsi[0].FullName; signature = 'not_signed'; complete = $true },
            [pscustomobject]@{ id = 'standalone-catalog'; filename = $catalog[0].Name; size_bytes = $catalog[0].Length; sha256 = Get-Sha256 -Path $catalog[0].FullName; signature = 'not_applicable'; complete = $true },
            [pscustomobject]@{ id = 'portable-zip-checksum'; filename = "$($candidateZip[0].Name).sha256"; size_bytes = (Get-Item -LiteralPath "$($candidateZip[0].FullName).sha256").Length; sha256 = Get-Sha256 -Path "$($candidateZip[0].FullName).sha256"; signature = 'not_applicable'; complete = $true },
            [pscustomobject]@{ id = 'windows-msi-checksum'; filename = "$($candidateMsi[0].Name).sha256"; size_bytes = (Get-Item -LiteralPath "$($candidateMsi[0].FullName).sha256").Length; sha256 = Get-Sha256 -Path "$($candidateMsi[0].FullName).sha256"; signature = 'not_applicable'; complete = $true },
            [pscustomobject]@{ id = 'standalone-catalog-checksum'; filename = "$($catalog[0].Name).sha256"; size_bytes = (Get-Item -LiteralPath "$($catalog[0].FullName).sha256").Length; sha256 = Get-Sha256 -Path "$($catalog[0].FullName).sha256"; signature = 'not_applicable'; complete = $true }
        )
        $entryRows = @($contract.shared_entries | ForEach-Object { $entryFile = Get-Item -LiteralPath (Join-Path $zipRoot $_.path); [pscustomobject]@{ path = $_.path; role = $_.role; size_bytes = $entryFile.Length; sha256 = Get-Sha256 -Path $entryFile.FullName; signature = $_.signature; complete = $true } })
        $reportIdentity = [ordered]@{ product = $contract.release_identity.product; target = $contract.release_identity.target; architecture = $contract.release_identity.architecture; pe_machine = $contract.release_identity.pe_machine; features = @($contract.release_identity.features); deep_capture_backend = $contract.release_identity.deep_capture_backend }
        $report = [ordered]@{ schema_version = 1; contract_sha256 = Get-Sha256 -Path $contractFile; release_identity = $reportIdentity; build_identity = $buildIdentity; artifacts = $artifactRows; entries = $entryRows; pe_inspections = @($zipPe, $installedPe); smoke = [ordered]@{ backend = 'fragcap-native'; network = 'loopback-only'; process_observation = 'complete'; network_observation = 'firewall-contained-and-socket-observed'; samples = $smoke.Observation.samples; observed_endpoint_count = $smoke.Observation.observed_addresses.Count; observed_non_loopback_attempt_count = $smoke.Observation.non_loopback_addresses.Count; loopback_socket_observed = $smoke.Observation.loopback_observed; complete = $true }; lifecycle = @($lifecycle); findings = @(); complete = $true }
        $json = $report | ConvertTo-Json -Depth 16
        if ([System.Text.Encoding]::UTF8.GetByteCount($json) -gt [int]$contract.report_limits.max_report_bytes) { throw 'certification report exceeds its byte bound' }
        [void][System.IO.Directory]::CreateDirectory([System.IO.Path]::GetDirectoryName($reportFile))
        [System.IO.File]::WriteAllText($reportFile, "$json`n", [System.Text.UTF8Encoding]::new($false))
        Write-ShruggieLog 'Final package bytes and installer lifecycle are certified.' -Level Success -Source Complete
        exit 0
    } catch {
        Write-ShruggieLog "$($_.Exception.Message) [$($_.InvocationInfo.ScriptLineNumber)]" -Level Error -Source Certification
        exit 1
    } finally {
        if ($smokeFirewallRuleName -and $PSCmdlet.ShouldProcess($smokeFirewallRuleName, 'Remove package-certification smoke firewall rule during final cleanup')) { try { Remove-NetFirewallRule -Name $smokeFirewallRuleName -ErrorAction Stop } catch { Write-ShruggieLog "Firewall cleanup failed: $($_.Exception.Message)" -Level Warn -Source Cleanup } }
        foreach ($cleanupProductCode in @($currentProductCode, $predecessorProductCode) | Where-Object { $_ } | Select-Object -Unique) {
            if ((Get-ProductRegistrationCount -ProductCode $cleanupProductCode) -gt 0 -and $PSCmdlet.ShouldProcess($cleanupProductCode, 'Uninstall registered certification product during final cleanup')) {
                try {
                    $cleanupResult = Invoke-HiddenProcess -FilePath "$env:SystemRoot\System32\msiexec.exe" -ArgumentList @('/x', $cleanupProductCode, '/qn', '/norestart') -TimeoutSeconds 600
                    if (@(0, 1605, 3010) -notcontains $cleanupResult.ExitCode) { Write-ShruggieLog "Cleanup uninstall for $cleanupProductCode exited $($cleanupResult.ExitCode)." -Level Warn -Source Cleanup }
                } catch { Write-ShruggieLog "Cleanup uninstall for $cleanupProductCode failed: $($_.Exception.Message)" -Level Warn -Source Cleanup }
            }
        }
        if ($installDirectory) {
            $cleanupMarkerPath = 'HKLM:\Software\ShruggieTech\fragcap'
            $cleanupMarker = Get-ItemProperty -LiteralPath $cleanupMarkerPath -Name 'FRAGCAP_DEFENDER_EXCLUSION_OWNER' -ErrorAction SilentlyContinue
            if ($null -ne $cleanupMarker -and $cleanupMarker.FRAGCAP_DEFENDER_EXCLUSION_OWNER -eq $installDirectory -and $PSCmdlet.ShouldProcess($installDirectory, 'Remove exact installer-owned Defender state during final cleanup')) {
                try {
                    Remove-MpPreference -ExclusionPath $installDirectory -ErrorAction SilentlyContinue
                    if (-not (@((Get-MpPreference).ExclusionPath) -contains $installDirectory)) { Remove-ItemProperty -LiteralPath $cleanupMarkerPath -Name 'FRAGCAP_DEFENDER_EXCLUSION_OWNER' -Force -ErrorAction SilentlyContinue }
                } catch { Write-ShruggieLog "Installer-owned Defender cleanup failed: $($_.Exception.Message)" -Level Warn -Source Cleanup }
            }
        }
        if ($seededDefender -and $installDirectory -and $PSCmdlet.ShouldProcess($installDirectory, 'Remove certification-seeded Defender exclusion during final cleanup')) { try { Remove-MpPreference -ExclusionPath $installDirectory -ErrorAction SilentlyContinue } catch { Write-ShruggieLog "Defender cleanup failed: $($_.Exception.Message)" -Level Warn -Source Cleanup } }
        if ($scratch -and (Test-Path -LiteralPath $scratch) -and $PSCmdlet.ShouldProcess($scratch, 'Remove package-certification scratch directory')) { Remove-Item -LiteralPath $scratch -Recurse -Force -ErrorAction SilentlyContinue }
    }

#_______________________________________________________________________________
## End of script
