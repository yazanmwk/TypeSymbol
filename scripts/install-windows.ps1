param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

function Write-Log {
    param([string]$Message)
    Write-Host "`n[TypeSymbol installer] $Message"
}

function Test-Command {
    param([string]$Name)
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

function Ensure-Winget {
    if (-not (Test-Command winget)) {
        throw "winget is required. Install App Installer from Microsoft Store, then rerun."
    }
}

function Ensure-Git {
    if (Test-Command git) {
        Write-Log "Git already installed: $(git --version)"
        return
    }

    Write-Log "Installing Git..."
    winget install --id Git.Git --exact --source winget --accept-package-agreements --accept-source-agreements
}

function Ensure-Rustup {
    if (Test-Command rustup) {
        Write-Log "rustup already installed: $((rustup --version)[0])"
        return
    }

    Write-Log "Installing rustup..."
    winget install --id Rustlang.Rustup --exact --source winget --accept-package-agreements --accept-source-agreements
}

function Ensure-BuildTools {
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vswhere) {
        $msvc = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
        if ($msvc) {
            Write-Log "MSVC build tools already installed."
            return
        }
    }

    Write-Log "Installing Visual Studio Build Tools (C++ workload)..."
    winget install --id Microsoft.VisualStudio.2022.BuildTools --exact --source winget --accept-package-agreements --accept-source-agreements --override "--quiet --wait --norestart --nocache --add Microsoft.VisualStudio.Workload.VCTools"
}

function Refresh-Path {
    $machinePath = [System.Environment]::GetEnvironmentVariable("Path", "Machine")
    $userPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
    $env:Path = "$machinePath;$userPath"
}

function Ensure-RustToolchain {
    Refresh-Path
    if (-not (Test-Command rustup)) {
        throw "rustup not found on PATH after install. Open a new terminal and rerun."
    }

    rustup toolchain install stable | Out-Null
    rustup default stable | Out-Null
}

function Install-TypeSymbolBinary {
    if ($SkipBuild) {
        Write-Log "Skipping binary build by request."
        return
    }

    Refresh-Path
    if (-not (Test-Command cargo)) {
        throw "cargo not found on PATH. Open a new terminal and rerun installer."
    }

    $repoRoot = Split-Path -Parent $PSScriptRoot
    Write-Log "Building TypeSymbol from: $repoRoot"
    Push-Location $repoRoot
    try {
        cargo build --release
    }
    finally {
        Pop-Location
    }

    $target = Join-Path $repoRoot "target\release\typesymbol.exe"
    $binDir = Join-Path $env:USERPROFILE "bin"
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null
    Copy-Item $target (Join-Path $binDir "typesymbol.exe") -Force

    $userPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$binDir*") {
        [System.Environment]::SetEnvironmentVariable("Path", "$userPath;$binDir", "User")
        Write-Log "Added $binDir to your user PATH."
    }

    Write-Log "Installed binary to $binDir\typesymbol.exe"
    Write-Log "Open a new PowerShell window, then run: typesymbol"
}

Write-Log "Starting Windows dependency bootstrap..."
Ensure-Winget
Ensure-Git
Ensure-BuildTools
Ensure-Rustup
Ensure-RustToolchain
Install-TypeSymbolBinary
Write-Log "Done."
