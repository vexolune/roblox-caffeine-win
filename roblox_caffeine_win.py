#!/usr/bin/env python3
"""
Roblox Caffeine (Windows) - Minimalist Keep-Alive Utility for Windows
======================================================================
Periodically nudges a virtual Xbox 360 thumbstick every 10 minutes via
ViGEmBus to prevent Roblox's 20-minute inactivity kick.

Key properties:
  - Zero focus stealing  : never touches your mouse or keyboard
  - No window targeting  : XInput is a system-wide broadcast — all open
                           Roblox windows receive the input simultaneously
  - No process injection : uses a signed kernel driver (ViGEmBus), same
                           as DS4Windows / Steam Input
  - Multi-window         : one virtual controller keeps every Roblox
                           instance alive at the same time
"""

import sys
import time
import random
from datetime import datetime, timedelta

# ==============================================================================
# Terminal Styling — ANSI codes (Windows Terminal / PowerShell 7+ / cmd with VT)
# ==============================================================================
GRAY   = '\033[90m'
CYAN   = '\033[96m'
BLUE   = '\033[94m'
YELLOW = '\033[93m'
GREEN  = '\033[92m'
RED    = '\033[91m'
BOLD   = '\033[1m'
DIM    = '\033[2m'
RESET  = '\033[0m'
CLEAR  = '\r\033[K'
WIDTH  = 50

SPINNER          = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏']
STICK_DEFLECTION = 22000          # ±22 000 of ±32 767 — same as Linux version
INTERVAL_SECONDS = 10 * 60        # Fixed 10-minute interval

# Roblox process names across different install methods
ROBLOX_PROCESS_NAMES = {
    'robloxplayerbeta.exe',
    'robloxplayer.exe',
    'windows10universal.exe',   # Microsoft Store version
}

BANNER_ART = r"""
 ___  ___  ___ _    _____  __   ___   _   ___ ___ ___ ___ _  _ ___ 
| _ \/ _ \| _ ) |  / _ \ \/  / __| /_\ | __| __| __|_ _| \| | __|
|   / (_) | _ \ |_| (_) >  <  | (__ / _ \| _|| _|| _| | || .` | _| 
|_|_\\___/|___/____\___/_/\_\  \___/_/ \_\_| |_| |___|___|_|\_|___|
"""

BANNER_SUBTITLE = "         Windows Edition — Keep your Roblox windows awake"


# ==============================================================================
# Helpers
# ==============================================================================

def _enable_ansi_windows():
    """Enable VT100 ANSI escape codes on older Windows consoles."""
    if sys.platform != "win32":
        return
    try:
        import ctypes
        kernel32 = ctypes.windll.kernel32
        # ENABLE_VIRTUAL_TERMINAL_PROCESSING = 0x0004
        kernel32.SetConsoleMode(kernel32.GetStdHandle(-11), 7)
    except Exception:
        pass  # Non-fatal — colours just won't render in very old consoles


