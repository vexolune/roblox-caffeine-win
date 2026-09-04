<div align="center">

```text
 ___  ___  ___ _    _____  __   ___   _   ___ ___ ___ ___ _  _ ___ 
| _ \/ _ \| _ ) |  / _ \ \/  / __| /_\ | __| __| __|_ _| \| | __|
|   / (_) | _ \ |_| (_) >  <  | (__ / _ \| _|| _|| _| | || .` | _| 
|_|_\\___/|___/____\___/_/\_\  \___/_/ \_\_| |_| |___|___|_|\_|___|
                      [ Windows Edition v2.0 ]
```

### Keep all your Roblox windows awake on Windows — zero focus stealing
#### Ported & expanded from the original Linux project by [EbadShelby](https://github.com/EbadShelby/roblox-caffeine)

[![GitHub Release](https://img.shields.io/github/v/release/vexolune/roblox-caffeine-win?style=flat-square&color=blue)](https://github.com/vexolune/roblox-caffeine-win/releases)
[![Original Project](https://img.shields.io/badge/Original%20Linux%20Project-EbadShelby%2Froblox--caffeine-orange?style=flat-square&logo=github)](https://github.com/EbadShelby/roblox-caffeine)
[![Python 3.8+](https://img.shields.io/badge/Python-3.8%2B-blue.svg?style=flat-square&logo=python&logoColor=white)](https://www.python.org/)
[![Windows](https://img.shields.io/badge/Platform-Windows%2010%20%2F%2011-blue.svg?style=flat-square&logo=windows&logoColor=white)](https://microsoft.com/windows)
[![ViGEmBus](https://img.shields.io/badge/Driver-ViGEmBus%20(WHQL)-informational.svg?style=flat-square)](https://github.com/nefarius/ViGEmBus)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg?style=flat-square)](LICENSE)

*A minimalist, zero-focus-stealing anti-AFK utility that prevents Roblox's 20-minute inactivity kick using a virtual Xbox 360 gamepad (ViGEmBus). Keeps **all** open Roblox windows alive simultaneously without touching your mouse, keyboard, or window focus.*

</div>

---

## ☕ About This Contribution

This repository is a comprehensive Windows port and enhancement of the original [roblox-caffeine](https://github.com/EbadShelby/roblox-caffeine) created by **[EbadShelby](https://github.com/EbadShelby)**.

| Feature | Original Linux ([@EbadShelby](https://github.com/EbadShelby/roblox-caffeine)) | Windows Edition (This Contribution) |
| :--- | :---: | :---: |
| **Gamepad Driver** | Linux Kernel `/dev/uinput` | Windows WHQL [ViGEmBus](https://github.com/nefarius/ViGEmBus) |
| **Virtual Device** | Microsoft Xbox 360 Gamepad | Microsoft Xbox 360 Gamepad |
| **Input Stealing** | ❌ None (zero mouse/keyboard impact) | ❌ None (zero mouse/keyboard impact) |
| **Multi-Window** | Target: Sober / SDL2 window | ✅ Global XInput broadcast (**all** windows at once) |
| **Interface** | Terminal CLI daemon | ✅ Terminal CLI **+** Windows System Tray GUI v2.0 |
| **Standalone Exe** | Python script | ✅ Portable `.exe` (no Python installation required) |
| **System Integration** | systemd service | ✅ Tray menu, Windows Toast, Mutex, Auto-start registry |

---

## 🚀 Why Roblox Caffeine?

Traditional anti-AFK methods on Windows like auto-clickers, macros, or Lua script injectors either:
- ❌ **Hijack your mouse/keyboard** (interrupting your work, typing, or browsing).
- ❌ **Require the Roblox window to stay in focus/foreground**.
- ❌ **Fail on multiple Roblox accounts/instances**.
- ⚠️ **Risk account bans** from Hyperion/Byfron anti-cheat due to memory injection or DLL hooking.

**Roblox Caffeine takes the hardware-level approach pioneered by EbadShelby:**
- **Virtual Xbox 360 Gamepad**: Creates a real Microsoft Xbox 360 gamepad at the kernel level via [ViGEmBus](https://github.com/nefarius/ViGEmBus) — the trusted WHQL-signed driver used by **DS4Windows** and **Steam Input**.
- **Zero Focus Stealing**: You can play other games, write code, or browse while Roblox is minimised or on another virtual desktop. Your mouse and keyboard are never touched.
- **Multi-Instance Support**: Because Windows XInput broadcasts gamepad states system-wide, a single virtual stick deflection resets the idle timer in **every** running Roblox instance at once.
- **100% Anticheat Safe**: Zero process injection, zero memory reading, zero DLL hooking. Roblox detects the input as coming from a genuine physical controller.

---

## ✨ Features (v2.0 System Tray Edition)

- ☕ **System Tray Daemon**: Runs silently in the Windows system tray with zero console clutter.
- ⏱️ **Real-Time Countdown**: Hover over the tray icon to see exact countdown until the next pulse (e.g., `Next: 9m 45s | 2 Roblox`).
- 🎨 **Dynamic Status Dot**: Color-coded icon states (🟢 Active · 🟡 Paused · 🔴 Error).
- 🛡️ **Single Instance Check**: Windows Named Mutex (`Global\RobloxCaffeineMutex_2_0`) prevents duplicate processes.
- ⚙️ **Configurable Pulse Interval**: Choose between **5, 10, 15, or 20 minutes** directly from the tray menu (persists in `%APPDATA%\RobloxCaffeine\settings.json`).
- 🚀 **Auto-Start with Windows**: Toggle `[✓] Start with Windows` with one click from the tray menu.
- ⚡ **Smart UAC Elevation**: Runs cleanly as a standard non-admin user. UAC is requested only once on the first run if the ViGEmBus driver is not yet installed.
- 🔔 **Rich Toast Notifications**: Notifies you on startup with active status, Roblox instance count, and interval settings.
- 🎮 **Manual Pulse & Pause**: Send a pulse immediately or pause anti-AFK without quitting.

---

## 📥 Quick Start (For Players / End Users)

No Python installation required!

1. Download the latest release from **[GitHub Releases](https://github.com/vexolune/roblox-caffeine-win/releases/latest)**:
   - **`RobloxCaffeine-v2.0.0-Windows.zip`** (Recommended package)
   - Or download **`RobloxCaffeine.exe`** directly.
2. Extract the zip and run **`Setup.bat`** (or double-click **`RobloxCaffeine.exe`**).
   - If prompted by UAC on the first launch, click **Yes** to allow the ViGEmBus driver installation.
3. Look for the coffee cup icon (☕) in the bottom-right system tray.
4. That's it! Minimise Roblox and work or play freely.

---

## 🛠️ Developer Setup & Running from Source

### Requirements
- Windows 10 / 11 (64-bit)
- Python 3.8+
- Dependencies: `vgamepad`, `psutil`, `pywin32`, `pystray`, `Pillow`

### 1. Installation
Clone the repository and install requirements:
```powershell
git clone https://github.com/EbadShelby/roblox-caffeine.git
cd roblox-caffeine/roblox-caffeine-win
pip install -r requirements.txt
# Or run install.bat directly
```

### 2. Run in Terminal (CLI Mode)
Runs with the classic terminal dashboard:
```powershell
python roblox_caffeine_win.py
```

### 3. Run in System Tray (GUI Mode)
Runs silently in the Windows taskbar:
```powershell
python roblox_caffeine_tray.py
```

### 4. Build Standalone `.exe`
To compile the standalone executable yourself:
```powershell
.\build.bat
```
Output will be generated in `dist\RobloxCaffeine.exe` and synced to `release\RobloxCaffeine.exe`.

---

## 📁 Repository Structure

```text
roblox-caffeine-win/
├── release/                        # Ready-to-use distribution
│   ├── RobloxCaffeine.exe         # Prebuilt standalone executable
│   ├── Setup.bat                  # First-run driver installer
│   └── README.txt                 # End-user quick start guide
├── roblox_caffeine_tray.py        # v2.0 System Tray implementation
├── roblox_caffeine_win.py         # Classic CLI terminal daemon
├── build.bat                      # Build script (Nuitka / PyInstaller)
├── install.bat                    # Dependency installer
├── icon.ico                       # Embedded multi-resolution coffee icon
├── icon.png                       # Asset preview image
├── pyproject.toml                 # Package definition
├── PULL_REQUEST.md                # Ready-to-submit PR documentation for upstream
├── LICENSE                        # MIT License
└── README.md                      # This documentation
```

---

## 🤝 Contribution to Upstream (`EbadShelby/roblox-caffeine`)

This project was developed to be contributed directly back to the original author **[@EbadShelby](https://github.com/EbadShelby)** to turn **roblox-caffeine** into a complete cross-platform solution:

- **Linux**: Handled by EbadShelby's native `/dev/uinput` daemon for Sober/Wine.
- **Windows**: Handled by this ViGEmBus + System Tray edition for the native Windows Roblox client.

If you are merging this into upstream, refer to [`PULL_REQUEST.md`](PULL_REQUEST.md) for a summary of changes, architecture details, and verification steps.

---

## ❓ FAQ & Troubleshooting

<details>
<summary><b>Why did a UAC prompt appear on first launch?</b></summary>

ViGEmBus is a kernel-mode virtual bus driver created by Nefarius. Windows requires Administrator privileges to register any virtual hardware device on the system. With v2.0's Smart UAC, this prompt **only appears once** on the initial run. Subsequent launches run in normal user mode.
</details>

<details>
<summary><b>Will this trigger Hyperion / Byfron anti-cheat bans?</b></summary>

No. Anti-cheat software monitors process memory, thread hooks, and injected DLLs. Roblox Caffeine does not touch Roblox processes. It simply tells the operating system that an Xbox controller thumbstick moved for 300 milliseconds. Roblox queries Windows for standard controller events via `XInputGetState` and cannot distinguish it from a physical gamepad.
</details>

<details>
<summary><b>How does multi-window support work?</b></summary>

Windows handles XInput devices at the system level. When a game supports controller input, it polls the controller state regardless of whether other Roblox instances are open. When Roblox Caffeine sends an input event, all active Roblox processes receive the update simultaneously.
</details>

<details>
<summary><b>Why did my character step slightly?</b></summary>

Every 10 minutes (or your configured interval), the left stick deflects for 0.3 seconds and immediately re-centers. This resets Roblox's 20-minute idle timer. We recommend placing your avatar in a safe place or lobby when AFK.
</details>

---

## 📜 Credits & Acknowledgements

- **[EbadShelby](https://github.com/EbadShelby)** — Original creator and author of [roblox-caffeine](https://github.com/EbadShelby/roblox-caffeine) for Linux. The concept, interval timing, and thumbstick deflection math are based on his original work.
- **[Benjamin Höglinger-Stelzer (Nefarius)](https://github.com/nefarius/ViGEmBus)** — Creator of the ViGEmBus driver.
- **[yannbouteiller](https://github.com/yannbouteiller/vgamepad)** — Python bindings for ViGEmClient.
- **[moses-palmer](https://github.com/moses-palmer/pystray)** — Python system tray library.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE) — matching the upstream repository.
Copyright (c) 2024-2026 EbadShelby and Contributors.
