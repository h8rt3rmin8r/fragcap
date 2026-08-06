<#
.SYNOPSIS
    Maintainer aid: fetch the current upstream spec-kit command templates and
    docs into a scratch directory for diffing against speckit-reference.md.

.DESCRIPTION
    Clones the upstream spec-kit repository shallowly and copies its command
    templates and key docs into the given output directory so a maintainer can
    diff them against assets/speckit-reference.md and refresh it. This is never
    run at skill runtime. It performs a read-only network fetch and does not
    modify the repository.

    This is the PowerShell twin of scripts/update-speckit-reference.sh. The
    upstream layout can change; if the expected paths are absent, the script
    copies whatever it finds and prints a note.

.PARAMETER OutDir
    Scratch directory to receive the fetched command templates and docs.
    Required.

.PARAMETER Repo
    Upstream git URL. Defaults to https://github.com/github/spec-kit.git or the
    SPECKIT_REPO environment variable.

.PARAMETER Ref
    Branch or tag to fetch. Defaults to main or the SPECKIT_REF environment
    variable.

.EXAMPLE
    ./update-speckit-reference.ps1 -OutDir ../scratch/speckit
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$OutDir,

    [string]$Repo = $(if ($env:SPECKIT_REPO) { $env:SPECKIT_REPO } else { 'https://github.com/github/spec-kit.git' }),

    [string]$Ref = $(if ($env:SPECKIT_REF) { $env:SPECKIT_REF } else { 'main' })
)

$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Error 'git is required'
    exit 1
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Force -Path $tmp | Out-Null

try {
    $clone = Join-Path $tmp 'spec-kit'
    Write-Host "Cloning $Repo ($Ref) ..."
    git clone --depth 1 --branch $Ref $Repo $clone 2>$null
    if ($LASTEXITCODE -ne 0) {
        git clone --depth 1 $Repo $clone 2>$null
    }
    if ($LASTEXITCODE -ne 0) {
        Write-Error "clone failed for $Repo"
        exit 1
    }

    $found = $false
    foreach ($path in @('templates/commands', 'docs', 'spec-driven.md', 'README.md')) {
        $src = Join-Path $clone $path
        if (Test-Path $src) {
            $dest = Join-Path $OutDir (Split-Path $path -Leaf)
            Copy-Item -Recurse -Force -Path $src -Destination $dest
            Write-Host "  fetched: $path -> $dest"
            $found = $true
        }
    }

    if (-not $found) {
        Write-Warning "none of the expected upstream paths were found; the upstream layout may have changed. Inspect $clone and refresh the reference by hand."
    }

    Write-Host 'Done. Diff the fetched material against assets/speckit-reference.md.'
}
finally {
    Remove-Item -Recurse -Force -Path $tmp -ErrorAction SilentlyContinue
}
