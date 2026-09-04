#![allow(static_mut_refs)]

use crate::config::{is_autostart_enabled, load_settings, save_settings, set_autostart, Settings};
use crate::detector::{count_roblox_instances, find_roblox_windows};
use crate::focus::{execute_stealth_pulse, is_user_actively_typing};
use crate::gamepad::GamepadManager;
use std::mem;
use std::ptr;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIIF_INFO, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIF_ICON, NIF_INFO, NIF_MESSAGE,
    NIF_TIP, NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CheckMenuItem, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
    DestroyWindow, DispatchMessageW, GetCursorPos, GetMessageW, KillTimer, LoadIconW,
    PostQuitMessage, RegisterClassExW, SetForegroundWindow, SetTimer, TrackPopupMenu,
    TranslateMessage, MF_BYCOMMAND, MF_CHECKED, MF_DISABLED, MF_GRAYED, MF_POPUP, MF_SEPARATOR,
    MF_STRING, MF_UNCHECKED, MSG, TPM_BOTTOMALIGN, TPM_RIGHTBUTTON, WM_COMMAND, WM_DESTROY,
    WM_RBUTTONUP, WM_TIMER, WM_USER, WNDCLASSEXW,
};

const WM_TRAYICON: u32 = WM_USER + 1;
const TIMER_ID: usize = 1;

// Menu Command IDs
const IDM_TITLE: usize = 100;
const IDM_STATUS: usize = 101;
const IDM_PULSES: usize = 102;
const IDM_ROBLOX: usize = 103;
const IDM_INTERVAL_5: usize = 201;
const IDM_INTERVAL_10: usize = 202;
const IDM_INTERVAL_15: usize = 203;
const IDM_INTERVAL_20: usize = 204;
const IDM_AUTOSTART: usize = 301;
const IDM_PAUSE: usize = 401;
const IDM_PULSE_NOW: usize = 402;
const IDM_QUIT: usize = 999;

pub struct AppState {
    pub settings: Settings,
    pub gamepad: Option<GamepadManager>,
    pub is_paused: bool,
    pub pulses_count: u64,
    pub remaining_seconds: u64,
    pub roblox_count: usize,
    pub pulse_direction_step: usize,
}

static mut GLOBAL_STATE: Option<AppState> = None;

fn copy_to_wide(dest: &mut [u16], src: &str) {
    let mut i = 0;
    for c in src.encode_utf16() {
        if i < dest.len() - 1 {
            dest[i] = c;
            i += 1;
        }
    }
    if i < dest.len() {
        dest[i] = 0;
    }
}

