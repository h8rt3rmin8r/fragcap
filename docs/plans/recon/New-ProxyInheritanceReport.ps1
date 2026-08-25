# SPDX-License-Identifier: Apache-2.0
<#
.SYNOPSIS
Generates a scrubbed proxy-inheritance findings report.

.DESCRIPTION
Reads an alias-only JSON evidence summary and writes the public Markdown report
for the Steam launcher proxy inheritance protocol. The source JSON must already
be scrubbed before this script runs. The script validates the allowed verdict
vocabulary and scans the generated report for common private-data patterns
before writing it.

Raw captures, process logs, socket logs, proxy logs, title names, account
values, executable paths, command lines, hostnames, and addresses must remain in
gitignored capture directories.

.PARAMETER EvidencePath
Path to an alias-only JSON evidence summary.

.PARAMETER OutputPath
Path for the Markdown report. When omitted, the report is written under
docs/plans/recon/proxy-inheritance-<date>.md.

.PARAMETER Force
Overwrite an existing output file.

.PARAMETER PrivateTermsPath
Optional path to a local newline-delimited private term blocklist. Keep this
file under captures/recon/ or another ignored private directory.

.PARAMETER Quiet
Suppress informational output.

.PARAMETER Silent
Suppress all non-error output.

.PARAMETER Help
Show this help text.

