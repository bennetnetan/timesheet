# Timesheet Windows Uninstaller Script
$AppName = "Timesheet"
$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\$AppName"

Write-Host "Uninstalling $AppName..." -ForegroundColor Cyan

# Terminate running instances
Get-Process -Name "timesheet" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# Remove Start Menu shortcut
$StartMenuDir = [Environment]::GetFolderPath("Programs")
$StartShortcutPath = Join-Path $StartMenuDir "$AppName.lnk"
if (Test-Path $StartShortcutPath) {
    Remove-Item -Path $StartShortcutPath -Force
    Write-Host "Removed Start Menu shortcut." -ForegroundColor Gray
}

# Remove Desktop shortcut
$DesktopDir = [Environment]::GetFolderPath("Desktop")
$DesktopShortcutPath = Join-Path $DesktopDir "$AppName.lnk"
if (Test-Path $DesktopShortcutPath) {
    Remove-Item -Path $DesktopShortcutPath -Force
    Write-Host "Removed Desktop shortcut." -ForegroundColor Gray
}

# Remove Windows Apps Registry entry
$RegKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$AppName"
if (Test-Path $RegKey) {
    Remove-Item -Path $RegKey -Recurse -Force
    Write-Host "Removed Windows Apps & Features registry entry." -ForegroundColor Gray
}

# Remove program files (leaves user database in %APPDATA%\timesheet safe and intact!)
if (Test-Path $InstallDir) {
    Remove-Item -Path $InstallDir -Recurse -Force
    Write-Host "Removed installation directory: $InstallDir" -ForegroundColor Gray
}

Write-Host "`n$AppName has been uninstalled." -ForegroundColor Green
Write-Host "Note: Your timesheet database in %APPDATA%\timesheet was preserved so you won't lose your tracked time." -ForegroundColor White
