use crate::gamepad::GamepadManager;
use std::mem;
use std::ptr;
use std::thread;
use std::time::Duration;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::SystemInformation::GetTickCount64;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FlashWindowEx, GetForegroundWindow, GetWindowLongW, GetWindowThreadProcessId,
    SetForegroundWindow, SetLayeredWindowAttributes, SetWindowLongW, SetWindowPos,
    SystemParametersInfoW, FLASHWINFO, FLASHW_STOP, GWL_EXSTYLE, HWND_BOTTOM, LWA_ALPHA,
    SPI_GETFOREGROUNDLOCKTIMEOUT, SPI_SETFOREGROUNDLOCKTIMEOUT, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOSIZE, WS_EX_LAYERED,
};

#[link(name = "user32")]
extern "system" {
    fn AttachThreadInput(idattach: u32, idattachto: u32, fattach: i32) -> i32;
}

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

/// Stop any taskbar button flashing on the given window
fn cancel_window_flash(hwnd: HWND) {
    unsafe {
        let mut fwi: FLASHWINFO = mem::zeroed();
        fwi.cbSize = mem::size_of::<FLASHWINFO>() as u32;
        fwi.hwnd = hwnd;
        fwi.dwFlags = FLASHW_STOP;
        FlashWindowEx(&fwi);
    }
}

/// Perform 100% Silent Stealth Micro-Focus on target Roblox windows using Right Thumbstick (Camera Pan)
/// Zero taskbar popups, zero screen flicker, zero taskbar flashing, zero mouse/keyboard interference
pub fn execute_stealth_pulse(
    hwnds: &[HWND],
    gamepad: &mut Option<GamepadManager>,
    pulse_step: usize,
) -> bool {
    unsafe {
        let old_foreground = GetForegroundWindow();
        let cur_thread = GetCurrentThreadId();

        // 1. Temporarily disable Windows Foreground Lock Timeout (prevents Windows from flashing taskbar button)
        let mut old_timeout: u32 = 0;
        SystemParametersInfoW(
            SPI_GETFOREGROUNDLOCKTIMEOUT,
            0,
            &mut old_timeout as *mut _ as *mut _,
            0,
        );
        SystemParametersInfoW(SPI_SETFOREGROUNDLOCKTIMEOUT, 0, ptr::null_mut(), 0);

        // 2. Attach thread input to current foreground thread if any
        let fg_thread = if !old_foreground.is_null() {
            GetWindowThreadProcessId(old_foreground, ptr::null_mut())
        } else {
            0
        };

        if fg_thread != 0 && fg_thread != cur_thread {
            AttachThreadInput(cur_thread, fg_thread, 1);
        }

        for &hwnd in hwnds {
            let target_thread = GetWindowThreadProcessId(hwnd, ptr::null_mut());
            if target_thread != 0 && target_thread != cur_thread {
                AttachThreadInput(cur_thread, target_thread, 1);
            }

            // 3. Make window 100% transparent before focusing (zero screen flash/flicker)
            let orig_ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            SetWindowLongW(hwnd, GWL_EXSTYLE, orig_ex_style | WS_EX_LAYERED as i32);
            SetLayeredWindowAttributes(hwnd, 0, 0, LWA_ALPHA);

            // 4. Silent instant focus (NO ShowWindow SW_RESTORE to prevent auto-hide taskbar from unhiding!)
            SetForegroundWindow(hwnd);
            thread::sleep(Duration::from_millis(20));

            // 5. Send Right Thumbstick Pulse (rotates camera, character stays in place)
            if let Some(gp) = gamepad.as_mut() {
                let _ = gp.pulse(pulse_step);
            }

            thread::sleep(Duration::from_millis(20));

            // 6. Stop any taskbar flash and restore window position & transparency
            cancel_window_flash(hwnd);
            SetWindowLongW(hwnd, GWL_EXSTYLE, orig_ex_style);
            SetWindowPos(
                hwnd,
                HWND_BOTTOM,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );

            if target_thread != 0 && target_thread != cur_thread {
                AttachThreadInput(cur_thread, target_thread, 0);
            }
        }

        // 7. Instantly restore original active window
        if !old_foreground.is_null() {
            SetForegroundWindow(old_foreground);
        }

        if fg_thread != 0 && fg_thread != cur_thread {
            AttachThreadInput(cur_thread, fg_thread, 0);
        }

        // 8. Restore Windows Foreground Lock Timeout
        SystemParametersInfoW(
            SPI_SETFOREGROUNDLOCKTIMEOUT,
            0,
            old_timeout as usize as *mut _,
            0,
        );
    }
    true
}
