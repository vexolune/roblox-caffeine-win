#!/usr/bin/env python3
r"""
Roblox Caffeine — System Tray Edition v2.0
============================================
What's new in v2.0:
  - Single instance check (Windows Mutex)
  - Real-time tray tooltip countdown
  - Smart UAC: only elevates once if ViGEmBus not installed
  - Auto-start with Windows (toggleable from menu)
  - Configurable interval: 5 / 10 / 15 / 20 min
  - Proper coffee cup .ico icon
  - Detailed startup notification with Roblox count
  - Settings persist across restarts (%APPDATA%\RobloxCaffeine\settings.json)

Dependencies:
    pip install vgamepad psutil pywin32 pystray Pillow
"""

from __future__ import annotations

import sys
import os
import time
import random
import threading
import ctypes
import json
import winreg
from pathlib import Path

# ── PIL / pystray ──────────────────────────────────────────────────────────
try:
    from PIL import Image, ImageDraw
    import pystray
    from pystray import MenuItem as Item, Menu
except ImportError:
    import subprocess
    subprocess.run([sys.executable, "-m", "pip", "install", "pystray", "Pillow"], check=False)
    from PIL import Image, ImageDraw  # type: ignore
    import pystray                    # type: ignore
    from pystray import MenuItem as Item, Menu  # type: ignore

# ── App constants ──────────────────────────────────────────────────────────
APP_NAME       = "Roblox Caffeine"
APP_VERSION    = "2.0.0"
MUTEX_NAME     = "Global\\RobloxCaffeineMutex_2_0"
AUTOSTART_KEY  = "RobloxCaffeine"
AUTOSTART_REG  = r"Software\Microsoft\Windows\CurrentVersion\Run"
SETTINGS_DIR   = Path(os.environ.get("APPDATA", ".")) / "RobloxCaffeine"
SETTINGS_FILE  = SETTINGS_DIR / "settings.json"

STICK_DEFLECTION = 22_000

ROBLOX_PROCESS_NAMES = {
    "robloxplayerbeta.exe",
    "robloxplayer.exe",
    "windows10universal.exe",
}

INTERVAL_OPTIONS = [
    ("5 minutes",  5 * 60),
    ("10 minutes", 10 * 60),
    ("15 minutes", 15 * 60),
    ("20 minutes", 20 * 60),
]

# Status dot colours
COLOR_ACTIVE = (34,  197,  94)   # green
COLOR_PAUSED = (234, 179,   8)   # yellow
COLOR_ERROR  = (239,  68,  68)   # red


# ===========================================================================
# Resource path — works frozen (PyInstaller/Nuitka) and in dev
# ===========================================================================

def _resource_path(filename: str) -> Path:
    """Return absolute path to a bundled resource."""
    if getattr(sys, "frozen", False) and hasattr(sys, "_MEIPASS"):
        # PyInstaller
        base = Path(sys._MEIPASS)
    elif getattr(sys, "frozen", False):
        # Nuitka onefile extracts next to exe
        base = Path(sys.argv[0]).parent
    else:
        base = Path(__file__).parent
    return base / filename


# ===========================================================================
# Icon helpers
# ===========================================================================

_BASE_ICON_CACHE: Image.Image | None = None


def _load_base_icon() -> Image.Image:
    """Load coffee cup icon once and cache it."""
    global _BASE_ICON_CACHE
    if _BASE_ICON_CACHE is None:
        for name in ("icon.ico", "icon.png"):
            p = _resource_path(name)
            if p.exists():
                img = Image.open(p).convert("RGBA")
                _BASE_ICON_CACHE = img.resize((64, 64), Image.LANCZOS)
                break
        else:
            _BASE_ICON_CACHE = _make_fallback_icon()
    return _BASE_ICON_CACHE.copy()


def _make_fallback_icon() -> Image.Image:
    """Programmatic fallback if no icon file found."""
    size = 64
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d   = ImageDraw.Draw(img)
    d.ellipse([2, 2, 62, 62], fill=(15, 15, 15))
    d.rounded_rectangle([14, 20, 44, 48], radius=4, fill=(200, 155, 100))
    d.arc([40, 26, 52, 42], start=270, end=90, fill=(180, 130, 80), width=4)
    for sx in (22, 29, 36):
        d.line([(sx, 14), (sx - 2, 9)], fill=(200, 200, 200), width=2)
    return img


def _make_icon(state: str = "active") -> Image.Image:
    """Return base icon with a colour-coded status dot overlay."""
    img = _load_base_icon()
    d   = ImageDraw.Draw(img)
    dot = {"active": COLOR_ACTIVE, "paused": COLOR_PAUSED, "error": COLOR_ERROR}.get(
        state, COLOR_ACTIVE
    )
    # Draw dot bottom-right
    d.ellipse([47, 47, 63, 63], fill=dot, outline=(0, 0, 0, 180), width=1)
    return img


