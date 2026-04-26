param(
    [Parameter(Mandatory = $true)]
    [string]$Version,
    [Parameter(Mandatory = $true)]
    [string]$ChecksumsFile
)

$ErrorActionPreference = "Stop"

# Override for your fork, e.g. $env:TYPESYMBOL_GITHUB_REPO = "owner/TypeSymbol"
$repo = if ($env:TYPESYMBOL_GITHUB_REPO) { $env:TYPESYMBOL_GITHUB_REPO } else { "yazanmwk/TypeSymbol" }
# WinGet publisher segment (lowercase first letter in path), e.g. $env:WINGET_PUBLISHER = "yazanmwk"
$wingetPublisher = if ($env:WINGET_PUBLISHER) { $env:WINGET_PUBLISHER } else { "yazanmwk" }
$firstLetter = $wingetPublisher.Substring(0, 1).ToLower()
$packageId = "$wingetPublisher.TypeSymbol"

$artifact = "typesymbol-v$Version-x86_64-pc-windows-msvc.msi"
$checksums = Get-Content $ChecksumsFile
$sha = $null
foreach ($line in $checksums) {
    if ($line -match [regex]::Escape($artifact)) {
        $sha = ($line -split "\s+")[0]
    }
}

if (-not $sha) {
    throw "Could not find checksum for $artifact in $ChecksumsFile"
}

$url = "https://github.com/$repo/releases/download/v$Version/$artifact"
$releaseNotesUrl = "https://github.com/$repo/releases/tag/v$Version"

$outputRoot = Join-Path "packaging/winget/manifests" "$firstLetter/$wingetPublisher/TypeSymbol/$Version"
New-Item -ItemType Directory -Path $outputRoot -Force | Out-Null

function Render-Template {
    param(
        [string]$TemplatePath,
        [string]$OutputPath
    )
    $content = Get-Content $TemplatePath -Raw
    $content = $content.Replace("__VERSION__", $Version)
    $content = $content.Replace("__URL_WINDOWS_X64__", $url)
    $content = $content.Replace("__SHA_WINDOWS_X64__", $sha)
    $content = $content.Replace("__RELEASE_NOTES_URL__", $releaseNotesUrl)
    Set-Content -Path $OutputPath -Value $content -NoNewline
}

Render-Template "packaging/winget/typesymbol.yaml.template" (Join-Path $outputRoot "$packageId.yaml")
Render-Template "packaging/winget/typesymbol.installer.yaml.template" (Join-Path $outputRoot "$packageId.installer.yaml")
Render-Template "packaging/winget/typesymbol.locale.en-US.yaml.template" (Join-Path $outputRoot "$packageId.locale.en-US.yaml")

Write-Host "Generated Winget manifests in: $outputRoot"
Write-Host "Next: submit these files to microsoft/winget-pkgs."
