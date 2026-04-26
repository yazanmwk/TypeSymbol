param(
    [Parameter(Mandatory = $true)]
    [string]$Version,
    [Parameter(Mandatory = $true)]
    [string]$BinaryPath,
    [Parameter(Mandatory = $true)]
    [string]$OutputPath
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $BinaryPath)) {
    throw "Binary not found at $BinaryPath"
}

$wixSource = Join-Path $PSScriptRoot "..\packaging\windows\typesymbol.wxs"
$wixSource = (Resolve-Path $wixSource).Path
if (-not (Test-Path $wixSource)) {
    throw "WiX source file not found at $wixSource"
}

$binaryAbsolute = (Resolve-Path $BinaryPath).Path

$wixBinPath = Join-Path $env:USERPROFILE ".dotnet\tools"
if ($env:PATH -notlike "*$wixBinPath*") {
    $env:PATH = "$env:PATH;$wixBinPath"
}

wix build $wixSource `
    -d Version=$Version `
    -d BinaryPath=$binaryAbsolute `
    -o $OutputPath `
    -arch x64

Write-Host "Built MSI installer: $OutputPath"