.EXAMPLE
pwsh -File docs/plans/recon/New-ProxyInheritanceReport.ps1 `
  -EvidencePath captures/recon/proxy-summary.json

.EXAMPLE
pwsh -File docs/plans/recon/New-ProxyInheritanceReport.ps1 `
  -EvidencePath captures/recon/proxy-summary.json `
  -OutputPath docs/plans/recon/proxy-inheritance-2026-08-24.md
#>
[CmdletBinding(
    SupportsShouldProcess = $true,
    ConfirmImpact = 'Low',
    DefaultParameterSetName = 'Default'
)]
param(
    [Parameter(Mandatory = $true, ParameterSetName = 'Default')]
    [Alias('p')]
    [string]$EvidencePath,

    [Parameter(Mandatory = $false, ParameterSetName = 'Default')]
    [Alias('o')]
    [string]$OutputPath,

    [Parameter(Mandatory = $false, ParameterSetName = 'Default')]
    [Alias('f')]
    [switch]$Force,

    [Parameter(Mandatory = $false, ParameterSetName = 'Default')]
    [Alias('t')]
    [string]$PrivateTermsPath,

    [Parameter(Mandatory = $false, ParameterSetName = 'Default')]
    [Alias('q')]
    [switch]$Quiet,

    [Parameter(Mandatory = $false, ParameterSetName = 'Default')]
    [switch]$Silent,

    [Parameter(Mandatory = $true, ParameterSetName = 'Help')]
    [Alias('h')]
    [switch]$Help
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

#_______________________________________________________________________________
## Declare Functions

function Write-ShruggieLog {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [ValidateSet('Info', 'Warn', 'Error')]
        [string]$Level,

        [Parameter(Mandatory = $true)]
        [string]$Message
    )

    $line = "[$Level] $Message"
    if ($Level -eq 'Error') {
        Write-Error $line
        return
    }

    if ($Silent) {
        return
    }

    if ($Quiet -and $Level -eq 'Info') {
        return
    }

    Write-Host $line
}

function Assert-PSVersion {
    [CmdletBinding()]
    param()

    if ($PSVersionTable.PSVersion.Major -lt 7) {
        throw 'PowerShell 7 or newer is required.'
    }
}

function Get-RepositoryRoot {
    [CmdletBinding()]
    param()

    $scriptDir = Split-Path -Parent $ThisScriptPath
    $plansDir = Split-Path -Parent $scriptDir
    $docsDir = Split-Path -Parent $plansDir
    return Split-Path -Parent $docsDir
}

function Assert-ValueInSet {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [AllowEmptyString()]
        [string]$Value,

        [Parameter(Mandatory = $true)]
        [string[]]$Allowed,

        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    if ($Allowed -notcontains $Value) {
        $allowedText = $Allowed -join ', '
        throw "$Name must be one of: $allowedText"
    }
}

function Assert-Alias {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [AllowEmptyString()]
        [string]$Value,

        [Parameter(Mandatory = $true)]
        [string]$Pattern,

        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    if ($Value -notmatch $Pattern) {
        throw "$Name is not a valid public alias: $Value"
    }
}

function Get-RequiredString {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [pscustomobject]$Object,

        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    if (-not ($Object.PSObject.Properties.Name -contains $Name)) {
        throw "Missing required property: $Name"
    }

    $value = [string]$Object.$Name
    if ([string]::IsNullOrWhiteSpace($value)) {
        throw "Property must not be empty: $Name"
    }

    return $value
}

function Test-UnsafeText {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$Text
    )

    $findings = New-Object System.Collections.Generic.List[string]
    foreach ($item in $Script:UnsafePatterns) {
        if ($Text -match $item.Pattern) {
            $findings.Add([string]$item.Name)
        }
    }

    return $findings.ToArray()
}

function Add-PrivateTerms {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $false)]
        [AllowNull()]
        [AllowEmptyString()]
        [string]$Path
    )

    if ([string]::IsNullOrWhiteSpace($Path)) {
        return
    }

    $resolvedPath = Resolve-Path -LiteralPath $Path
    foreach ($line in (Get-Content -LiteralPath $resolvedPath)) {
        $term = [string]$line
        if ([string]::IsNullOrWhiteSpace($term)) {
            continue
        }

        $Script:UnsafePatterns += @{
            Name = 'operator private term'
            Pattern = "(?i)$([regex]::Escape($term))"
        }
    }
}

function ConvertTo-MarkdownCell {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [AllowEmptyString()]
        [string]$Value
    )

    return ($Value -replace '\|', '\|' -replace "`r?`n", ' ')
}

function ConvertTo-BulletList {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [AllowNull()]
        $Items
    )

    $lines = New-Object System.Collections.Generic.List[string]
    foreach ($item in @($Items)) {
        if ($null -eq $item) {
            continue
        }

        $text = [string]$item
        if (-not [string]::IsNullOrWhiteSpace($text)) {
            $lines.Add("- $text")
        }
    }

    if ($lines.Count -eq 0) {
        $lines.Add("- None recorded.")
    }

    return $lines.ToArray()
}

function Assert-NonBlankList {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [AllowNull()]
        $Items,

        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    $count = 0
    foreach ($item in @($Items)) {
        if ($null -eq $item) {
            continue
        }

        if (-not [string]::IsNullOrWhiteSpace([string]$item)) {
            $count += 1
        }
    }

    if ($count -eq 0) {
        throw "$Name must contain at least one nonblank entry."
    }
}

function Assert-Run {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [pscustomobject]$Run
    )

    Assert-Alias `
        -Value (Get-RequiredString -Object $Run -Name 'session_alias') `
        -Pattern '^run-[0-9]{3}$' `
        -Name 'session_alias'
    Assert-Alias `
        -Value (Get-RequiredString -Object $Run -Name 'title_alias') `
        -Pattern '^title-[a-z]+$' `
        -Name 'title_alias'
    Assert-Alias `
        -Value (Get-RequiredString -Object $Run -Name 'platform_alias') `
        -Pattern '^steam-app-[a-z]+$' `
        -Name 'platform_alias'
    Assert-Alias `
        -Value (Get-RequiredString -Object $Run -Name 'invoked_alias') `
        -Pattern $Script:ProcessAliasPattern `
        -Name 'invoked_alias'
    Assert-Alias `
        -Value (Get-RequiredString -Object $Run -Name 'final_socket_owner_alias') `
        -Pattern $Script:ProcessAliasPattern `
        -Name 'final_socket_owner_alias'
    Assert-Alias `
        -Value (Get-RequiredString -Object $Run -Name 'observed_ancestry') `
        -Pattern $Script:AncestryPattern `
        -Name 'observed_ancestry'
    Assert-Alias `
        -Value (Get-RequiredString -Object $Run -Name 'proxy_listener') `
        -Pattern '^loopback:port-[a-z]+$' `
        -Name 'proxy_listener'

    Assert-ValueInSet `
        -Value (Get-RequiredString -Object $Run -Name 'launch_case') `
        -Allowed $Script:LaunchCases `
        -Name 'launch_case'
    Assert-ValueInSet `
        -Value (Get-RequiredString -Object $Run -Name 'steam_pre_state') `
        -Allowed $Script:SteamStates `
        -Name 'steam_pre_state'
    Assert-ValueInSet `
        -Value (Get-RequiredString -Object $Run -Name 'proxy_traffic_observed') `
        -Allowed $Script:ProxyTrafficValues `
        -Name 'proxy_traffic_observed'
    Assert-ValueInSet `
        -Value (Get-RequiredString -Object $Run -Name 'relevant_sockets_observed') `
        -Allowed $Script:ObservedValues `
        -Name 'relevant_sockets_observed'
    Assert-ValueInSet `
        -Value (Get-RequiredString -Object $Run -Name 'routing_verdict') `
        -Allowed $Script:RoutingVerdicts `
        -Name 'routing_verdict'
    Assert-ValueInSet `
        -Value (Get-RequiredString -Object $Run -Name 'propagation_finding') `
        -Allowed $Script:PropagationFindings `
        -Name 'propagation_finding'
    Assert-ValueInSet `
        -Value (Get-RequiredString -Object $Run -Name 'confidence') `
        -Allowed $Script:ConfidenceValues `
        -Name 'confidence'
    Assert-ValueInSet `
        -Value (Get-RequiredString -Object $Run -Name 'product_consequence') `
        -Allowed $Script:Consequences `
        -Name 'product_consequence'

    if (-not ($Run.PSObject.Properties.Name -contains 'evidence')) {
        throw 'Missing required property: evidence'
    }

    Assert-NonBlankList -Items $Run.evidence -Name 'evidence'
}