# ===========================================================================
# Settings
# ===========================================================================

def _load_settings() -> dict:
    try:
        SETTINGS_DIR.mkdir(parents=True, exist_ok=True)
        if SETTINGS_FILE.exists():
            return json.loads(SETTINGS_FILE.read_text(encoding="utf-8"))
    except Exception:
        pass
    return {"interval_seconds": 10 * 60}


def _save_settings(data: dict):
    try:
        SETTINGS_DIR.mkdir(parents=True, exist_ok=True)
        SETTINGS_FILE.write_text(json.dumps(data, indent=2), encoding="utf-8")
    except Exception:
        pass


# ===========================================================================
# Helpers
# ===========================================================================

def _fmt_time(seconds: int) -> str:
    h, m = divmod(seconds // 60, 60)
    return f"{h}h {m:02d}m" if h else f"{m}m"


def _detect_roblox() -> int:
    """Count running Roblox processes."""
    try:
        import psutil
        count = 0
        for proc in psutil.process_iter(["name"]):
            try:
                if (proc.info.get("name") or "").lower() in ROBLOX_PROCESS_NAMES:
                    count += 1
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                pass
        return count
    except ImportError:
        return 0


def _simulate_pulse(gamepad) -> None:
    """Nudge virtual sticks for 0.3 s then re-centre."""
    x = random.choice([-STICK_DEFLECTION, STICK_DEFLECTION])
    y = random.choice([-STICK_DEFLECTION, STICK_DEFLECTION])
    gamepad.left_joystick(x_value=x,  y_value=y)
    gamepad.right_joystick(x_value=-x, y_value=0)
    gamepad.update()
    time.sleep(0.3)
    gamepad.left_joystick(x_value=0, y_value=0)
    gamepad.right_joystick(x_value=0, y_value=0)
    gamepad.update()


# ===========================================================================
# Smart UAC / Admin helpers
# ===========================================================================

def _is_admin() -> bool:
    try:
        return bool(ctypes.windll.shell32.IsUserAnAdmin())
    except Exception:
        return False


def _is_vigem_installed() -> bool:
    """Return True if the ViGEmBus kernel driver service is registered."""
    for reg_path in (
        r"SYSTEM\CurrentControlSet\Services\ViGEmBus",
        r"SYSTEM\ControlSet001\Services\ViGEmBus",
    ):
        try:
            k = winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, reg_path)
            winreg.CloseKey(k)
            return True
        except FileNotFoundError:
            continue
    return False


def _relaunch_as_admin() -> None:
    """Re-launch this process with administrator rights, then exit."""
    exe    = sys.argv[0] if getattr(sys, "frozen", False) else sys.executable
    params = " ".join(f'"{a}"' for a in sys.argv[1:])
    ctypes.windll.shell32.ShellExecuteW(None, "runas", exe, params or None, None, 1)
    sys.exit(0)


# ===========================================================================
# Auto-start with Windows
# ===========================================================================

def _get_exe_path() -> str:
    return sys.argv[0] if getattr(sys, "frozen", False) else sys.executable


def _autostart_get() -> bool:
    try:
        k = winreg.OpenKey(winreg.HKEY_CURRENT_USER, AUTOSTART_REG, 0, winreg.KEY_READ)
        winreg.QueryValueEx(k, AUTOSTART_KEY)
        winreg.CloseKey(k)
        return True
    except FileNotFoundError:
        return False


def _autostart_set(enable: bool) -> None:
    k = winreg.OpenKey(winreg.HKEY_CURRENT_USER, AUTOSTART_REG, 0, winreg.KEY_SET_VALUE)
    if enable:
        winreg.SetValueEx(k, AUTOSTART_KEY, 0, winreg.REG_SZ, f'"{_get_exe_path()}"')
    else:
        try:
            winreg.DeleteValue(k, AUTOSTART_KEY)
        except FileNotFoundError:
            pass
    winreg.CloseKey(k)


# ===========================================================================
# Application
# ===========================================================================

