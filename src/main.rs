#![windows_subsystem = "windows"]

mod config;
mod detector;
mod gamepad;
mod tray;

use gamepad::GamepadManager;
use std::ptr;
use tray::run_tray_app;
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS};
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_ICONINFORMATION, MB_OK};

fn main() {
    unsafe {
        // 1. Single Instance Check via Windows Named Mutex
        let mutex_name: Vec<u16> = "Global\\RobloxCaffeineMutex_2_0\0"
            .encode_utf16()
            .collect();
        
        let mutex = CreateMutexW(ptr::null(), 0, mutex_name.as_ptr());
        if mutex.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
            let title: Vec<u16> = "Roblox Caffeine\0".encode_utf16().collect();
            let msg: Vec<u16> = "Roblox Caffeine is already running in your system tray.\nLook for the coffee cup icon (☕) in the bottom-right corner.\0"
                .encode_utf16()
                .collect();
            MessageBoxW(ptr::null_mut(), msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONINFORMATION);
            return;
        }

        // 2. Initialize Virtual Gamepad (ViGEmBus)
        let gamepad = match GamepadManager::new() {
            Ok(manager) => Some(manager),
            Err(err) => {
                let title: Vec<u16> = "Roblox Caffeine — Driver Required\0"
                    .encode_utf16()
                    .collect();
                let msg_text = format!(
                    "Failed to connect to ViGEmBus virtual controller:\n\n{}\n\nPlease run Setup.bat once as Administrator to install the ViGEmBus driver.",
                    err
                );
                let msg: Vec<u16> = msg_text
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();
                MessageBoxW(ptr::null_mut(), msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR);
                None
            }
        };

        // 3. Run Native Win32 Tray Loop
        run_tray_app(gamepad);

        // Clean up mutex
        if !mutex.is_null() {
            CloseHandle(mutex);
        }
    }
}
