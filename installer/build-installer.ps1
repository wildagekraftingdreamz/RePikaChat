<#
.SYNOPSIS
  Builds RePikaChat and packages it for Windows.

.DESCRIPTION
  Runs `cargo build --release`, stages a portable folder, and builds a real
  setup.exe when Inno Setup 6 is installed. Without Inno Setup it still produces
  a working portable ZIP containing Install.cmd, which performs the same
  per-user install the binary supports natively (--install).

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File .\installer\build-installer.ps1
#>
param(
  [switch]$SkipBuild,
  [string]$Configuration = 'release'
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

function Find-Cargo {
  $candidates = @(
    (Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'),
    'cargo.exe'
  )
  foreach ($c in $candidates) {
    try {
      $cmd = Get-Command $c -ErrorAction Stop
      return $cmd.Source
    } catch { }
  }
  throw "cargo was not found. Install Rust from https://rustup.rs and try again."
}

$cargo = Find-Cargo
Write-Host "Using cargo: $cargo"

if (-not $SkipBuild) {
  Write-Host "Building RePikaChat ($Configuration)..."
  & $cargo build --release
  if ($LASTEXITCODE -ne 0) { throw "cargo build failed with exit code $LASTEXITCODE" }
}

$exe = Join-Path $root "target\$Configuration\RePikaChat.exe"
if (-not (Test-Path $exe)) { throw "Expected binary not found: $exe" }

$version = '0.1.0'
$cargoToml = Get-Content (Join-Path $root 'Cargo.toml') -Raw
$m = [regex]::Match($cargoToml, '(?m)^version\s*=\s*"([^"]+)"')
if ($m.Success) { $version = $m.Groups[1].Value }

$dist = Join-Path $root 'dist'
$portable = Join-Path $dist 'RePikaChat'
if (Test-Path $portable) { Remove-Item $portable -Recurse -Force }
New-Item -ItemType Directory -Force -Path $portable | Out-Null

Copy-Item $exe (Join-Path $portable 'RePikaChat.exe') -Force
Copy-Item (Join-Path $root 'README.md') $portable -Force
Copy-Item (Join-Path $root 'LICENSE') $portable -Force

$installCmd = @'
@echo off
rem Per-user install: copies the app to %LOCALAPPDATA%\Programs\RePikaChat and
rem adds a Start Menu shortcut. No administrator rights needed.
"%~dp0RePikaChat.exe" --install
echo.
pause
'@
Set-Content -Path (Join-Path $portable 'Install.cmd') -Value $installCmd -Encoding ASCII

$zip = Join-Path $dist "RePikaChat-$version-portable.zip"
if (Test-Path $zip) { Remove-Item $zip -Force }
Compress-Archive -Path (Join-Path $portable '*') -DestinationPath $zip -CompressionLevel Optimal
Write-Host "Portable build: $zip"

$isccCandidates = @(
  (Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe'),
  (Join-Path $env:ProgramFiles 'Inno Setup 6\ISCC.exe')
)
$iscc = $isccCandidates | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1

if ($iscc) {
  Write-Host "Building installer with $iscc"
  & $iscc "/DMyAppVersion=$version" (Join-Path $root 'installer\RePikaChat.iss')
  if ($LASTEXITCODE -ne 0) { throw "ISCC failed with exit code $LASTEXITCODE" }
  Write-Host "Installer: $(Join-Path $dist "RePikaChat-$version-setup.exe")"
} else {
  Write-Warning "Inno Setup 6 was not found, so no setup.exe was produced."
  Write-Host "The portable ZIP contains Install.cmd, which installs the same way."
  Write-Host "Install Inno Setup (https://jrsoftware.org/isdl.php) and rerun this script for a setup.exe."
}
