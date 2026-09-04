use crate::gamepad::GamepadManager;
use std::mem;
use std::thread;
use std::time::Duration;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::SystemInformation::GetTickCount64;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowLongW, IsIconic, SetForegroundWindow, SetLayeredWindowAttributes,
    SetWindowLongW, SetWindowPos, ShowWindow, GWL_EXSTYLE, HWND_BOTTOM, LWA_ALPHA, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE, SW_MINIMIZE, SW_RESTORE, WS_EX_LAYERED,
};

/// Check if user has actively interacted (typed or moved mouse) within `threshold_ms`
pub fn is_user_actively_typing(threshold_ms: u64) -> bool {
    unsafe {
        let mut lii: LASTINPUTINFO = mem::zeroed();
        lii.cbSize = mem::size_of::<LASTINPUTINFO>() as u32;
        if GetLastInputInfo(&mut lii) != 0 {
            let now = GetTickCount64();
            let idle = now.saturating_sub(lii.dwTime as u64);
            return idle < threshold_ms;
        }
        false
    }
}

/// Perform Stealth Micro-Focus on target Roblox windows using Right Thumbstick (Camera Pan)
pub fn execute_stealth_pulse(
    hwnds: &[HWND],
    gamepad: &mut Option<GamepadManager>,
    pulse_step: usize,
) -> bool {
    unsafe {
        let old_foreground = GetForegroundWindow();

        for &hwnd in hwnds {
            let is_minimized = IsIconic(hwnd) != 0;

            // 1. Make window 100% transparent before focusing (zero screen flash/flicker)
            let orig_ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            SetWindowLongW(hwnd, GWL_EXSTYLE, orig_ex_style | WS_EX_LAYERED as i32);
            SetLayeredWindowAttributes(hwnd, 0, 0, LWA_ALPHA);

            // 2. Micro-focus the Roblox window (restore invisibly if minimized)
            if is_minimized {
                ShowWindow(hwnd, SW_RESTORE);
            }
            SetForegroundWindow(hwnd);
            thread::sleep(Duration::from_millis(20));

            // 3. Send Right Thumbstick Pulse (gentle camera pan, zero character movement, no keyboard keys)
            if let Some(gp) = gamepad.as_mut() {
                let _ = gp.pulse(pulse_step);
            }

            thread::sleep(Duration::from_millis(20));

            // 4. If it was minimized, re-minimize; otherwise push to bottom
            if is_minimized {
                ShowWindow(hwnd, SW_MINIMIZE);
            } else {
                SetWindowPos(
                    hwnd,
                    HWND_BOTTOM,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }

            // 5. Restore original window style
            SetWindowLongW(hwnd, GWL_EXSTYLE, orig_ex_style);
        }

        // 6. Instantly restore original foreground window
        if !old_foreground.is_null() {
            SetForegroundWindow(old_foreground);
        }
    }
    true
}
