<#
.SYNOPSIS
    One-line description of what this script does.

.DESCRIPTION
    Multi-paragraph explanation including notable behaviors, side effects, and
    constraints (PowerShell version, network reachability, clipboard or browser
    interaction, anything an operator should know before running it).

    Replace this entire block with real help content. The comment-based help
    block is the interface boundary between the agent that authored the script
    and the operator who runs it, so more help is better than less.

.PARAMETER Example
    Description of the parameter. Document the single-letter alias inline, for
    example "Alias: e", rather than as a separate tag.
    Default: 'value'.
    Alias: e

.PARAMETER Help
    Print this help text to the terminal.
    Alias: h

.EXAMPLE
    .\Verb-Noun.ps1
    The most common invocation.

.EXAMPLE
    .\Verb-Noun.ps1 -Example 'other'
    A second example covering a meaningful flag combination.

.NOTES
    Optional. Caveats and provenance.
#>
[CmdletBinding(SupportsShouldProcess=$false,ConfirmImpact='None',DefaultParameterSetName='Default')]
Param(
    [Parameter(Mandatory=$false,ParameterSetName='Default')]
    [Alias("e")]
    [string]$Example = 'value',

    [Parameter(Mandatory=$true,ParameterSetName='HelpText')]
    [Alias("h")]
    [Switch]$Help
)
#_______________________________________________________________________________
## Declare Functions

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

    # Replace with the script body.

#_______________________________________________________________________________
## End of script
