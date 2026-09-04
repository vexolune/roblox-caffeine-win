@echo off
title Roblox Caffeine v2.0 — Build
color 0A
echo.
echo  ╔══════════════════════════════════════════════════╗
echo  ║   Roblox Caffeine v2.0 — Build System Tray .exe  ║
echo  ╚══════════════════════════════════════════════════╝
echo.
echo  [1/3] Installing dependencies...
echo  ─────────────────────────────────────────────────────
echo.

pip install vgamepad psutil pywin32 pystray Pillow nuitka

echo.
echo  ─────────────────────────────────────────────────────
echo  [2/3] Building RobloxCaffeine.exe with Nuitka...
echo        (first build ~10-20 min, subsequent builds faster)
echo  ─────────────────────────────────────────────────────
echo.

python -m nuitka ^
    --onefile ^
    --windows-disable-console ^
    --windows-icon-from-ico=icon.ico ^
    --include-package=vgamepad ^
    --include-package=pystray ^
    --include-package=PIL ^
    --include-package=psutil ^
    --include-data-files=icon.ico=icon.ico ^
    --include-data-files=icon.png=icon.png ^
    --output-filename=RobloxCaffeine.exe ^
    --output-dir=dist ^
    roblox_caffeine_tray.py

echo.
echo  ─────────────────────────────────────────────────────
if exist "dist\RobloxCaffeine.exe" (
    echo  [3/3] SUCCESS!
    echo.
    echo         dist\RobloxCaffeine.exe  ^<-- your .exe is here!
    echo.
    echo  Double-click it. UAC only appears on first run
    echo  to install ViGEmBus. Never again after that.
    echo.
    echo  Updating release folder...
    copy /Y "dist\RobloxCaffeine.exe" "release\RobloxCaffeine.exe" >nul
    echo  release\RobloxCaffeine.exe updated!
) else (
    echo  [3/3] Nuitka build FAILED. Trying PyInstaller fallback...
    echo  ─────────────────────────────────────────────────────
    echo.
    pip install pyinstaller
    python -m PyInstaller ^
        --onefile ^
        --noconsole ^
        --icon=icon.ico ^
        --name "RobloxCaffeine" ^
        --collect-all vgamepad ^
        --add-data "icon.ico;." ^
        --add-data "icon.png;." ^
        roblox_caffeine_tray.py
    echo.
    if exist "dist\RobloxCaffeine.exe" (
        echo  PyInstaller fallback SUCCESS!
        copy /Y "dist\RobloxCaffeine.exe" "release\RobloxCaffeine.exe" >nul
        echo  release\RobloxCaffeine.exe updated!
    ) else (
        echo  Both Nuitka and PyInstaller failed. Check errors above.
    )
)
echo  ─────────────────────────────────────────────────────
echo.
pause
