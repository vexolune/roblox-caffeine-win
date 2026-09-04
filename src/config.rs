use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::ptr;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ,
};

const RUN_SUBKEY: windows_sys::core::PCWSTR = windows_sys::w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const APP_KEY_NAME: windows_sys::core::PCWSTR = windows_sys::w!("RobloxCaffeine");

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub interval_seconds: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            interval_seconds: 600, // 10 minutes
        }
    }
}

fn get_settings_path() -> Option<PathBuf> {
    env::var_os("APPDATA").map(|appdata| {
        let mut path = PathBuf::from(appdata);
        path.push("RobloxCaffeine");
        path.push("settings.json");
        path
    })
}

pub fn load_settings() -> Settings {
    if let Some(path) = get_settings_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str(&content) {
                    return settings;
                }
            }
        }
    }
    Settings::default()
}

pub fn save_settings(settings: &Settings) {
    if let Some(path) = get_settings_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(settings) {
            let _ = fs::write(&path, json);
        }
    }
}

pub fn is_autostart_enabled() -> bool {
    unsafe {
        let mut hkey: HKEY = ptr::null_mut();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            RUN_SUBKEY,
            0,
            KEY_QUERY_VALUE,
            &mut hkey,
        ) != 0
        {
            return false;
        }

        let mut data_type: u32 = 0;
        let mut data_len: u32 = 0;
        let status = RegQueryValueExW(
            hkey,
            APP_KEY_NAME,
            ptr::null(),
            &mut data_type,
            ptr::null_mut(),
            &mut data_len,
        );

        RegCloseKey(hkey);
        status == 0
    }
}

pub fn set_autostart(enable: bool) -> bool {
    unsafe {
        let mut hkey: HKEY = ptr::null_mut();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            RUN_SUBKEY,
            0,
            KEY_SET_VALUE,
            &mut hkey,
        ) != 0
        {
            return false;
        }

        let success = if enable {
            if let Ok(current_exe) = env::current_exe() {
                let path_str = current_exe.to_string_lossy();
                let wide: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();
                let size_bytes = (wide.len() * 2) as u32;

                RegSetValueExW(
                    hkey,
                    APP_KEY_NAME,
                    0,
                    REG_SZ,
                    wide.as_ptr() as *const u8,
                    size_bytes,
                ) == 0
            } else {
                false
            }
        } else {
            RegDeleteValueW(hkey, APP_KEY_NAME) == 0
        };

        RegCloseKey(hkey);
        success
    }
}
