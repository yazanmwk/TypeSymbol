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

function Test-VcRuntimePresent {
    $system32 = Join-Path $env:WINDIR "System32"
    $required = @(
        "vcruntime140.dll",
        "vcruntime140_1.dll",
        "msvcp140.dll"
    )

    foreach ($dll in $required) {
        if (-not (Test-Path (Join-Path $system32 $dll))) {
            return $false
        }
    }

    return $true
}

function Ensure-VcRuntime {
    if (Test-VcRuntimePresent) {
        return
    }

    Write-Log "Microsoft VC++ runtime not found. Installing VC++ Redistributable (x64)..."

    $vcRedistUrl = "https://aka.ms/vs/17/release/vc_redist.x64.exe"
    $vcRedistPath = Join-Path $env:TEMP "vc_redist.x64.exe"
    Invoke-WebRequest -Uri $vcRedistUrl -OutFile $vcRedistPath

    try {
        $proc = Start-Process -FilePath $vcRedistPath -ArgumentList "/install", "/quiet", "/norestart" -PassThru -Wait -Verb RunAs
        if ($proc.ExitCode -ne 0 -and $proc.ExitCode -ne 3010) {
            throw "VC++ Redistributable installer failed with exit code $($proc.ExitCode)."
        }
    }
    finally {
        if (Test-Path $vcRedistPath) {
            Remove-Item -Force $vcRedistPath
        }
    }

    if (-not (Test-VcRuntimePresent)) {
        throw "VC++ runtime is still missing after install. Install manually from https://aka.ms/vs/17/release/vc_redist.x64.exe and rerun."
    }
}

function Ensure-GitHubConnectivity {
    try {
        Resolve-DnsName github.com -ErrorAction Stop | Out-Null
    }
    catch {
        throw "Cannot resolve github.com (DNS failure). Check internet/proxy/DNS settings, then rerun installer."
    }
}

function Stop-RunningTypeSymbol {
    param([string]$InstalledExePath)

    # Best-effort graceful shutdown through CLI first.
    if (Test-Path $InstalledExePath) {
        try {
            & $InstalledExePath off | Out-Null
        }
        catch {
            # Ignore and continue to process-level stop below.
        }
    }

    # Ensure all running instances release the binary lock.
    try {
        $procs = Get-Process -Name "typesymbol" -ErrorAction SilentlyContinue
        if ($procs) {
            Write-Log "Stopping running TypeSymbol processes..."
            $procs | Stop-Process -Force -ErrorAction SilentlyContinue
            Start-Sleep -Milliseconds 400
        }
    }
    catch {
        # Do not fail install if process enumeration fails unexpectedly.
    }
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
    Ensure-VcRuntime
    Ensure-GitHubConnectivity

    Write-Log "Downloading $asset..."
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath
    }
    catch {
        throw "Failed to download release asset from GitHub ($downloadUrl). If DNS/proxy blocks github.com, use a different network or configure proxy, then rerun."
    }

    Write-Log "Installing typesymbol.exe to $InstallDir..."
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Expand-Archive -Path $zipPath -DestinationPath $tempDir -Force
    $installedExe = Join-Path $InstallDir "typesymbol.exe"
    Stop-RunningTypeSymbol -InstalledExePath $installedExe
    Copy-Item (Join-Path $tempDir "typesymbol.exe") $installedExe -Force

    Add-ToUserPath -PathToAdd $InstallDir

    Write-Log "Starting TypeSymbol daemon..."
    & $installedExe on | Out-Host

    Write-Log "Install complete."
    Write-Log "Quick checks:"
    Write-Host "  $installedExe test `"alpha -> beta`""
    Write-Host "  $installedExe daemon status"
    Write-Log "Restart PowerShell, then run: typesymbol test `"alpha -> beta`""
}
finally {
    if (Test-Path $tempDir) {
        Remove-Item -Recurse -Force $tempDir
    }
}
