# Timesheet Windows Installer Script (v0.11.2)
param(
    [switch]$NoDesktopShortcut,
    [switch]$NoLaunch
)

$ErrorActionPreference = "Stop"

$AppName = "Timesheet"
$AppVersion = "0.11.2"
$ExeName = "timesheet.exe"

$Candidates = @(
    (Join-Path $PSScriptRoot $ExeName),
    (Join-Path $PSScriptRoot "target\release\$ExeName"),
    (Join-Path $PSScriptRoot "..\target\release\$ExeName"),
    (Join-Path (Get-Location) $ExeName),
    (Join-Path (Get-Location) "target\release\$ExeName"),
    (Join-Path (Get-Location) "dist\Timesheet-v$AppVersion-Windows-x64\$ExeName")
)

$SourceExe = $Candidates | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $SourceExe) {
    Write-Error "Could not find $ExeName in '$PSScriptRoot' or target\release."
    exit 1
}

$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\$AppName"
Write-Host "Installing $AppName v$AppVersion to: $InstallDir..." -ForegroundColor Cyan

# Terminate running instances before updating files
Get-Process -Name "timesheet" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# Ensure target directory exists
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Copy executable
$TargetExe = Join-Path $InstallDir $ExeName
Copy-Item -Path $SourceExe -Destination $TargetExe -Force
Write-Host "Copied $ExeName to $InstallDir" -ForegroundColor Green

# Copy uninstaller script to install directory
$UninstallerPath = Join-Path $InstallDir "uninstall.ps1"
$UninstallerCandidates = @(
    (Join-Path $PSScriptRoot "uninstall.ps1"),
    (Join-Path $PSScriptRoot "scripts\uninstall.ps1"),
    (Join-Path $PSScriptRoot "..\scripts\uninstall.ps1")
)
$SourceUninstaller = $UninstallerCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if ($SourceUninstaller) {
    Copy-Item -Path $SourceUninstaller -Destination $UninstallerPath -Force
}

# Create Start Menu shortcut
$StartMenuDir = [Environment]::GetFolderPath("Programs")
$StartShortcutPath = Join-Path $StartMenuDir "$AppName.lnk"

$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut($StartShortcutPath)
$Shortcut.TargetPath = $TargetExe
$Shortcut.WorkingDirectory = $InstallDir
$Shortcut.Description = "Timesheet - Offline-first Windows Desktop Timesheet"
$Shortcut.Save()
Write-Host "Created Start Menu shortcut: $StartShortcutPath" -ForegroundColor Green

# Create Desktop shortcut if requested
if (-not $NoDesktopShortcut) {
    $DesktopDir = [Environment]::GetFolderPath("Desktop")
    $DesktopShortcutPath = Join-Path $DesktopDir "$AppName.lnk"
    $DShortcut = $WshShell.CreateShortcut($DesktopShortcutPath)
    $DShortcut.TargetPath = $TargetExe
    $DShortcut.WorkingDirectory = $InstallDir
    $DShortcut.Description = "Timesheet - Offline-first Windows Desktop Timesheet"
    $DShortcut.Save()
    Write-Host "Created Desktop shortcut: $DesktopShortcutPath" -ForegroundColor Green
}

# Register in Windows Add/Remove Programs (HKCU)
$RegKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$AppName"
if (-not (Test-Path $RegKey)) {
    New-Item -Path $RegKey -Force | Out-Null
}
Set-ItemProperty -Path $RegKey -Name "DisplayName" -Value "$AppName"
Set-ItemProperty -Path $RegKey -Name "DisplayVersion" -Value "$AppVersion"
Set-ItemProperty -Path $RegKey -Name "Publisher" -Value "Timesheet Desktop"
Set-ItemProperty -Path $RegKey -Name "DisplayIcon" -Value "$TargetExe,0"
Set-ItemProperty -Path $RegKey -Name "InstallLocation" -Value "$InstallDir"
Set-ItemProperty -Path $RegKey -Name "UninstallString" -Value "powershell.exe -ExecutionPolicy Bypass -File `"$UninstallerPath`""

Write-Host "`n$AppName v$AppVersion installed successfully!" -ForegroundColor Green
Write-Host "Find it in your Start Menu, on your Desktop, or in Windows Installed Apps." -ForegroundColor White

if (-not $NoLaunch) {
    Write-Host "Launching $AppName..." -ForegroundColor Cyan
    Start-Process -FilePath $TargetExe
}
