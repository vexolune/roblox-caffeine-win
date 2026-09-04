use std::env;
use std::mem;
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::ptr;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::System::Threading::{WaitForSingleObject, INFINITE};
use windows_sys::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, IDYES, MB_ICONERROR, MB_ICONQUESTION, MB_OK, MB_YESNO, SW_SHOWNORMAL,
};

#[link(name = "urlmon")]
extern "system" {
    fn URLDownloadToFileW(
        pcaller: *mut std::ffi::c_void,
        szurl: *const u16,
        szfilename: *const u16,
        dwreserved: u32,
        lpfnstatuscallback: *mut std::ffi::c_void,
    ) -> i32;
}

const VIGEM_INSTALLER_URL: &str =
    "https://github.com/nefarius/ViGEmBus/releases/download/v1.22.0/ViGEmBus_1.22.0_x64_x86_arm64.exe";

pub fn prompt_and_install_vigembus() -> bool {
    unsafe {
        let title: Vec<u16> = "Roblox Caffeine — Driver Required\0".encode_utf16().collect();
        let prompt_msg: Vec<u16> = concat!(
            "Roblox Caffeine requires the ViGEmBus virtual controller driver to prevent AFK kicks:\n\n",
            "  • Runs silently in background (never steals mouse or keyboard)\n",
            "  • Works on all open Roblox windows simultaneously\n",
            "  • Safe & legitimate (standard driver used by Steam & DS4Windows)\n\n",
            "Would you like to automatically download and install it now?\n",
            "(A standard Windows Administrator prompt will appear)\0"
        )
        .encode_utf16()
        .collect();

        let choice = MessageBoxW(
            ptr::null_mut(),
            prompt_msg.as_ptr(),
            title.as_ptr(),
            MB_YESNO | MB_ICONQUESTION,
        );

        if choice != IDYES {
            return false;
        }

        // Destination in %TEMP%
        let temp_dir = env::temp_dir();
        let installer_path = temp_dir.join("ViGEmBus_Setup.exe");
        let installer_path_str = installer_path.to_string_lossy();

        let url_wide: Vec<u16> = VIGEM_INSTALLER_URL
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let dest_wide: Vec<u16> = installer_path_str
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        // 1. Download official ViGEmBus installer using Win32 URLDownloadToFileW
        let hr = URLDownloadToFileW(
            ptr::null_mut(),
            url_wide.as_ptr(),
            dest_wide.as_ptr(),
            0,
            ptr::null_mut(),
        );

        let mut download_ok =
            hr == 0 && installer_path.metadata().map(|m| m.len() > 1_000_000).unwrap_or(false);

        // Fallback: Windows built-in curl.exe if URLDownloadToFileW failed
        if !download_ok {
            let _ = Command::new("curl.exe")
                .args(["-L", "-f", "-s", "-S", "-o", &installer_path_str, VIGEM_INSTALLER_URL])
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .status();
            download_ok =
                installer_path.metadata().map(|m| m.len() > 1_000_000).unwrap_or(false);
        }

        if !download_ok {
            let err_msg: Vec<u16> = format!(
                "Failed to download ViGEmBus installer.\n\nPlease check your internet connection or download manually from:\n{}\0",
                VIGEM_INSTALLER_URL
            )
            .encode_utf16()
            .collect();
            MessageBoxW(ptr::null_mut(), err_msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR);
            return false;
        }

        // 2. Launch installer with Administrator elevation (runas) and /passive flag
        let verb: Vec<u16> = "runas\0".encode_utf16().collect();
        let params: Vec<u16> = "/passive\0".encode_utf16().collect();

        let mut sei: SHELLEXECUTEINFOW = mem::zeroed();
        sei.cbSize = mem::size_of::<SHELLEXECUTEINFOW>() as u32;
        sei.fMask = SEE_MASK_NOCLOSEPROCESS;
        sei.hwnd = ptr::null_mut();
        sei.lpVerb = verb.as_ptr();
        sei.lpFile = dest_wide.as_ptr();
        sei.lpParameters = params.as_ptr();
        sei.nShow = SW_SHOWNORMAL;

        if ShellExecuteExW(&mut sei) == 0 || sei.hProcess.is_null() {
            let err_msg: Vec<u16> =
                "Installation was cancelled or Administrator permission was denied.\0"
                    .encode_utf16()
                    .collect();
            MessageBoxW(ptr::null_mut(), err_msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR);
            let _ = std::fs::remove_file(&installer_path);
            return false;
        }

        // Wait for installer to complete
        WaitForSingleObject(sei.hProcess, INFINITE);
        CloseHandle(sei.hProcess);

        // Delete temporary installer file
        let _ = std::fs::remove_file(&installer_path);

        true
    }
}