class CaffeineApp:
    def __init__(self):
        settings       = _load_settings()
        self.interval  = settings.get("interval_seconds", 10 * 60)

        self.running       = False
        self.paused        = False
        self.pulse_count   = 0
        self.session_start = 0.0
        self.next_pulse_at = 0.0
        self.roblox_count  = 0
        self.last_error    = ""

        self._gamepad = None
        self._icon    = None
        self._lock    = threading.Lock()

    # ── Gamepad ──────────────────────────────────────────────────────────

    def _create_gamepad(self) -> bool:
        try:
            import vgamepad as vg
            self._gamepad = vg.VX360Gamepad()
            return True
        except Exception as ex:
            self.last_error = str(ex)
            return False

    # ── Keep-alive loop ───────────────────────────────────────────────────

    def _loop(self):
        while self.running:
            if self.paused:
                time.sleep(1)
                continue

            with self._lock:
                self.roblox_count = _detect_roblox()
                try:
                    if self._gamepad:
                        _simulate_pulse(self._gamepad)
                        self.pulse_count  += 1
                        self.next_pulse_at = time.time() + self.interval
                except Exception as ex:
                    self.last_error = str(ex)

            self._refresh_icon()
            self._refresh_menu()

            deadline = time.time() + self.interval
            while time.time() < deadline and self.running and not self.paused:
                time.sleep(1)

        if self._gamepad:
            self._gamepad = None

    # ── Real-time tooltip ──────────────────────────────────────────────────

    def _tooltip_loop(self):
        while self.running:
            rem  = max(0, int(self.next_pulse_at - time.time()))
            m, s = divmod(rem, 60)
            if self.paused:
                tip = f"{APP_NAME} — Paused"
            elif self.roblox_count == 0:
                tip = f"{APP_NAME} — No Roblox detected"
            else:
                tip = f"{APP_NAME} — Next pulse in {m}m {s:02d}s"
            if self._icon:
                try:
                    self._icon.title = tip
                except Exception:
                    pass
            time.sleep(1)

    # ── Tray helpers ──────────────────────────────────────────────────────

    def _state_str(self) -> str:
        if not self.running:
            return "error"
        return "paused" if self.paused else "active"

    def _refresh_icon(self):
        if self._icon:
            try:
                self._icon.icon = _make_icon(self._state_str())
            except Exception:
                pass

    def _refresh_menu(self):
        if self._icon:
            try:
                self._icon.menu = self._build_menu()
            except Exception:
                pass

    # ── Menu ──────────────────────────────────────────────────────────────

    def _build_menu(self):
        uptime  = int(time.time() - self.session_start) if self.running else 0
        rem     = max(0, int(self.next_pulse_at - time.time()))
        m, s    = divmod(rem, 60)
        rem_str = f"{m}m {s:02d}s" if m else f"{s}s"

        state_badge = (
            "  [ PAUSED ]  " if self.paused else
            "  [ ACTIVE ]  " if self.running else
            "  [ STOPPED ] "
        )
        pause_lbl = "  > Resume" if self.paused else "  | Pause"

        autostart_on = _autostart_get()
        autostart_lbl = (
            "  [v] Start with Windows" if autostart_on else
            "  [ ] Start with Windows"
        )

        def _lbl(text):
            return Item(text, None, enabled=False)

        # ── Interval submenu ───────────────────────────────────────────
        interval_items = []
        for label, secs in INTERVAL_OPTIONS:
            tick  = "v" if self.interval == secs else " "
            ivl_s = secs  # capture for lambda
            interval_items.append(
                Item(f"  [{tick}] {label}", lambda _, s=ivl_s: self._set_interval(s))
            )
        interval_submenu = Menu(*interval_items)

        return Menu(
            # Header
            _lbl(f"  {APP_NAME} v{APP_VERSION}"),
            Menu.SEPARATOR,
            # State
            _lbl(state_badge),
            Menu.SEPARATOR,
            # Stats
            _lbl(f"  Pulses  {self.pulse_count:>6}"),
            _lbl(f"  Uptime  {_fmt_time(uptime):>6}"),
            _lbl(f"  Next in {rem_str:>6}"),
            _lbl(f"  Roblox  {self.roblox_count:>6} window(s)"),
            Menu.SEPARATOR,
            # Actions
            Item(pause_lbl,           self._toggle_pause),
            Item("  + Pulse Now",     self._pulse_now),
            Menu.SEPARATOR,
            # Interval submenu
            Item("  ~ Interval",      interval_submenu),
            # Auto-start toggle
            Item(autostart_lbl,       self._toggle_autostart),
            Menu.SEPARATOR,
            Item("  x Quit",          self._quit),
        )

    # ── Actions ───────────────────────────────────────────────────────────

    def _toggle_pause(self, icon=None, item=None):
        self.paused = not self.paused
        if not self.paused:
            self.next_pulse_at = time.time() + self.interval
        self._refresh_icon()
        self._refresh_menu()
        lbl = "Paused" if self.paused else "Resumed"
        self._notify(f"{APP_NAME} — {lbl}",
                     "Anti-AFK is now " + ("paused." if self.paused else "active."))

    def _pulse_now(self, icon=None, item=None):
        def _do():
            with self._lock:
                try:
                    if self._gamepad:
                        _simulate_pulse(self._gamepad)
                        self.pulse_count  += 1
                        self.next_pulse_at = time.time() + self.interval
                        self.roblox_count  = _detect_roblox()
                    self._refresh_icon()
                    self._refresh_menu()
                    self._notify(APP_NAME, "Manual pulse sent!")
                except Exception as ex:
                    self.last_error = str(ex)
        threading.Thread(target=_do, daemon=True).start()

    def _set_interval(self, seconds: int, icon=None, item=None):
        self.interval = seconds
        self.next_pulse_at = time.time() + seconds
        _save_settings({"interval_seconds": seconds})
        self._refresh_menu()
        self._notify(APP_NAME, f"Interval set to {seconds // 60} minutes.")

    def _toggle_autostart(self, icon=None, item=None):
        new_val = not _autostart_get()
        _autostart_set(new_val)
        self._refresh_menu()
        lbl = "enabled" if new_val else "disabled"
        self._notify(APP_NAME, f"Start with Windows {lbl}.")

    def _quit(self, icon=None, item=None):
        self.running = False
        if self._icon:
            self._icon.stop()

    def _notify(self, title: str, message: str):
        try:
            if self._icon:
                self._icon.notify(message, title)
        except Exception:
            pass

    # ── Entry point ───────────────────────────────────────────────────────

    def run(self):
        # ── #1 Single instance ────────────────────────────────────────
        _mutex = ctypes.windll.kernel32.CreateMutexW(None, False, MUTEX_NAME)
        if ctypes.windll.kernel32.GetLastError() == 183:  # ERROR_ALREADY_EXISTS
            ctypes.windll.user32.MessageBoxW(
                0,
                "Roblox Caffeine is already running!\n\n"
                "Check your system tray (bottom-right of taskbar).",
                f"{APP_NAME} — Already Running",
                0x40,  # MB_ICONINFORMATION
            )
            return

        # ── Check Python dependencies ─────────────────────────────────
        missing = []
        try:
            import vgamepad  # noqa: F401
        except ImportError:
            missing.append("vgamepad")
        try:
            import psutil    # noqa: F401
        except ImportError:
            missing.append("psutil")

        if missing:
            ctypes.windll.user32.MessageBoxW(
                0,
                f"Missing dependencies: {', '.join(missing)}\n\nRun install.bat to fix this.",
                f"{APP_NAME} — Error",
                0x10,
            )
            return

        # ── #3/#8 Smart UAC: elevate only if ViGEmBus not installed ──
        if not _is_vigem_installed() and not _is_admin():
            result = ctypes.windll.user32.MessageBoxW(
                0,
                "First-time setup required!\n\n"
                "Roblox Caffeine needs to install the ViGEmBus virtual controller driver.\n\n"
                "• This only happens ONCE — never again after this.\n"
                "• ViGEmBus is the same trusted driver used by DS4Windows and Steam Input.\n\n"
                "Click OK to allow the installation (a UAC prompt will appear).",
                f"{APP_NAME} — First Time Setup",
                0x21,  # MB_OKCANCEL | MB_ICONINFORMATION
            )
            if result == 1:
                _relaunch_as_admin()
            return

        # ── Create virtual gamepad ────────────────────────────────────
        if not self._create_gamepad():
            ctypes.windll.user32.MessageBoxW(
                0,
                f"Failed to create virtual gamepad:\n{self.last_error}\n\n"
                "Try right-clicking the exe → Run as administrator.",
                f"{APP_NAME} — Error",
                0x10,
            )
            return

        # ── Launch ────────────────────────────────────────────────────
        self.running       = True
        self.session_start = time.time()
        self.next_pulse_at = time.time()

        threading.Thread(target=self._loop,         daemon=True).start()
        threading.Thread(target=self._tooltip_loop, daemon=True).start()

        self._icon = pystray.Icon(
            "roblox_caffeine",
            icon  = _make_icon("active"),
            title = APP_NAME,
            menu  = self._build_menu(),
        )

        def _on_setup(icon):
            icon.visible      = True
            roblox            = _detect_roblox()
            self.roblox_count = roblox
            # ── #12 Detailed startup notification ────────────────────
            if roblox > 0:
                msg = f"{roblox} Roblox window(s) detected — Anti-AFK is active!"
            else:
                msg = "No Roblox running yet. Will detect automatically when you open it."
            self._notify(f"{APP_NAME} v{APP_VERSION} Active", msg)

        self._icon.run(_on_setup)


# ===========================================================================
# Entry
# ===========================================================================

def main():
    CaffeineApp().run()


if __name__ == "__main__":
    main()
