<#
.SYNOPSIS
    Encode an image as a base64 data URI and emit a Markdown reference-style
    image definition for the bottom of a document.

.DESCRIPTION
    Reads an image file, resolves its MIME type from the file extension, and
    base64-encodes the bytes with no inserted line breaks. Emits a Markdown
    link reference definition of the form

        [label]: data:<mime>;base64,<blob>

    suitable for appending to the very bottom of a document, plus the matching
    in-body usage hint (![alt][label]) to place where the image should appear.

    This is the PowerShell twin of scripts/encode-image-datauri.sh and produces
    byte-identical Markdown so the two are interchangeable. Supported image
    types are png, jpg, jpeg, gif, webp, and svg; any other extension is a
    runtime failure.

    With -OutFile the definition is appended to the named Markdown file
    (preceded by a separating blank line) instead of being written to the
    output stream. Appending mutates a file, so the script declares
    SupportsShouldProcess and gates the write behind $PSCmdlet.ShouldProcess;
    operators get -WhatIf and -Confirm for free. The file is written UTF-8 with
    no BOM and LF line endings.

    Data-URI images do not render on GitHub.com (the data: scheme is blocked in
    image sources). They render in VS Code preview, browsers, and pandoc or
    WeasyPrint output. For an image that must render on GitHub, commit the file
    and reference it with a relative path instead.

    Exit codes: 0 success; 1 runtime failure (file not found, unsupported image
    type); 2 usage error (the required -Path was not supplied).

.PARAMETER Path
    Path to the source image (png, jpg, jpeg, gif, webp, svg). Required; the
    script exits 2 if it is omitted. Read with literal-path semantics so a name
    containing wildcard metacharacters is handled verbatim.
    Alias: p

.PARAMETER Label
    Reference label for the definition and the in-body image. Defaults to a
    value derived from the file name the same way the Bash twin does: strip the
    extension, lowercase, turn spaces into hyphens, drop every remaining
    character that is not a letter, digit, or hyphen, then prefix "img-".
    Alias: l

.PARAMETER Alt
    Alt text for the in-body reference. Defaults to the label.
    Alias: a

.PARAMETER OutFile
    Markdown file to append the definition to, preceded by a blank line. When
    omitted, the definition is written to the output stream instead.
    Alias: o

.PARAMETER Quiet
    Suppress the informational guidance lines and emit only the definition
    payload. Warnings and errors still emit.
    Alias: q

.PARAMETER Silent
    Suppress all log output including warnings. Genuine errors still reach the
    error stream.

.PARAMETER Help
    Print this help text to the terminal.
    Alias: h

.EXAMPLE
    .\Encode-ImageDataUri.ps1 -Path diagram.png
    Prints the in-body usage hint and the reference definition for diagram.png
    to the output stream.

.EXAMPLE
    .\Encode-ImageDataUri.ps1 -Path diagram.png -Alt "Architecture overview" -OutFile spec.md
    Appends the definition for diagram.png to the bottom of spec.md (after a
    separating blank line) and reports the in-body usage hint.

.EXAMPLE
    .\Encode-ImageDataUri.ps1 -Path logo.svg -Label brand-logo -Quiet
    Emits only the reference definition, with no guidance lines, using an
    explicit label.

.NOTES
    Twin of scripts/encode-image-datauri.sh. Targets PowerShell 7+ (invoked as
    pwsh). The exit-code mapping (1 runtime, 2 usage) is specific to this
    script and documented above.
#>
[CmdletBinding(SupportsShouldProcess=$true,ConfirmImpact='Low',DefaultParameterSetName='Default')]
Param(
    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("p")]
    [string]$Path = '',

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("l")]
    [string]$Label = '',

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("a")]
    [string]$Alt = '',

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("o")]
    [string]$OutFile = '',

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

    function Get-DefaultLabel {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$ImagePath
        )
        # Mirror the Bash twin: stem, lowercase, spaces to hyphens, strip every
        # remaining character that is not a-z, 0-9, or hyphen, then prefix.
        $stem = [System.IO.Path]::GetFileNameWithoutExtension($ImagePath)
        $stem = $stem.ToLowerInvariant() -replace ' ', '-'
        $stem = $stem -replace '[^a-z0-9-]', ''
        return "img-$stem"
    }

    function Resolve-ImageMime {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [string]$ImagePath
        )
        $ext = [System.IO.Path]::GetExtension($ImagePath).TrimStart('.').ToLowerInvariant()
        switch ($ext) {
            'png'  { return 'image/png' }
            'jpg'  { return 'image/jpeg' }
            'jpeg' { return 'image/jpeg' }
            'gif'  { return 'image/gif' }
            'webp' { return 'image/webp' }
            'svg'  { return 'image/svg+xml' }
            default { return $null }
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

    # Require PowerShell 7+ for the documented BOM-free, LF append behavior
    Assert-PSVersion -Minimum '7.0'

    # Wire suppression flags to the logging state
    if ($Quiet)  { $script:LogQuiet  = $true }
    if ($Silent) { $script:LogSilent = $true }

    # Usage precondition: the image path is required
    if ([string]::IsNullOrWhiteSpace($Path)) {
        Write-Host "FAIL: -Path <image> is required." -ForegroundColor Red
        exit 2
    }

    # Runtime precondition: the image file must exist
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        Write-Host ("FAIL: file not found: {0}" -f $Path) -ForegroundColor Red
        exit 1
    }

    # Resolve the MIME type from the extension
    $mime = Resolve-ImageMime -ImagePath $Path
    if ($null -eq $mime) {
        $badExt = [System.IO.Path]::GetExtension($Path)
        Write-Host ("FAIL: unsupported image type: {0}" -f $badExt) -ForegroundColor Red
        exit 1
    }

    # Derive defaults for label and alt text
    if ([string]::IsNullOrWhiteSpace($Label)) {
        $Label = Get-DefaultLabel -ImagePath $Path
    }
    if ([string]::IsNullOrWhiteSpace($Alt)) {
        $Alt = $Label
    }

    # Read the bytes and encode with no inserted line breaks
    $imgFull = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Path)
    $bytes = [System.IO.File]::ReadAllBytes($imgFull)
    $b64 = [Convert]::ToBase64String($bytes)
    $definition = "[$Label]: data:$mime;base64,$b64"

    if (-not [string]::IsNullOrWhiteSpace($OutFile)) {
        # Append the definition to the target document, gated for the mutation
        if ($PSCmdlet.ShouldProcess($OutFile, 'Append image data-URI definition')) {
            $outFull = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutFile)
            $enc = New-Object System.Text.UTF8Encoding($false)
            [System.IO.File]::AppendAllText($outFull, "`n$definition`n", $enc)
            Write-Log ("Appended reference '[{0}]' to {1}" -f $Label, $OutFile) -Level Success -Source 'Append'
            Write-Log ("In-body usage: ![{0}][{1}]" -f $Alt, $Label) -Level Info -Source 'Append'
        }
    } else {
        # Write the definition to the output stream; -Quiet emits only the payload
        if (-not $Quiet) {
            Write-Log "In-body usage (place where the image should appear):" -Level Info -Source 'Emit'
            Write-Log ("  ![{0}][{1}]" -f $Alt, $Label) -Level Info -Source 'Emit'
            Write-Log "Reference definition (place at the BOTTOM of the document):" -Level Info -Source 'Emit'
        }
        Write-Output $definition
    }

    exit 0

#_______________________________________________________________________________
## End of script
