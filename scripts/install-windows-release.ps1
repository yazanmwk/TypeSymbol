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

function Add-ToCurrentSessionPath {
    param([string]$PathToAdd)

    $segments = $env:Path -split ";"
    if ($segments -contains $PathToAdd) {
        return
    }

    $env:Path = "$env:Path;$PathToAdd"
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
            & $InstalledExePath off *> $null
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

function Configure-WindowsAutostart {
    param([string]$InstalledExePath)

    $taskName = "TypeSymbolDaemon"
    $runCommand = "`"$InstalledExePath`" daemon run-internal"

    $schtasksStderr = Join-Path $env:TEMP ("typesymbol-schtasks-" + [Guid]::NewGuid().ToString("N") + ".log")
    try {
        $taskProc = Start-Process -FilePath "schtasks.exe" -ArgumentList @("/Create", "/TN", $taskName, "/SC", "ONLOGON", "/TR", $runCommand, "/F") -PassThru -Wait -WindowStyle Hidden -RedirectStandardError $schtasksStderr
        if ($taskProc.ExitCode -eq 0) {
            Write-Log "Autostart configured via Task Scheduler."
            return
        }
        Write-Warning "Task Scheduler setup failed (exit $($taskProc.ExitCode)). Falling back to HKCU Run key."
    }
    catch {
        Write-Warning "Task Scheduler setup failed ($($_.Exception.Message)). Falling back to HKCU Run key."
    }
    finally {
        if (Test-Path $schtasksStderr) {
            Remove-Item -Force $schtasksStderr -ErrorAction SilentlyContinue
        }
    }

    $regPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
    try {
        New-Item -Path $regPath -Force | Out-Null
        New-ItemProperty -Path $regPath -Name $taskName -PropertyType String -Value $runCommand -Force | Out-Null
        Write-Log "Autostart configured via HKCU Run key."
    }
    catch {
        Write-Warning "Autostart setup failed (Task Scheduler + HKCU Run key). You can still start manually with: `"$InstalledExePath`" on"
    }
}

function Start-TypeSymbolDaemon {
    param([string]$InstalledExePath)

    $stateDir = Join-Path $env:LOCALAPPDATA "TypeSymbol\state"
    New-Item -ItemType Directory -Force -Path $stateDir | Out-Null
    $logPath = Join-Path $stateDir "daemon.log"

    $proc = Start-Process -FilePath $InstalledExePath -ArgumentList @("daemon", "run-internal") -WindowStyle Hidden -PassThru
    Write-Host "Started TypeSymbol daemon in background (pid $($proc.Id))."
    Write-Host "Logs: $logPath"
}

function Resolve-ExtractedExePath {
    param([string]$ExtractionRoot)

    $direct = Join-Path $ExtractionRoot "typesymbol.exe"
    if (Test-Path $direct) {
        return $direct
    }

    $candidate = Get-ChildItem -Path $ExtractionRoot -Filter "typesymbol.exe" -File -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($candidate) {
        return $candidate.FullName
    }

    throw "typesymbol.exe was not found in extracted archive at $ExtractionRoot"
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
    $extractedExe = Resolve-ExtractedExePath -ExtractionRoot $tempDir
    $installedExe = Join-Path $InstallDir "typesymbol.exe"
    Stop-RunningTypeSymbol -InstalledExePath $installedExe
    Copy-Item $extractedExe $installedExe -Force

    Add-ToUserPath -PathToAdd $InstallDir
    Add-ToCurrentSessionPath -PathToAdd $InstallDir

    Write-Log "Configuring TypeSymbol autostart..."
    Configure-WindowsAutostart -InstalledExePath $installedExe

    Write-Log "Starting TypeSymbol daemon..."
    Start-TypeSymbolDaemon -InstalledExePath $installedExe

    Write-Log "Verifying CLI..."
    $smoke = & $installedExe test "alpha -> beta"
    if ($smoke -ne "α → β") {
        throw "CLI verification failed. Expected 'α → β', got '$smoke'"
    }
    Write-Host "CLI smoke test: ok (alpha -> beta => $smoke)"
    Write-Host "CLI shell: run 'typesymbol' (no args) to open the interactive interface."

    Write-Log "Install complete."
    Write-Log "Quick checks:"
    Write-Host "  $installedExe test `"alpha -> beta`""
    Write-Host "  $installedExe daemon status"
    Write-Log "Run now: typesymbol"
}
finally {
    if (Test-Path $tempDir) {
        Remove-Item -Recurse -Force $tempDir
    }
}