def format_time(seconds: int) -> str:
    """Format a duration in seconds to 'Xh YYm' or 'Ym'."""
    h, m = divmod(seconds // 60, 60)
    return f"{h}h {m:02d}m" if h else f"{m}m"


def print_banner():
    """Print the ASCII art banner and subtitle."""
    _enable_ansi_windows()
    print(f"\n{YELLOW}{BOLD}{BANNER_ART.strip()}{RESET}")
    print(f"{DIM}{BANNER_SUBTITLE}{RESET}\n")


def render_box(title_icon: str, title_text: str, key_val_pairs: list):
    """Render a fastfetch-style bordered status box."""
    print(f"{GRAY}┌{'─' * WIDTH}┐{RESET}")
    print(f"    {title_icon} : {title_text}")
    for icon, val in key_val_pairs:
        print(f"    {icon} : {val}")
    print(f"{GRAY}└{'─' * WIDTH}┘{RESET}")


# ==============================================================================
# Dependency Check
# ==============================================================================

def check_dependencies() -> bool:
    """
    Verify that vgamepad and psutil are importable.
    Provides actionable error messages if not.
    """
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
        render_box(
            f"{RED}✕{RESET}", f"{BOLD}Missing Dependencies{RESET}",
            [
                (f"{YELLOW}!{RESET} ", f"Run:  pip install {' '.join(missing)}"),
                (f"{YELLOW}!{RESET} ", "Or double-click  install.bat"),
                (f"{CYAN}ℹ{RESET} ", "vgamepad installs ViGEmBus driver (UAC prompt)"),
                (f"{CYAN}ℹ{RESET} ", "ViGEmBus is the same driver used by DS4Windows"),
            ]
        )
        return False
    return True


# ==============================================================================
# Roblox Window Detection
# ==============================================================================

def detect_roblox_instances() -> list:
    """
    Enumerate all running Roblox processes and attempt to collect their
    window titles. Returns a list of dicts: {pid, name, title}.
    """
    try:
        import psutil
    except ImportError:
        return []

    found = []
    for proc in psutil.process_iter(['pid', 'name']):
        try:
            pname = (proc.info.get('name') or '').lower()
            if pname in ROBLOX_PROCESS_NAMES:
                found.append({
                    'pid':   proc.info['pid'],
                    'name':  proc.info['name'],
                    'title': '',
                })
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            pass

    # Attempt to enrich with window titles via pywin32 (optional but nice)
    try:
        import win32gui
        import win32process

        def _enum_cb(hwnd, ctx):
            if not win32gui.IsWindowVisible(hwnd):
                return
            _, pid = win32process.GetWindowThreadProcessId(hwnd)
            for entry in ctx:
                if entry['pid'] == pid and not entry['title']:
                    title = win32gui.GetWindowText(hwnd)
                    if title:
                        entry['title'] = title

        win32gui.EnumWindows(_enum_cb, found)
    except Exception:
        pass  # pywin32 not available — titles just show as process name

    return found


# ==============================================================================
# Keep-Alive Pulse
# ==============================================================================

def simulate_keep_alive(gamepad, count: int, windows: list):
    """
    Nudge left & right thumbsticks for 0.3 s then re-centre.
    Mirrors the Linux version's axis deflection approach exactly.

    Why 0.3 s deflection then re-centre?
      - Roblox registers an input-change event → idle timer resets.
      - Re-centering prevents the avatar from walking indefinitely.
      - Same technique used by the Linux /dev/uinput version.
    """
    now = datetime.now()
    x   = random.choice([-STICK_DEFLECTION, STICK_DEFLECTION])
    y   = random.choice([-STICK_DEFLECTION, STICK_DEFLECTION])

    # --- Deflect both sticks ---
    gamepad.left_joystick(x_value=x, y_value=y)
    gamepad.right_joystick(x_value=-x, y_value=0)
    gamepad.update()
    time.sleep(0.3)

    # --- Re-centre cleanly ---
    gamepad.left_joystick(x_value=0, y_value=0)
    gamepad.right_joystick(x_value=0, y_value=0)
    gamepad.update()

    # --- Build status display ---
    dir_name = ("↗ UP-RIGHT"   if x > 0 and y > 0 else
                "↖ UP-LEFT"    if x < 0 and y > 0 else
                "↘ DOWN-RIGHT" if x > 0              else
                "↙ DOWN-LEFT")

    win_count = len(windows)
    if win_count == 0:
        win_label = f"{YELLOW}⚠ No Roblox instances detected{RESET}"
    elif win_count == 1:
        win_label = f"{GREEN}✔ 1 instance detected{RESET}"
    else:
        win_label = f"{GREEN}✔ {win_count} instances detected{RESET}"

    time_now  = now.strftime('%I:%M %p')
    next_time = (now + timedelta(seconds=INTERVAL_SECONDS)).strftime('%I:%M %p')

    pairs = [
        (f"{BLUE}🎮{RESET}", f"Motion: {dir_name}"),
        (f"{CYAN}⏱ {RESET}", f"Next: {next_time} (in 10m)"),
        (f"{GREEN}🪟{RESET}", win_label),
    ]

    # List individual window titles (up to 5)
    for i, w in enumerate(windows[:5], 1):
        title = (w.get('title') or w.get('name') or 'Roblox')[:44]
        pairs.append((f"{DIM}  #{i}{RESET}", f"{DIM}{title}{RESET}"))
    if win_count > 5:
        pairs.append((f"{DIM}   {RESET}", f"{DIM}… and {win_count - 5} more{RESET}"))

    sys.stdout.write(CLEAR)
    render_box(
        f"{YELLOW}☕{RESET}",
        f"Caffeine Active #{count:03d} [{time_now}]",
        pairs,
    )


# ==============================================================================
# Countdown Progress Bar
# ==============================================================================

def wait_with_progress(session_start: float):
    """
    Display a live single-line animated countdown bar during the 10-minute
    wait interval. Degrades gracefully when stdout is not a TTY (e.g. piped).
    """
    if not sys.stdout.isatty():
        time.sleep(INTERVAL_SECONDS)
        return

    for tick in range(INTERVAL_SECONDS):
        rem    = INTERVAL_SECONDS - tick
        bar_len = 12
        filled  = int((tick / INTERVAL_SECONDS) * bar_len)
        bar     = '█' * filled + '░' * (bar_len - filled)

        spin       = SPINNER[tick % len(SPINNER)]
        rem_m      = (rem + 59) // 60
        rem_str    = f"~{rem_m}m" if rem_m > 1 else "<1m"
        uptime_str = format_time(int(time.time() - session_start))

        sys.stdout.write(
            f"{CLEAR} {CYAN}{spin}{RESET} Next in {rem_str} "
            f"[{GREEN}{bar}{RESET}] • Uptime: {uptime_str} "
        )
        sys.stdout.flush()
        time.sleep(1)

    sys.stdout.write(CLEAR)
    sys.stdout.flush()


# ==============================================================================
# Entry Point
# ==============================================================================

def main():
    # ── Help ──────────────────────────────────────────────────────────────────
    if '-h' in sys.argv or '--help' in sys.argv:
        print_banner()
        print("Usage: python roblox_caffeine_win.py [OPTIONS]\n")
        print("Options:")
        print("  -h, --help    Show this help message and exit\n")
        print("How it works:")
        print("  Creates a virtual Xbox 360 controller via the ViGEmBus kernel driver.")
        print("  Every 10 minutes it nudges the thumbstick for 0.3 s to reset Roblox's")
        print("  idle timer, then re-centres. No focus stealing — XInput is a system-wide")
        print("  broadcast, so ALL open Roblox windows are kept alive simultaneously.\n")
        print("Requirements:")
        print("  pip install vgamepad psutil pywin32")
        print("  (vgamepad auto-installs the ViGEmBus driver — one-time UAC prompt)\n")
        return

    # ── Startup ───────────────────────────────────────────────────────────────
    print_banner()

    if not check_dependencies():
        return

    import vgamepad as vg

    render_box(
        f"{YELLOW}☕{RESET}", f"{BOLD}Roblox Caffeine (Windows){RESET}",
        [
            (f"{BLUE}🎮{RESET}", "Virtual Xbox 360 Gamepad (ViGEmBus driver)"),
            (f"{CYAN}⏱ {RESET}", "Interval: Every 10 minutes"),
            (f"{GREEN}🪟{RESET}", "Multi-window: all Roblox instances at once"),
            (f"{DIM}ℹ {RESET}", f"{DIM}No focus stealing  •  No mouse/keyboard input{RESET}"),
            (f"{DIM}ℹ {RESET}", f"{DIM}Press Ctrl+C to stop{RESET}"),
        ]
    )
    print()

    # ── Create virtual gamepad ─────────────────────────────────────────────
    try:
        gamepad = vg.VX360Gamepad()
    except Exception as ex:
        render_box(
            f"{RED}✕{RESET}", f"{BOLD}Failed to create virtual gamepad{RESET}",
            [
                (f"{YELLOW}!{RESET} ", f"Error: {ex}"),
                (f"{CYAN}ℹ{RESET} ", "ViGEmBus driver may need a UAC prompt to install."),
                (f"{CYAN}ℹ{RESET} ", "Run install.bat then try again."),
                (f"{CYAN}ℹ{RESET} ", "If prompted, try running as Administrator once."),
            ]
        )
        return

    # ── Main loop ─────────────────────────────────────────────────────────
    session_start = time.time()
    pulse_count   = 1

    try:
        windows = detect_roblox_instances()
        simulate_keep_alive(gamepad, pulse_count, windows)

        while True:
            wait_with_progress(session_start)
            pulse_count += 1
            windows = detect_roblox_instances()
            simulate_keep_alive(gamepad, pulse_count, windows)

    except KeyboardInterrupt:
        uptime_str = format_time(int(time.time() - session_start))
        sys.stdout.write(CLEAR)
        render_box(
            f"{YELLOW}☕{RESET}", f"{BOLD}Roblox Caffeine Stopped{RESET}",
            [
                (f"{BLUE}🎮{RESET}", "Virtual gamepad released"),
                (f"{CYAN}⏱ {RESET}", f"Session: {uptime_str} ({pulse_count} active events)"),
            ]
        )
        print()


if __name__ == "__main__":
    main()
