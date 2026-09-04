@echo off
title Roblox Caffeine — Setup Info
color 0A
mode con: cols=60 lines=28

echo.
echo  ╔══════════════════════════════════════════════╗
echo  ║        Roblox Caffeine — Quick Setup         ║
echo  ║           Anti-AFK for Roblox (Windows)      ║
echo  ╚══════════════════════════════════════════════╝
echo.
echo  HOW TO USE:
echo.
echo  1. Close this window
echo  2. Double-click  RobloxCaffeine.exe
echo  3. Click YES on the UAC prompt
echo     (required to install the virtual controller
echo      driver on first run only)
echo  4. Look for the coffee cup icon in your taskbar
echo     (bottom-right, system tray area)
echo.
echo  ─────────────────────────────────────────────
echo.
echo  WHY THE UAC PROMPT?
echo.
echo  RobloxCaffeine uses the ViGEmBus driver to
echo  create a virtual Xbox controller. This is the
echo  same trusted driver used by DS4Windows and
echo  Steam Input. The UAC prompt only appears on
echo  the FIRST run — never again after that.
echo.
echo  ─────────────────────────────────────────────
echo.
echo  Press any key to launch RobloxCaffeine now...
pause > nul

start "" "%~dp0RobloxCaffeine.exe"
