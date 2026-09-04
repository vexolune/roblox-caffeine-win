@echo off
title Roblox Caffeine (Windows) — Installer
echo.
echo  Installing Roblox Caffeine (Windows) dependencies...
echo  ─────────────────────────────────────────────────────
echo.
echo  NOTE: vgamepad will install the ViGEmBus kernel driver.
echo  A UAC (User Account Control) prompt will appear — this is normal.
echo  ViGEmBus is the same signed driver used by DS4Windows and Steam Input.
echo.
pause

pip install vgamepad psutil pywin32 pystray Pillow

echo.
echo  ─────────────────────────────────────────────────────
echo  Done! Run the tool with:
echo.
echo    python roblox_caffeine_win.py
echo.
pause
