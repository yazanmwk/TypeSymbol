param(
    [string]$Version = "latest",
    [string]$InstallDir = "$env:USERPROFILE\bin"
)

$ErrorActionPreference = "Stop"

$repo = "yazanmwk/TypeSymbol"

function Write-Log {
    param([string]$Message)
    Write-Host "`n[TypeSymbol release installer] $Message"
}

function Resolve-Tag {
    param([string]$RequestedVersion)

    if ($RequestedVersion -eq "latest") {
        return "latest"
    }

    if ($RequestedVersion.StartsWith("v")) {
        return $RequestedVersion
    }

    return "v$RequestedVersion"
}

function Add-ToUserPath {
    param([string]$PathToAdd)

    $current = [System.Environment]::GetEnvironmentVariable("Path", "User")
    if (-not $current) {
        [System.Environment]::SetEnvironmentVariable("Path", $PathToAdd, "User")
        return
    }

    $entries = $current -split ";"
    if ($entries -contains $PathToAdd) {
        return
    }

    [System.Environment]::SetEnvironmentVariable("Path", "$current;$PathToAdd", "User")
}

$tag = Resolve-Tag -RequestedVersion $Version
$assetTag = $tag

if ($tag -eq "latest") {
    Write-Log "Resolving latest release tag..."
    $latestApi = "https://api.github.com/repos/$repo/releases/latest"
    $releaseInfo = Invoke-RestMethod -Uri $latestApi -Headers @{ "User-Agent" = "TypeSymbolInstaller" }
    $assetTag = $releaseInfo.tag_name
}

$asset = "typesymbol-$assetTag-x86_64-pc-windows-msvc.zip"
$downloadUrl = "https://github.com/$repo/releases/download/$assetTag/$asset"

$tempDir = Join-Path $env:TEMP ("typesymbol-install-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $tempDir | Out-Null
$zipPath = Join-Path $tempDir $asset

try {
    Write-Log "Downloading $asset..."
    Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath

    Write-Log "Installing typesymbol.exe to $InstallDir..."
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Expand-Archive -Path $zipPath -DestinationPath $tempDir -Force
    Copy-Item (Join-Path $tempDir "typesymbol.exe") (Join-Path $InstallDir "typesymbol.exe") -Force

    Add-ToUserPath -PathToAdd $InstallDir

    Write-Log "Install complete."
    Write-Log "Restart PowerShell, then run: typesymbol"
}
finally {
    if (Test-Path $tempDir) {
        Remove-Item -Recurse -Force $tempDir
    }
}
