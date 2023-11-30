use std::path::Path;

use log::warn;
use serde::Serialize;
use serde_json::Value;
use windows::core::{w, HSTRING, PCWSTR};
use windows::Win32::System::WindowsProgramming::{
    GetPrivateProfileIntW, WritePrivateProfileStringW,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    hd_version: bool,
    fullscreen: bool,
    fullscreen_w: i32,
    fullscreen_h: i32,
    window_w: i32,
    window_h: i32,
    sound: u8,
    sound_unfocused: u8,
    sound_minimized: u8,
}

pub fn get_settings(path: &Path) -> Settings {
    let ini_file = HSTRING::from(path.join("mhf.ini").as_os_str());
    let ini_file = PCWSTR(ini_file.as_ptr());
    unsafe {
        Settings {
            hd_version: GetPrivateProfileIntW(w!("VIDEO"), w!("GRAPHICS_VER"), 1, ini_file) > 0,
            fullscreen: GetPrivateProfileIntW(w!("SCREEN"), w!("FULLSCREEN_MODE"), 1, ini_file) > 0,
            fullscreen_w: GetPrivateProfileIntW(
                w!("SCREEN"),
                w!("FULLSCREEN_RESOLUTION_W"),
                1920,
                ini_file,
            ),
            fullscreen_h: GetPrivateProfileIntW(
                w!("SCREEN"),
                w!("FULLSCREEN_RESOLUTION_H"),
                1080,
                ini_file,
            ),
            window_w: GetPrivateProfileIntW(
                w!("SCREEN"),
                w!("WINDOW_RESOLUTION_W"),
                1920,
                ini_file,
            ),
            window_h: GetPrivateProfileIntW(
                w!("SCREEN"),
                w!("WINDOW_RESOLUTION_H"),
                1080,
                ini_file,
            ),
            sound: GetPrivateProfileIntW(w!("SOUND"), w!("SOUND_VOLUME"), 0, ini_file) as u8,
            sound_unfocused: GetPrivateProfileIntW(
                w!("SOUND"),
                w!("SOUND_VOLUME_INACTIVITY"),
                0,
                ini_file,
            ) as u8,
            sound_minimized: GetPrivateProfileIntW(
                w!("SOUND"),
                w!("SOUND_VOLUME_MINIMIZE"),
                0,
                ini_file,
            ) as u8,
        }
    }
}

const FALSE_VALUE: PCWSTR = w!("0");
const TRUE_VALUE: PCWSTR = w!("1");
fn w_bool(value: bool) -> PCWSTR {
    if value {
        TRUE_VALUE
    } else {
        FALSE_VALUE
    }
}

fn w_string(value: String) -> PCWSTR {
    PCWSTR(HSTRING::from(value).as_ptr())
}

pub fn set_setting(path: &Path, name: &str, value: Value) -> Result<(), String> {
    let ini_file = HSTRING::from(path.join("mhf.ini").as_os_str());
    println!("INI FILE: {}", ini_file);
    let ini_file = PCWSTR(ini_file.as_ptr());
    unsafe {
        match (name, value) {
            ("hdVersion", Value::Bool(v)) => {
                WritePrivateProfileStringW(w!("VIDEO"), w!("GRAPHICS_VER"), w_bool(v), ini_file)
            }
            ("fullscreen", Value::Bool(v)) => {
                WritePrivateProfileStringW(w!("SCREEN"), w!("FULLSCREEN_MODE"), w_bool(v), ini_file)
            }
            ("fullscreenW", Value::Number(n)) => WritePrivateProfileStringW(
                w!("SCREEN"),
                w!("FULLSCREEN_RESOLUTION_W"),
                w_string(n.to_string()),
                ini_file,
            ),
            ("fullscreenH", Value::Number(n)) => WritePrivateProfileStringW(
                w!("SCREEN"),
                w!("FULLSCREEN_RESOLUTION_H"),
                w_string(n.to_string()),
                ini_file,
            ),
            ("windowW", Value::Number(n)) => WritePrivateProfileStringW(
                w!("SCREEN"),
                w!("WINDOW_RESOLUTION_W"),
                w_string(n.to_string()),
                ini_file,
            ),
            ("windowH", Value::Number(n)) => WritePrivateProfileStringW(
                w!("SCREEN"),
                w!("WINDOW_RESOLUTION_H"),
                w_string(n.to_string()),
                ini_file,
            ),
            ("sound", Value::Number(n)) => WritePrivateProfileStringW(
                w!("SOUND"),
                w!("SOUND_VOLUME"),
                w_string(n.to_string()),
                ini_file,
            ),
            ("soundUnfocused", Value::Number(n)) => WritePrivateProfileStringW(
                w!("SOUND"),
                w!("SOUND_VOLUME_INACTIVITY"),
                w_string(n.to_string()),
                ini_file,
            ),
            ("soundMinimized", Value::Number(n)) => WritePrivateProfileStringW(
                w!("SOUND"),
                w!("SOUND_VOLUME_MINIMIZE"),
                w_string(n.to_string()),
                ini_file,
            ),
            _ => {
                warn!("unknown setting: {}", name);
                Ok(())
            }
        }
    }
    .map_err(|e| {
        warn!("failed to write to config: {}, {}", name, e);
        "settings-error".to_owned()
    })
}
