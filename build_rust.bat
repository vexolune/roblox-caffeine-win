@echo off
title Building Roblox Caffeine (Rust Edition)
color 0B
echo.
echo  ======================================================
echo    Building Roblox Caffeine v2.1.0 in Rust (Ultra-Lightweight)
echo  ======================================================
echo.

cargo build --release -j 2

if %ERRORLEVEL% equ 0 (
    echo.
    echo  [OK] Rust release build SUCCEEDED!
    echo.
    copy /Y "target\x86_64-pc-windows-gnu\release\roblox-caffeine-win.exe" "release\RobloxCaffeine.exe" >nul
    echo  [OK] Copied binary to release\RobloxCaffeine.exe
    echo.
    powershell -Command "Get-Item 'release\RobloxCaffeine.exe' | ForEach-Object { Write-Host ('  Final Size: ' + [math]::Round($_.Length/1KB, 2) + ' KB (' + [math]::Round($_.Length/1MB, 2) + ' MB)') }"
    echo.
    echo  ======================================================
) else (
    echo.
    echo  [ERROR] Cargo build failed. Check errors above.
    echo  ======================================================
)
pause