pub fn run_tray_app(gamepad: Option<GamepadManager>) {
    unsafe {
        let settings = load_settings();
        let remaining = settings.interval_seconds;

        GLOBAL_STATE = Some(AppState {
            settings,
            gamepad,
            is_paused: false,
            pulses_count: 0,
            remaining_seconds: remaining,
            roblox_count: count_roblox_instances(),
            pulse_direction_step: 0,
        });

        let hinstance = GetModuleHandleW(ptr::null());
        let class_name: Vec<u16> = "RobloxCaffeineRustClass\0".encode_utf16().collect();

        // Load icon ID 1 from embedded PE resource (icon.ico embedded via build.rs)
        let hicon = LoadIconW(hinstance, 1 as *const u16);

        let wnd_class = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: hicon,
            hCursor: ptr::null_mut(),
            hbrBackground: ptr::null_mut(),
            lpszMenuName: ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: hicon,
        };

        RegisterClassExW(&wnd_class);

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            class_name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            hinstance,
            ptr::null(),
        );

        if hwnd.is_null() {
            return;
        }

        // Initialize Notify Icon
        let mut nid: NOTIFYICONDATAW = mem::zeroed();
        nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_INFO;
        nid.uCallbackMessage = WM_TRAYICON;
        nid.hIcon = hicon;

        let initial_tip = format!("Roblox Caffeine — Active | Next: {}s", remaining);
        copy_to_wide(&mut nid.szTip, &initial_tip);

        // Startup Balloon Notification
        copy_to_wide(&mut nid.szInfoTitle, "Roblox Caffeine v2.1 (Rust)");
        copy_to_wide(
            &mut nid.szInfo,
            "Anti-AFK active! Keeping your Roblox windows awake with zero focus stealing.",
        );
        nid.dwInfoFlags = NIIF_INFO;

        Shell_NotifyIconW(NIM_ADD, &nid);

        // 1-second timer for countdown update & idle loop
        SetTimer(hwnd, TIMER_ID, 1000, None);

        // Standard Win32 Message Loop
        let mut msg: MSG = mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Clean up on exit
        KillTimer(hwnd, TIMER_ID);
        Shell_NotifyIconW(NIM_DELETE, &nid);
        DestroyWindow(hwnd);
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TIMER => {
            if let Some(state) = GLOBAL_STATE.as_mut() {
                // Update Roblox process count every 5 seconds
                if state.remaining_seconds % 5 == 0 {
                    state.roblox_count = count_roblox_instances();
                }

                if !state.is_paused {
                    if state.remaining_seconds > 0 {
                        state.remaining_seconds -= 1;
                    }

                    if state.remaining_seconds == 0 {
                        // Smart check: postpone pulse by 5s if user is actively typing or moving mouse
                        if is_user_actively_typing(2500) {
                            state.remaining_seconds = 5;
                        } else {
                            let hwnds = find_roblox_windows();
                            if !hwnds.is_empty() {
                                execute_stealth_pulse(&hwnds, &mut state.gamepad, state.pulse_direction_step);
                            } else if let Some(gamepad) = state.gamepad.as_mut() {
                                let _ = gamepad.pulse(state.pulse_direction_step);
                            }
                            state.pulse_direction_step += 1;
                            state.pulses_count += 1;
                            state.remaining_seconds = state.settings.interval_seconds;
                        }
                    }
                }

                // Update dynamic tooltip
                let mut nid: NOTIFYICONDATAW = mem::zeroed();
                nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
                nid.hWnd = hwnd;
                nid.uID = 1;
                nid.uFlags = NIF_TIP;

                let tooltip = if state.is_paused {
                    format!("Roblox Caffeine — [PAUSED] | {} Roblox", state.roblox_count)
                } else {
                    let m = state.remaining_seconds / 60;
                    let s = state.remaining_seconds % 60;
                    format!(
                        "Roblox Caffeine — Active | Next: {:02}m {:02}s | {} Roblox",
                        m, s, state.roblox_count
                    )
                };

                copy_to_wide(&mut nid.szTip, &tooltip);
                Shell_NotifyIconW(NIM_MODIFY, &nid);
            }
            0
        }

        WM_TRAYICON => {
            if lparam as u32 == WM_RBUTTONUP {
                show_context_menu(hwnd);
            }
            0
        }

        WM_COMMAND => {
            let cmd = (wparam & 0xFFFF) as usize;
            handle_menu_command(hwnd, cmd);
            0
        }

        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn show_context_menu(hwnd: HWND) {
    if let Some(state) = GLOBAL_STATE.as_ref() {
        let menu = CreatePopupMenu();
        let submenu_interval = CreatePopupMenu();

        let title = "Roblox Caffeine v2.1 (Rust)\0".encode_utf16().collect::<Vec<_>>();
        AppendMenuW(menu, MF_STRING | MF_DISABLED | MF_GRAYED, IDM_TITLE, title.as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null());

        let status_str = if state.is_paused {
            "Status: [PAUSED]\0"
        } else {
            "Status: [ACTIVE]\0"
        }
        .encode_utf16()
        .collect::<Vec<_>>();
        AppendMenuW(menu, MF_STRING | MF_DISABLED | MF_GRAYED, IDM_STATUS, status_str.as_ptr());

        let pulses_str = format!("Pulses Sent: {}\0", state.pulses_count)
            .encode_utf16()
            .collect::<Vec<_>>();
        AppendMenuW(menu, MF_STRING | MF_DISABLED | MF_GRAYED, IDM_PULSES, pulses_str.as_ptr());

        let roblox_str = format!("Roblox Windows: {}\0", state.roblox_count)
            .encode_utf16()
            .collect::<Vec<_>>();
        AppendMenuW(menu, MF_STRING | MF_DISABLED | MF_GRAYED, IDM_ROBLOX, roblox_str.as_ptr());

        AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null());

        // Interval Submenu
        let int_5 = "5 Minutes\0".encode_utf16().collect::<Vec<_>>();
        let int_10 = "10 Minutes\0".encode_utf16().collect::<Vec<_>>();
        let int_15 = "15 Minutes\0".encode_utf16().collect::<Vec<_>>();
        let int_20 = "20 Minutes\0".encode_utf16().collect::<Vec<_>>();

        AppendMenuW(submenu_interval, MF_STRING, IDM_INTERVAL_5, int_5.as_ptr());
        AppendMenuW(submenu_interval, MF_STRING, IDM_INTERVAL_10, int_10.as_ptr());
        AppendMenuW(submenu_interval, MF_STRING, IDM_INTERVAL_15, int_15.as_ptr());
        AppendMenuW(submenu_interval, MF_STRING, IDM_INTERVAL_20, int_20.as_ptr());

        // Check active interval
        let active_id = match state.settings.interval_seconds {
            300 => IDM_INTERVAL_5,
            600 => IDM_INTERVAL_10,
            900 => IDM_INTERVAL_15,
            1200 => IDM_INTERVAL_20,
            _ => IDM_INTERVAL_10,
        };
        CheckMenuItem(submenu_interval, active_id as u32, MF_BYCOMMAND | MF_CHECKED);

        let int_label = "Pulse Interval\0".encode_utf16().collect::<Vec<_>>();
        AppendMenuW(menu, MF_POPUP, submenu_interval as usize, int_label.as_ptr());

        // Auto-start with Windows
        let autostart_flags = if is_autostart_enabled() {
            MF_STRING | MF_CHECKED
        } else {
            MF_STRING | MF_UNCHECKED
        };
        let autostart_str = "Start with Windows\0".encode_utf16().collect::<Vec<_>>();
        AppendMenuW(menu, autostart_flags, IDM_AUTOSTART, autostart_str.as_ptr());

        AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null());

        // Pause / Resume
        let pause_label = if state.is_paused {
            "Resume Anti-AFK\0"
        } else {
            "Pause Anti-AFK\0"
        }
        .encode_utf16()
        .collect::<Vec<_>>();
        AppendMenuW(menu, MF_STRING, IDM_PAUSE, pause_label.as_ptr());

        let pulse_now_str = "Pulse Now (Manual Trigger)\0".encode_utf16().collect::<Vec<_>>();
        AppendMenuW(menu, MF_STRING, IDM_PULSE_NOW, pulse_now_str.as_ptr());

        AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null());

        let quit_str = "Quit Roblox Caffeine\0".encode_utf16().collect::<Vec<_>>();
        AppendMenuW(menu, MF_STRING, IDM_QUIT, quit_str.as_ptr());

        let mut pt: POINT = mem::zeroed();
        GetCursorPos(&mut pt);
        SetForegroundWindow(hwnd);
        TrackPopupMenu(
            menu,
            TPM_RIGHTBUTTON | TPM_BOTTOMALIGN,
            pt.x,
            pt.y,
            0,
            hwnd,
            ptr::null(),
        );
        DestroyMenu(menu);
    }
}