function Assert-Evidence {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [pscustomobject]$Evidence
    )

    $null = Get-RequiredString -Object $Evidence -Name 'date'
    $null = Get-RequiredString -Object $Evidence -Name 'scope'

    if (-not ($Evidence.PSObject.Properties.Name -contains 'runs')) {
        throw 'Missing required property: runs'
    }

    $runs = @($Evidence.runs)
    if ($runs.Count -eq 0) {
        throw 'Evidence must contain at least one run.'
    }

    foreach ($run in $runs) {
        Assert-Run -Run $run
    }
}

function New-ReportMarkdown {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [pscustomobject]$Evidence
    )

    $date = Get-RequiredString -Object $Evidence -Name 'date'
    $lines = New-Object System.Collections.Generic.List[string]
    $lines.Add("# Steam proxy inheritance findings, $date")
    $lines.Add('')
    $lines.Add('**Status:** scrubbed derivative findings for issue #215.\')
    $lines.Add("**Date:** $date.\")
    $lines.Add('**Audience:** maintainers and Deep Capture slice authors.')
    $lines.Add('')
    $lines.Add('## Scope')
    $lines.Add('')
    $lines.Add((Get-RequiredString -Object $Evidence -Name 'scope'))
    $lines.Add('')
    $lines.Add('## Verdicts')
    $lines.Add('')
    $lines.Add('| Title alias | Launch case | Routing verdict | Propagation finding | Confidence | Product consequence |')
    $lines.Add('| --- | --- | --- | --- | --- | --- |')

    foreach ($run in @($Evidence.runs)) {
        $row = @(
            (ConvertTo-MarkdownCell ([string]$run.title_alias)),
            (ConvertTo-MarkdownCell ([string]$run.launch_case)),
            (ConvertTo-MarkdownCell ([string]$run.routing_verdict)),
            (ConvertTo-MarkdownCell ([string]$run.propagation_finding)),
            (ConvertTo-MarkdownCell ([string]$run.confidence)),
            (ConvertTo-MarkdownCell ([string]$run.product_consequence))
        ) -join ' | '
        $lines.Add("| $row |")
    }

    $lines.Add('')
    $lines.Add('## Findings')

    foreach ($run in @($Evidence.runs)) {
        $lines.Add('')
        $lines.Add("### $($run.title_alias), $($run.launch_case)")
        $lines.Add('')
        $lines.Add('| Field | Value |')
        $lines.Add('| --- | --- |')

        $fields = [ordered]@{
            'Session alias' = $run.session_alias
            'Platform alias' = $run.platform_alias
            'Steam pre-state' = $run.steam_pre_state
            'Invoked alias' = $run.invoked_alias
            'Final socket owner alias' = $run.final_socket_owner_alias
            'Observed ancestry' = $run.observed_ancestry
            'Proxy listener' = $run.proxy_listener
            'Proxy traffic observed' = $run.proxy_traffic_observed
            'Relevant sockets observed' = $run.relevant_sockets_observed
        }

        foreach ($field in $fields.GetEnumerator()) {
            $value = ConvertTo-MarkdownCell ([string]$field.Value)
            $lines.Add("| $($field.Key) | $value |")
        }

        $lines.Add('')
        $lines.Add('Evidence:')
        foreach ($line in (ConvertTo-BulletList -Items $run.evidence)) {
            $lines.Add($line)
        }
    }

    $lines.Add('')
    $lines.Add('## Compatibility facts proposed')
    $lines.Add('')

    if ($Evidence.PSObject.Properties.Name -contains 'compatibility_facts') {
        foreach ($line in (ConvertTo-BulletList -Items $Evidence.compatibility_facts)) {
            $lines.Add($line)
        }
    } else {
        $lines.Add('- None recorded.')
    }

    $lines.Add('')
    $lines.Add('## Open questions')
    $lines.Add('')

    if ($Evidence.PSObject.Properties.Name -contains 'open_questions') {
        foreach ($line in (ConvertTo-BulletList -Items $Evidence.open_questions)) {
            $lines.Add($line)
        }
    } else {
        $lines.Add('- None recorded.')
    }

    $lines.Add('')
    $lines.Add('## Sanitization record')
    $lines.Add('')
    $lines.Add('- Source evidence was reduced to public aliases before writing.')
    $lines.Add('- The generated report was scanned for common private-data patterns.')
    $lines.Add('- Raw captures, logs, command lines, endpoints, and title names remain uncommitted.')

    return ($lines -join "`n") + "`n"
}

function Resolve-DefaultOutputPath {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [pscustomobject]$Evidence
    )

    $date = Get-RequiredString -Object $Evidence -Name 'date'
    $repoRoot = Get-RepositoryRoot
    return Join-Path $repoRoot "docs/plans/recon/proxy-inheritance-$date.md"
}

