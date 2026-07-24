param(
    [ValidateSet("x86_64-pc-windows-msvc")]
    [string] $Target = "x86_64-pc-windows-msvc",

    [ValidateSet("release", "debug")]
    [string] $Profile = "release",

    [string] $ProductVersion,

    [switch] $SkipInstaller
)

$ErrorActionPreference = "Stop"

$env:Path = "C:\Users\nakorncode\.cargo\bin;$env:USERPROFILE\.cargo\bin;" + $env:Path

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptDir
$cargoToml = Join-Path $repoRoot "Cargo.toml"
$releaseDir = Join-Path $repoRoot "artifacts\release"
$publishDir = Join-Path $repoRoot "artifacts\publish\win-x64"
$installerDir = Join-Path $repoRoot "artifacts\installer"
$zipPath = Join-Path $releaseDir "CapsLang-Portable-win-x64.zip"
$setupExePath = Join-Path $releaseDir "CapsLang-Setup-win-x64.exe"
$checksumPath = Join-Path $releaseDir "CapsLang-SHA256SUMS.txt"
$innoScriptPath = Join-Path $repoRoot "installer\CapsLang.iss"

function Get-ProductVersion {
    if (-not [string]::IsNullOrWhiteSpace($ProductVersion)) {
        return $ProductVersion.TrimStart("v")
    }

    if ($env:GITHUB_REF_TYPE -eq "tag" -and -not [string]::IsNullOrWhiteSpace($env:GITHUB_REF_NAME)) {
        return $env:GITHUB_REF_NAME.TrimStart("v")
    }

    $gitTag = git -C $repoRoot describe --tags --exact-match 2>$null
    if (-not [string]::IsNullOrWhiteSpace($gitTag)) {
        return $gitTag.TrimStart("v")
    }

    $match = Select-String -Path $cargoToml -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
    if ($match) {
        return $match.Matches[0].Groups[1].Value
    }

    return "0.0.0"
}

function Get-IsccPath {
    $command = Get-Command "ISCC.exe" -ErrorAction SilentlyContinue
    if ($command) {
        return $command.Source
    }

    foreach ($path in @(
            "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
            "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
        )) {
        if (Test-Path $path) {
            return $path
        }
    }

    return $null
}

$resolvedVersion = Get-ProductVersion

$runningApps = Get-Process -Name "CapsLang" -ErrorAction SilentlyContinue
if ($runningApps) {
    Write-Host "Stopping running CapsLang process..."
    $runningApps | Stop-Process -Force
}

New-Item -ItemType Directory -Force -Path $publishDir, $releaseDir, $installerDir | Out-Null

Get-ChildItem -Path $releaseDir -File -Filter "CapsLang-*" -ErrorAction SilentlyContinue | Remove-Item -Force
Get-ChildItem -Path $installerDir -File -Filter "CapsLang-*" -ErrorAction SilentlyContinue | Remove-Item -Force
Get-ChildItem -Path $publishDir -Force -ErrorAction SilentlyContinue | Remove-Item -Recurse -Force

Push-Location $repoRoot
try {
    if ($Profile -eq "release") {
        cargo build --release --target $Target
        $builtExe = Join-Path $repoRoot "target\$Target\release\CapsLang.exe"
    }
    else {
        cargo build --target $Target
        $builtExe = Join-Path $repoRoot "target\$Target\debug\CapsLang.exe"
    }
}
finally {
    Pop-Location
}

if (-not (Test-Path $builtExe)) {
    # Fallback for host-triple output layout
    $fallback = if ($Profile -eq "release") {
        Join-Path $repoRoot "target\release\CapsLang.exe"
    } else {
        Join-Path $repoRoot "target\debug\CapsLang.exe"
    }
    if (Test-Path $fallback) {
        $builtExe = $fallback
    } else {
        throw "Built CapsLang.exe was not found."
    }
}

Copy-Item -LiteralPath $builtExe -Destination (Join-Path $publishDir "CapsLang.exe") -Force
Copy-Item -LiteralPath (Join-Path $repoRoot "LICENSE") -Destination (Join-Path $publishDir "LICENSE") -Force
Copy-Item -LiteralPath (Join-Path $repoRoot "README.md") -Destination (Join-Path $publishDir "README.md") -Force

if (Test-Path $zipPath) {
    Remove-Item -LiteralPath $zipPath -Force
}

Compress-Archive -Path (Join-Path $publishDir "*") -DestinationPath $zipPath

if (-not $SkipInstaller) {
    $isccPath = Get-IsccPath
    if (-not $isccPath) {
        throw "Inno Setup compiler was not found. Install Inno Setup 6 or run with -SkipInstaller."
    }

    & $isccPath "/DMyAppVersion=$resolvedVersion" $innoScriptPath

    $builtSetupExe = Get-ChildItem -Path $installerDir -File -Filter "CapsLang-Setup-*.exe" |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1

    if (-not $builtSetupExe) {
        throw "Inno Setup did not create a setup executable."
    }

    Move-Item -LiteralPath $builtSetupExe.FullName -Destination $setupExePath -Force
}

$packages = Get-ChildItem -Path $releaseDir -File | Where-Object { $_.Extension -in ".zip", ".exe" } | Sort-Object Name
$hashLines = foreach ($package in $packages) {
    $hash = Get-FileHash -Path $package.FullName -Algorithm SHA256
    "$($hash.Hash.ToLowerInvariant())  $($package.Name)"
}

Set-Content -Path $checksumPath -Value $hashLines -Encoding ASCII

Write-Host "Release assets created:"
Get-ChildItem -Path $releaseDir -File | Sort-Object Name | ForEach-Object {
    Write-Host " - $($_.FullName) ($([math]::Round($_.Length / 1MB, 2)) MB)"
}
