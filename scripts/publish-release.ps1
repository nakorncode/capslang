param(
    [ValidateSet("win-x64")]
    [string] $Runtime = "win-x64",

    [ValidateSet("Release", "Debug")]
    [string] $Configuration = "Release",

    [string] $ProductVersion,

    [switch] $SkipInstaller
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptDir
$projectPath = Join-Path $repoRoot "CapsLang.csproj"
$publishDir = Join-Path $repoRoot "artifacts\publish\$Runtime"
$releaseDir = Join-Path $repoRoot "artifacts\release"
$installerDir = Join-Path $repoRoot "artifacts\installer"
$zipPath = Join-Path $releaseDir "CapsLang-Portable-$Runtime.zip"
$setupExePath = Join-Path $releaseDir "CapsLang-Setup-$Runtime.exe"
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

    [xml] $projectXml = Get-Content $projectPath
    return $projectXml.Project.PropertyGroup.Version
}

function Get-FileVersion([string] $version) {
    $parts = $version.Split(".")
    while ($parts.Count -lt 4) {
        $parts += "0"
    }

    return ($parts | Select-Object -First 4) -join "."
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
$fileVersion = Get-FileVersion $resolvedVersion

$runningApps = Get-Process -Name "CapsLang" -ErrorAction SilentlyContinue
if ($runningApps) {
    Write-Host "Stopping running CapsLang process..."
    $runningApps | Stop-Process -Force
}

if (Test-Path $publishDir) {
    Remove-Item -LiteralPath $publishDir -Recurse -Force
}

New-Item -ItemType Directory -Force -Path $publishDir, $releaseDir, $installerDir | Out-Null

Get-ChildItem -Path $releaseDir -File -Filter "CapsLang-*" -ErrorAction SilentlyContinue | Remove-Item -Force
Get-ChildItem -Path $installerDir -File -Filter "CapsLang-*" -ErrorAction SilentlyContinue | Remove-Item -Force

dotnet publish $projectPath `
    --configuration $Configuration `
    --runtime $Runtime `
    --self-contained true `
    --output $publishDir `
    -p:PublishSingleFile=true `
    -p:IncludeNativeLibrariesForSelfExtract=true `
    -p:DebugType=None `
    -p:DebugSymbols=false `
    -p:Version=$resolvedVersion `
    -p:FileVersion=$fileVersion `
    -p:AssemblyVersion=$fileVersion `
    -p:InformationalVersion=$resolvedVersion

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
    Write-Host " - $($_.FullName)"
}