#_______________________________________________________________________________
## Declare Variables and Arrays

$ThisScriptPath = $MyInvocation.MyCommand.Path

$Script:RoutingVerdicts = @(
    'reached-client',
    'launcher-only-routing',
    'escaped-tree',
    'no-proxy-traffic',
    'not-applicable',
    'inconclusive'
)
$Script:PropagationFindings = @(
    'confirmed',
    'not-confirmed',
    'not-tested'
)
$Script:LaunchCases = @(
    'steam-protocol-warm',
    'steam-protocol-cold',
    'direct-exe-warm',
    'direct-exe-cold',
    'publisher-launcher',
    'publisher-launcher-warm',
    'publisher-launcher-game-start-clean-warm',
    'publisher-launcher-cold',
    'final-owner-differs'
)
$Script:SteamStates = @('running', 'not-running', 'unknown')
$Script:ProxyTrafficValues = @('yes', 'no', 'partial')
$Script:ObservedValues = @('yes', 'no', 'unknown')
$Script:ConfidenceValues = @('observed', 'inferred', 'inconclusive')
$Script:Consequences = @(
    'supported',
    'unsupported',
    'needs fallback',
    'needs more data'
)
$Script:ProcessAliasPattern = '^(unknown|shell|platform-protocol|(?:client|launcher|platform|platform-service|helper|proxy|wrapper)-[a-z]+)$'
$Script:AncestryNodePattern = '(?:unknown|shell|platform-protocol|(?:client|launcher|platform|platform-service|helper|proxy|wrapper)-[a-z]+)'
$Script:AncestryPattern = "^$Script:AncestryNodePattern(?: -> $Script:AncestryNodePattern)*$"

