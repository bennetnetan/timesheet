# Package Timesheet for Windows Distribution
$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path $PSScriptRoot -Parent
Set-Location $RepoRoot

$CargoToml = Get-Content "Cargo.toml" -Raw
$VersionMatch = [regex]::Match($CargoToml, '(?m)^version\s*=\s*"([^"]+)"')
if (-not $VersionMatch.Success) {
    throw "Could not read the package version from Cargo.toml."
}
$Version = $VersionMatch.Groups[1].Value

Write-Host "Building release executable..." -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -ne 0) {
    throw "cargo build --release failed with exit code $LASTEXITCODE."
}

$DistDir = "dist\Timesheet-v$Version-Windows-x64"
$ZipPath = "dist\Timesheet-v$Version-Windows-x64.zip"
$SetupExe = "dist\Timesheet-Setup.exe"
$RawExe = "dist\timesheet.exe"

if (Test-Path $DistDir) {
    Remove-Item -Path $DistDir -Recurse -Force
}
if (Test-Path $ZipPath) {
    Remove-Item -Path $ZipPath -Force
}
foreach ($artifact in @($SetupExe, $RawExe)) {
    if (Test-Path $artifact) {
        Remove-Item -Path $artifact -Force
    }
}

New-Item -ItemType Directory -Path $DistDir -Force | Out-Null

Copy-Item -Path "target\release\timesheet.exe" -Destination "$DistDir\timesheet.exe" -Force
Copy-Item -Path "target\release\timesheet.exe" -Destination $RawExe -Force
Copy-Item -Path "scripts\install.bat" -Destination "$DistDir\install.bat" -Force
Copy-Item -Path "scripts\uninstall.bat" -Destination "$DistDir\uninstall.bat" -Force
Copy-Item -Path "scripts\uninstall.ps1" -Destination "$DistDir\uninstall.ps1" -Force

$InstallScript = Get-Content "scripts\install.ps1" -Raw
$InstallScript = $InstallScript.Replace("__APP_VERSION__", $Version)
Set-Content -Path "$DistDir\install.ps1" -Value $InstallScript -Encoding UTF8

$ReadmeContent = @"
========================================================
Timesheet Desktop for Windows (v$Version)
Offline-First Windows Desktop Timesheet App
========================================================

HOW TO RUN OR INSTALL:

Option 1: Automatic 1-Click Install (Recommended)
-------------------------------------------------
1. Double-click "install.bat" (or right-click "install.ps1" -> Run with PowerShell).
2. It installs Timesheet to %LOCALAPPDATA%\Programs\Timesheet\ (no Administrator rights needed).
3. It creates Start Menu and Desktop shortcuts and registers the app in Windows Settings.
4. Timesheet launches immediately!

Option 2: Standalone Portable Mode
----------------------------------
No installation is required. You can simply double-click "timesheet.exe" from any folder or USB drive.
All your time entries and settings are automatically stored offline in:
  %APPDATA%\timesheet\Timesheet\timesheet.db

HOW TO UNINSTALL:
-----------------
Double-click "uninstall.bat" or uninstall from Windows Settings -> Apps -> Installed Apps.
Your database is preserved so you never lose your tracked data.
"@

Set-Content -Path "$DistDir\README_INSTALL.txt" -Value $ReadmeContent

Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::CreateFromDirectory($DistDir, $ZipPath)

# Compile standalone single-file installer (Timesheet-Setup.exe)
$CscPath = "C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe"
if (-not (Test-Path $CscPath)) {
    throw "C# compiler was not found at '$CscPath'."
}

Write-Host "Compiling standalone single-file installer ($SetupExe)..." -ForegroundColor Cyan
$SetupSource = Get-Content "scripts\Setup.cs" -Raw
$SetupSource = $SetupSource.Replace("__APP_VERSION__", $Version)
$SetupSourcePath = Join-Path $DistDir "Setup.cs"
Set-Content -Path $SetupSourcePath -Value $SetupSource -Encoding UTF8
& $CscPath /nologo /target:winexe /optimize+ `
    /reference:System.dll,System.Drawing.dll,System.Windows.Forms.dll,System.IO.Compression.dll,System.IO.Compression.FileSystem.dll,Microsoft.CSharp.dll `
    "/resource:$ZipPath,Package.zip" `
    "/out:$SetupExe" $SetupSourcePath
if ($LASTEXITCODE -ne 0) {
    throw "Compiling the standalone installer failed with exit code $LASTEXITCODE."
}
Remove-Item -Path $SetupSourcePath -Force

Write-Host "`nPackage created successfully!" -ForegroundColor Green
Write-Host "  Single-file Setup: $SetupExe" -ForegroundColor White
Write-Host "  Extracted Folder:  $DistDir" -ForegroundColor White
Write-Host "  Zip Archive:       $ZipPath" -ForegroundColor White
Write-Host "  Raw executable:    $RawExe" -ForegroundColor White
