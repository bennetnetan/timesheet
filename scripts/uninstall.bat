@echo off
setlocal
cd /d "%~dp0"
echo ========================================================
echo Uninstalling Timesheet Desktop for Windows...
echo ========================================================
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0uninstall.ps1"
pause
