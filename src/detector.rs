use std::mem;
use windows_sys::Win32::Foundation::{CloseHandle, HWND, INVALID_HANDLE_VALUE, LPARAM};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::StationsAndDesktops::{
    CloseDesktop, EnumDesktopWindows, OpenInputDesktop, DESKTOP_ENUMERATE,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
};

type BOOL = i32;

const ROBLOX_TARGETS: &[&str] = &[
    "robloxplayerbeta.exe",
    "robloxplayer.exe",
    "windows10universal.exe",
];

pub fn get_roblox_pids() -> Vec<u32> {
    let mut pids = Vec::new();
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return pids;
        }

        let mut entry: PROCESSENTRY32W = mem::zeroed();
        entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let len = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                let exe_name = String::from_utf16_lossy(&entry.szExeFile[..len]).to_lowercase();

                if ROBLOX_TARGETS.iter().any(|&target| exe_name == target) {
                    pids.push(entry.th32ProcessID);
                }

                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
    }
    pids
}

pub fn count_roblox_instances() -> usize {
    get_roblox_pids().len()
}

struct EnumContext {
    pids: Vec<u32>,
    hwnds: Vec<HWND>,
}

unsafe extern "system" fn enum_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let ctx = &mut *(lparam as *mut EnumContext);
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, &mut pid);

    if ctx.pids.contains(&pid) {
        let len = GetWindowTextLengthW(hwnd);
        if len > 0 {
            let mut buf = vec![0u16; (len + 1) as usize];
            GetWindowTextW(hwnd, buf.as_mut_ptr(), len + 1);
            let title = String::from_utf16_lossy(&buf[..len as usize]);
            let title_lower = title.to_lowercase();

            // Match main Roblox game window, exclude helper/IME/GDI windows
            if title_lower.starts_with("roblox") && !title_lower.contains("gdi+") {
                if !ctx.hwnds.contains(&hwnd) {
                    ctx.hwnds.push(hwnd);
                }
            }
        }
    }
    1
}

pub fn find_roblox_windows() -> Vec<HWND> {
    let pids = get_roblox_pids();
    if pids.is_empty() {
        return Vec::new();
    }

    let mut ctx = EnumContext {
        pids,
        hwnds: Vec::new(),
    };

    unsafe {
        // First try OpenInputDesktop (works in all desktop station contexts)
        let desk = OpenInputDesktop(0, 0, DESKTOP_ENUMERATE);
        if !desk.is_null() {
            EnumDesktopWindows(desk, Some(enum_cb), &mut ctx as *mut _ as LPARAM);
            CloseDesktop(desk);
        }

        // Also run standard EnumWindows if no windows found yet
        if ctx.hwnds.is_empty() {
            EnumWindows(Some(enum_cb), &mut ctx as *mut _ as LPARAM);
        }
    }

    ctx.hwnds
}