unsafe fn handle_menu_command(hwnd: HWND, cmd: usize) {
    if let Some(state) = GLOBAL_STATE.as_mut() {
        match cmd {
            IDM_INTERVAL_5 => change_interval(300),
            IDM_INTERVAL_10 => change_interval(600),
            IDM_INTERVAL_15 => change_interval(900),
            IDM_INTERVAL_20 => change_interval(1200),

            IDM_AUTOSTART => {
                let current = is_autostart_enabled();
                set_autostart(!current);
            }

            IDM_PAUSE => {
                state.is_paused = !state.is_paused;
            }

            IDM_PULSE_NOW => {
                let hwnds = find_roblox_windows();
                if !hwnds.is_empty() {
                    execute_stealth_pulse(&hwnds, &mut state.gamepad, state.pulse_direction_step);
                } else if let Some(gamepad) = state.gamepad.as_mut() {
                    let _ = gamepad.pulse(state.pulse_direction_step);
                }
                state.pulse_direction_step += 1;
                state.pulses_count += 1;
                state.remaining_seconds = state.settings.interval_seconds;
            }

            IDM_QUIT => {
                let mut nid: NOTIFYICONDATAW = mem::zeroed();
                nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
                nid.hWnd = hwnd;
                nid.uID = 1;
                Shell_NotifyIconW(NIM_DELETE, &nid);
                PostQuitMessage(0);
            }

            _ => {}
        }
    }
}

fn change_interval(seconds: u64) {
    unsafe {
        if let Some(state) = GLOBAL_STATE.as_mut() {
            state.settings.interval_seconds = seconds;
            state.remaining_seconds = seconds;
            save_settings(&state.settings);
        }
    }
}
