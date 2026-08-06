# PowerShell Fixtures

Copy-paste-exact fixtures for ShruggieTech PowerShell scripts. Paste each one
verbatim; adjust only where a note says you may. These are reproduced from the
authoritative reference in `powershell-conventions.md` so that the shapes never
drift. All three sit inside the four-space-indented body of their section.

## Version Guard (Assert-PSVersion)

Include this only when the script uses version-sensitive operators or cmdlets
that misbehave on older engines (for example `Invoke-WebRequest
-SkipHttpErrorCheck`, which exists only on 7+). A script that uses nothing
version-sensitive does not need it. Do not use `#Requires -Version`.

Declare it in `## Declare Functions` and call it as the first operation in
`## Execute Operations`. It exits with the environment-precondition code (2).

```powershell
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
```

## Logging Helper (Write-Log)

For scripts that report operator-facing progress. The helper colorizes by level,
timestamps every line, and tags the emitting sub-process. Adjust levels and
colors only with reason. Declare the suppression state in `## Declare Variables
and Arrays` and the helper in `## Declare Functions`.

```powershell
#_______________________________________________________________________________
## Declare Variables and Arrays

    $script:LogQuiet  = $false
    $script:LogSilent = $false

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
```

Wire the suppression flags to the helper state in `## Execute Operations`,
before any logging:

```powershell
    if ($Quiet)  { $script:LogQuiet  = $true }
    if ($Silent) { $script:LogSilent = $true }
```

## Destructive Gate (ShouldProcess)

When the top-level `[CmdletBinding(...)]` sets `SupportsShouldProcess=$true`,
every destructive operation MUST be wrapped in a `$PSCmdlet.ShouldProcess(...)`
check. The declaration alone does nothing; an unguarded change runs anyway under
`-WhatIf`, which is worse than not supporting it. The first argument is the
target the action affects; the second is a short description of the action.

```powershell
    foreach ($file in $targets) {
        if ($PSCmdlet.ShouldProcess($file.FullName, 'Remove file')) {
            Remove-Item -LiteralPath $file.FullName -Force
        }
    }
```

Do not declare `-WhatIf` or `-Confirm` in the `Param(...)` block; they are
supplied automatically once `SupportsShouldProcess=$true` is set.
