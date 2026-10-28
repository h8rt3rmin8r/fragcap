# SPDX-License-Identifier: Apache-2.0
# Run inside a fresh Windows Sandbox or equivalent clean Windows guest.
param(
    [Parameter(Mandatory=$true)][string]$PortablePath,
    [Parameter(Mandatory=$true)][string]$ExpectedVersion,
    [Parameter(Mandatory=$true)][string]$ReportPath,
    [string]$MsiPath
)

$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class FragcapErrorMode {
    [DllImport("kernel32.dll")] public static extern uint SetErrorMode(uint mode);
}
'@
[void][FragcapErrorMode]::SetErrorMode(0x8001)

function Invoke-Hidden([string]$Exe, [string]$Arguments, [int]$TimeoutMs) {
    $info = New-Object System.Diagnostics.ProcessStartInfo
    $info.FileName = $Exe
    $info.Arguments = $Arguments
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $info.EnvironmentVariables['HTTP_PROXY'] = ''
    $info.EnvironmentVariables['HTTPS_PROXY'] = ''
    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $info
    try {
        [void]$process.Start()
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        $complete = $process.WaitForExit($TimeoutMs)
        if (-not $complete) {
            $process.Kill()
            [void]$process.WaitForExit(5000)
        }
        return [pscustomobject]@{
            complete = $complete
            exit_code = $(if ($complete) { $process.ExitCode } else { $null })
            stdout = $stdout.Result
            stderr = $stderr.Result
        }
    } finally {
        $process.Dispose()
    }
}

function Test-Executable([string]$Exe, [string]$Version) {
    if (-not (Test-Path -LiteralPath $Exe -PathType Leaf)) { throw 'missing executable' }
    $versionResult = Invoke-Hidden $Exe '--version' 30000
    $helpResult = Invoke-Hidden $Exe '--help' 30000
    $doctorResult = Invoke-Hidden $Exe 'doctor --json' 60000
    $driver = @($doctorResult.stdout -split "`r?`n" | Where-Object { $_ } |
        ForEach-Object { try { $_ | ConvertFrom-Json } catch { $null } } |
        Where-Object { $_.section -eq 'Capture driver' -and $_.name -eq 'npcap' })
    $proxy = @($doctorResult.stdout -split "`r?`n" | Where-Object { $_ } |
        ForEach-Object { try { $_ | ConvertFrom-Json } catch { $null } } |
        Where-Object { $_.section -eq 'Deep Capture' -and $_.name -eq 'proxy backend' })
    $result = [ordered]@{
        sha256 = (Get-FileHash -LiteralPath $Exe -Algorithm SHA256).Hash.ToLowerInvariant()
        version = ($versionResult.complete -and $versionResult.exit_code -eq 0 -and $versionResult.stdout.Trim() -eq "fragcap $Version")
        help = ($helpResult.complete -and $helpResult.exit_code -eq 0 -and $helpResult.stdout.Contains('Usage:'))
        doctor_reached = ($doctorResult.complete -and $doctorResult.exit_code -eq 1)
        npcap_missing = ($driver.Count -eq 1 -and $driver[0].status -eq 'fail' -and $driver[0].detail -eq 'npcap is not installed')
        native_proxy_available = ($proxy.Count -eq 1 -and $proxy[0].status -eq 'ok')
    }
    if (-not $result.version -or -not $result.help -or -not $result.doctor_reached -or
        -not $result.npcap_missing -or -not $result.native_proxy_available) {
        throw 'clean executable startup contract failed'
    }
    return $result
}

$report = [ordered]@{
    schema_version = 1
    os_build = [Environment]::OSVersion.Version.Build
    guest_model = (Get-CimInstance Win32_ComputerSystem).Model
    msvc_runtime_present = (Test-Path 'C:\Windows\System32\vcruntime140.dll')
    npcap_present = (Test-Path 'C:\Windows\System32\Npcap\wpcap.dll')
    portable = $null
    installed = $null
    install = $null
    repair = $null
    uninstall = $null
    user_state_preserved = $null
    complete = $false
}
try {
    if ($report.msvc_runtime_present -or $report.npcap_present) {
        throw 'guest has a runtime or capture driver prerequisite installed'
    }
    $report.portable = Test-Executable $PortablePath $ExpectedVersion
    if ($MsiPath) {
        if (-not (Test-Path -LiteralPath $MsiPath -PathType Leaf)) { throw 'missing MSI' }
        $roaming = Join-Path $env:APPDATA 'fragcap'
        $local = Join-Path $env:LOCALAPPDATA 'fragcap'
        [void][System.IO.Directory]::CreateDirectory($roaming)
        [void][System.IO.Directory]::CreateDirectory($local)
        [System.IO.File]::WriteAllText((Join-Path $roaming 's182-preserve.txt'), 'sentinel-roaming')
        [System.IO.File]::WriteAllText((Join-Path $local 's182-preserve.txt'), 'sentinel-local')
        $msiHash = (Get-FileHash -LiteralPath $MsiPath -Algorithm SHA256).Hash.ToLowerInvariant()
        $report.install = (Invoke-Hidden "$env:SystemRoot\System32\msiexec.exe" "/i `"$MsiPath`" /qn /norestart REBOOT=ReallySuppress" 600000).exit_code
        if ($report.install -ne 0) { throw 'clean MSI install failed' }
        $installedPath = Join-Path $env:ProgramFiles 'fragcap\fragcap.exe'
        $report.installed = Test-Executable $installedPath $ExpectedVersion
        if ($report.installed.sha256 -ne $report.portable.sha256) { throw 'installed and portable bytes differ' }
        $report.repair = (Invoke-Hidden "$env:SystemRoot\System32\msiexec.exe" "/fa `"$MsiPath`" /qn /norestart REBOOT=ReallySuppress" 600000).exit_code
        if ($report.repair -ne 0) { throw 'clean MSI repair failed' }
        $report.uninstall = (Invoke-Hidden "$env:SystemRoot\System32\msiexec.exe" "/x `"$MsiPath`" /qn /norestart REBOOT=ReallySuppress" 600000).exit_code
        if ($report.uninstall -ne 0 -or (Test-Path -LiteralPath $installedPath)) { throw 'clean MSI uninstall failed' }
        $report.user_state_preserved =
            ((Get-Content -Raw -LiteralPath (Join-Path $roaming 's182-preserve.txt')) -eq 'sentinel-roaming' -and
             (Get-Content -Raw -LiteralPath (Join-Path $local 's182-preserve.txt')) -eq 'sentinel-local')
        if (-not $report.user_state_preserved) { throw 'synthetic user state changed' }
        $report.msi_sha256 = $msiHash
    }
    $report.complete = $true
} catch {
    $report.failure = $_.Exception.Message
} finally {
    $json = $report | ConvertTo-Json -Depth 8
    [System.IO.File]::WriteAllText($ReportPath, "$json`n", [System.Text.UTF8Encoding]::new($false))
}
if (-not $report.complete) { exit 1 }
