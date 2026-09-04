# Pull Request: Add Windows Support (ViGEmBus Virtual Gamepad & System Tray Edition v2.0)

**Upstream Target:** [EbadShelby/roblox-caffeine](https://github.com/EbadShelby/roblox-caffeine)  
**Type of Change:** New Platform Support / Major Feature  

---

## 📌 Summary

First of all, huge thanks to **@EbadShelby** for creating the original **roblox-caffeine** on Linux! The concept of using a kernel-level virtual Xbox 360 gamepad to bypass the 20-minute idle kick without hijacking keyboard/mouse or injecting code is brilliant.

This PR introduces full **Windows support** (`roblox-caffeine-win`), maintaining parity with the Linux `/dev/uinput` approach while adding native Windows conveniences such as a background System Tray daemon and a standalone portable `.exe`.

---

## 🎮 How it Works on Windows

| Platform | Linux ([@EbadShelby](https://github.com/EbadShelby/roblox-caffeine)) | Windows Port (This PR) |
|---|---|---|
| **Driver** | Linux kernel `/dev/uinput` | Microsoft-signed [ViGEmBus](https://github.com/nefarius/ViGEmBus) (WHQL) |
| **Emulated Device** | Xbox 360 Gamepad | Xbox 360 Gamepad |
| **Focus Stealing** | ❌ None (zero mouse/keyboard impact) | ❌ None (zero mouse/keyboard impact) |
| **Multi-Window** | Targets Sober/SDL2 | ✅ Global XInput broadcast (**all** open Roblox clients at once) |
| **Modes** | CLI daemon / systemd service | CLI terminal dashboard + System Tray daemon (v2.0) |
| **Standalone** | Python package | Portable `.exe` (no Python installation required) |

Because Windows XInput broadcasts gamepad state changes system-wide, a single thumbstick pulse resets the AFK timer across **every running Roblox window** simultaneously.

---

## ✨ Features Added

1. **Classic CLI Dashboard (`roblox_caffeine_win.py`)**:
   - Matches the original Linux terminal aesthetic (Rich box layout, live countdown progress bar, active pulse logger).
2. **System Tray Daemon v2.0 (`roblox_caffeine_tray.py`)**:
   - Runs 100% in the background with a custom coffee cup icon (☕).
   - **Real-Time Hover Tooltip**: Dynamic countdown until the next pulse (e.g., `Roblox Caffeine — Active | Next: 9m 45s | 2 Roblox`).
   - **Status Dots**: Color-coded icon indicators (🟢 Active · 🟡 Paused · 🔴 Error).
   - **Configurable Interval**: Choose between 5, 10, 15, or 20 minutes (settings persist across restarts in `%APPDATA%\RobloxCaffeine\settings.json`).
   - **Single Instance Protection**: Windows Named Mutex (`Global\RobloxCaffeineMutex_2_0`) prevents accidental double launches.
   - **Smart UAC**: Runs as standard user privilege without UAC popups; only prompts for elevation if the ViGEmBus driver is missing on first launch.
   - **Auto-Start with Windows**: Toggleable directly from the tray menu (`HKCU\...\Run`).
3. **Standalone Distribution (`release/`)**:
   - `RobloxCaffeine.exe` (portable, no Python needed).
   - `Setup.bat` (first-time helper script for driver install).
   - `README.txt` (clear user guide with full credits to @EbadShelby).

---

## 🧪 Testing & Verification

- [x] Tested on Windows 10 & Windows 11 (64-bit).
- [x] Tested single Roblox client and multiple simultaneous Roblox clients (multi-window XInput broadcast verified).
- [x] Verified zero mouse or keyboard interruption while typing or playing another game.
- [x] Verified single instance mutex prevents duplicate background processes.
- [x] Verified interval switching (5m, 10m, 15m, 20m) and settings persistence.
- [x] Verified registry auto-start on boot toggle.

---

## 📄 License & Attribution

- Released under the same **MIT License** matching upstream.
- All documentation prominently attributes original ownership and architecture to **@EbadShelby**.
