@echo off
rem Updates Spiri72 in this folder to the newest build. Double-click it.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0update.ps1" %*
echo.
pause