$Script:UnsafePatterns = @(
    @{
        Name = 'IPv4 address'
        Pattern = '\b(?:\d{1,3}\.){3}\d{1,3}\b'
    },
    @{
        Name = 'IPv6 address'
        Pattern = '(?i)\b(?:[0-9a-f]{1,4}:){2,}[0-9a-f:]{0,}(?:%[0-9a-z_.-]+)?\b'
    },
    @{
        Name = 'URL'
        Pattern = 'https?://'
    },
    @{
        Name = 'Windows absolute path'
        Pattern = '\b[A-Za-z]:\\'
    },
    @{
        Name = 'profile path marker'
        Pattern = '\\Users\\|/Users/'
    },
    @{
        Name = 'numeric Steam app id'
        Pattern = '\b[1-9][0-9]{4,}\b'
    },
    @{
        Name = 'secret-bearing argument marker'
        Pattern = '(?i)(bearer\s+|cookie[:=]|token[:=]|ticket[:=]|session[_-]?id[:=]|onetime[_-]?token)'
    }
)

#_______________________________________________________________________________
## Execute Operations

try {
    if ($Help) {
        Get-Help $ThisScriptPath -Detailed
        exit 0
    }

    Assert-PSVersion
    Add-PrivateTerms -Path $PrivateTermsPath

    $resolvedEvidencePath = Resolve-Path -LiteralPath $EvidencePath
    $evidenceJson = Get-Content -LiteralPath $resolvedEvidencePath -Raw
    $evidence = $evidenceJson | ConvertFrom-Json -Depth 16
    Assert-Evidence -Evidence $evidence

    $report = New-ReportMarkdown -Evidence $evidence
    $unsafeFindings = @(Test-UnsafeText -Text $report)
    if ($unsafeFindings.Count -gt 0) {
        $joined = ($unsafeFindings | Sort-Object -Unique) -join ', '
        throw "Generated report failed private-data scan: $joined"
    }

    if ([string]::IsNullOrWhiteSpace($OutputPath)) {
        $OutputPath = Resolve-DefaultOutputPath -Evidence $evidence
    }

    $outputFullPath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath(
        $OutputPath
    )

    if ((Test-Path -LiteralPath $outputFullPath) -and -not $Force) {
        throw "Output already exists. Pass -Force to overwrite: $outputFullPath"
    }

    if ($PSCmdlet.ShouldProcess($outputFullPath, 'write scrubbed report')) {
        $parent = Split-Path -Parent $outputFullPath
        if (-not (Test-Path -LiteralPath $parent)) {
            New-Item -ItemType Directory -Path $parent | Out-Null
        }

        Set-Content -LiteralPath $outputFullPath -Value $report -Encoding utf8
        Write-ShruggieLog -Level Info -Message "Wrote $outputFullPath"
    }

    exit 0
} catch {
    Write-ShruggieLog -Level Error -Message $_.Exception.Message
    exit 1
}

#_______________________________________________________________________________
## End of script
