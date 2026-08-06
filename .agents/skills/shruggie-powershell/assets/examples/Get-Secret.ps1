<#
.SYNOPSIS
    Generate a cryptographically secure random secret string for use in
    full-stack development.

.DESCRIPTION
    Generates random bytes from a cryptographically secure source
    (System.Security.Cryptography.RandomNumberGenerator, which pulls from
    the OS CSPRNG) and encodes them in the requested format. Suitable for
    OAuth cookie secrets, Auth.js AUTH_SECRET values, JWT signing keys,
    API tokens, CSRF secrets, and similar credentials.

    Defaults to 32 bytes encoded as standard Base64, which is the format
    expected by most Node.js, Cloudflare Workers, and Auth.js secret
    consumers.

    NOTE: This script does NOT use Get-Random, which is non-cryptographic
    on Windows PowerShell 5.1 and unsuitable for secret generation.

.PARAMETER Length
    Number of random bytes to generate before encoding. The resulting
    string length depends on the encoding (for example, 32 bytes produces
    about 44 Base64 characters, 64 hex characters, or 43 Base64Url
    characters). For the Alphanumeric format, this value is interpreted as
    the character count of the output string.
    Default: 32.
    Alias: l

.PARAMETER Format
    Output encoding format. Valid options:
      - Base64       (default; standard padded base64)
      - Base64Url    (URL-safe; no padding; for JWT and URL parameters)
      - Hex          (lowercase hexadecimal)
      - Alphanumeric (A-Z, a-z, 0-9; for secrets that must avoid symbols)
    Alias: f

.PARAMETER Count
    Number of secrets to generate. Each is printed on its own line.
    Default: 1.
    Alias: n

.PARAMETER Clipboard
    Copy the generated secret(s) to the clipboard instead of printing to
    stdout. Avoids leaving the secret in the terminal scrollback.
    Alias: c

.PARAMETER Quiet
    Emit only the payload with no trailing newline, for clean piping into
    another command. Suppresses human-facing decoration.
    Alias: q

.PARAMETER Help
    Print this help text to the terminal.
    Alias: h

.EXAMPLE
    .\Get-Secret.ps1
    Generates a single 32-byte Base64 secret (the most common case;
    suitable for AUTH_SECRET, cookie secrets, and similar).

.EXAMPLE
    .\Get-Secret.ps1 -Length 64 -Format Hex
    Generates a 64-byte secret encoded as 128 hex characters.

.EXAMPLE
    .\Get-Secret.ps1 -Format Base64Url -Clipboard
    Generates a URL-safe Base64 secret and copies it directly to the
    clipboard without echoing it to the terminal.

.EXAMPLE
    .\Get-Secret.ps1 -Count 5 -Format Alphanumeric -Length 16
    Generates five 16-character alphanumeric secrets, one per line.

.EXAMPLE
    .\Get-Secret.ps1 -Quiet | Set-Clipboard
    Generates a secret with no trailing newline and pipes it to the
    clipboard. Useful when the consumer is sensitive to whitespace.
#>
[CmdletBinding(SupportsShouldProcess=$false,ConfirmImpact='None',DefaultParameterSetName='Default')]
Param(
    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("l")]
    [ValidateRange(1, 4096)]
    [int]$Length = 32,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("f")]
    [ValidateSet('Base64','Base64Url','Hex','Alphanumeric')]
    [string]$Format = 'Base64',

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("n")]
    [ValidateRange(1, 1000)]
    [int]$Count = 1,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("c")]
    [Switch]$Clipboard,

    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("q")]
    [Switch]$Quiet,

    [Parameter(Mandatory=$true,ParameterSetName='HelpText')]
    [Alias("h")]
    [Switch]$Help
)
#_______________________________________________________________________________
## Declare Functions

    function Get-CryptoBytes {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [int]$ByteCount
        )
        $bytes = New-Object byte[] $ByteCount
        $rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
        try {
            $rng.GetBytes($bytes)
        } finally {
            $rng.Dispose()
        }
        return ,$bytes
    }

    function ConvertTo-Base64Url {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [byte[]]$InputBytes
        )
        $b64 = [Convert]::ToBase64String($InputBytes)
        return $b64.TrimEnd('=').Replace('+','-').Replace('/','_')
    }

    function ConvertTo-HexString {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [byte[]]$InputBytes
        )
        return -join ($InputBytes | ForEach-Object { '{0:x2}' -f $_ })
    }

    function ConvertTo-AlphanumericString {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [int]$CharCount
        )
        # 62-char alphabet. Reject bytes >= 248 (4 * 62) to avoid modulo
        # bias. Over-fetch to reduce loop iterations.
        $alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'
        $result = New-Object System.Text.StringBuilder
        while ($result.Length -lt $CharCount) {
            $batch = Get-CryptoBytes -ByteCount ($CharCount * 2)
            foreach ($b in $batch) {
                if ($b -lt 248) {
                    [void]$result.Append($alphabet[$b % 62])
                    if ($result.Length -ge $CharCount) { break }
                }
            }
        }
        return $result.ToString()
    }

    function New-Secret {
        [CmdletBinding()]
        Param(
            [Parameter(Mandatory=$true)]
            [int]$ByteLength,

            [Parameter(Mandatory=$true)]
            [string]$Encoding
        )
        switch ($Encoding) {
            'Base64' {
                $bytes = Get-CryptoBytes -ByteCount $ByteLength
                return [Convert]::ToBase64String($bytes)
            }
            'Base64Url' {
                $bytes = Get-CryptoBytes -ByteCount $ByteLength
                return ConvertTo-Base64Url -InputBytes $bytes
            }
            'Hex' {
                $bytes = Get-CryptoBytes -ByteCount $ByteLength
                return ConvertTo-HexString -InputBytes $bytes
            }
            'Alphanumeric' {
                return ConvertTo-AlphanumericString -CharCount $ByteLength
            }
        }
    }

#_______________________________________________________________________________
## Declare Variables and Arrays

    $ThisScriptPath = $MyInvocation.MyCommand.Path

#_______________________________________________________________________________
## Execute Operations

    # Catch help text requests
    if (($Help) -or ($PSCmdlet.ParameterSetName -eq 'HelpText')) {
        Get-Help $ThisScriptPath -Detailed
        exit 0
    }

    # Generate the requested number of secrets
    $secrets = @()
    for ($i = 0; $i -lt $Count; $i++) {
        $secrets += New-Secret -ByteLength $Length -Encoding $Format
    }

    $output = $secrets -join [Environment]::NewLine

    # Emit per the requested output mode
    if ($Clipboard) {
        $output | Set-Clipboard
        Write-Verbose "Copied $Count secret(s) to clipboard."
    } elseif ($Quiet) {
        [Console]::Out.Write($output)
    } else {
        Write-Output $output
    }

#_______________________________________________________________________________
## End of script
