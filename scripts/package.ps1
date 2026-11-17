# Package Timesheet for Windows Distribution
$ErrorActionPreference = "Stop"

Write-Host "Building release executable..." -ForegroundColor Cyan
cargo build --release

$Version = "0.11.1"
$DistDir = "dist\Timesheet-v$Version-Windows-x64"
$ZipPath = "dist\Timesheet-v$Version-Windows-x64.zip"

if (Test-Path $DistDir) {
    Remove-Item -Path $DistDir -Recurse -Force
}
if (Test-Path $ZipPath) {
    Remove-Item -Path $ZipPath -Force
}

New-Item -ItemType Directory -Path $DistDir -Force | Out-Null

Copy-Item -Path "target\release\timesheet.exe" -Destination "$DistDir\timesheet.exe" -Force
Copy-Item -Path "scripts\install.bat" -Destination "$DistDir\install.bat" -Force
Copy-Item -Path "scripts\install.ps1" -Destination "$DistDir\install.ps1" -Force
Copy-Item -Path "scripts\uninstall.bat" -Destination "$DistDir\uninstall.bat" -Force
Copy-Item -Path "scripts\uninstall.ps1" -Destination "$DistDir\uninstall.ps1" -Force

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

Write-Host "`nPackage created successfully at:" -ForegroundColor Green
Write-Host "  Folder: $DistDir" -ForegroundColor White
Write-Host "  Zip:    $ZipPath" -ForegroundColor White
