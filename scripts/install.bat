@echo off
setlocal
cd /d "%~dp0"
echo ========================================================
echo Installing Timesheet Desktop for Windows...
echo ========================================================
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1"
if %errorlevel% neq 0 (
    echo.
    echo Installation encountered an error.
    pause
)
