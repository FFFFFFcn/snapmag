// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
fn check_single_instance() -> bool {
    use winapi::um::winuser::{FindWindowW, SetForegroundWindow, ShowWindow, SetFocus, SW_RESTORE};
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::shared::winerror::ERROR_ALREADY_EXISTS;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    const MUTEX_NAME: &str = "SnapMag_SingleInstance_Mutex";
    const WINDOW_TITLE: &str = "SnapMag";

    let mutex_name: Vec<u16> = OsStr::new(MUTEX_NAME).encode_wide().chain(std::iter::once(0)).collect();
    let window_title: Vec<u16> = OsStr::new(WINDOW_TITLE).encode_wide().chain(std::iter::once(0)).collect();

    unsafe {
        let mutex = winapi::um::synchapi::CreateMutexW(
            std::ptr::null_mut(),
            0,
            mutex_name.as_ptr()
        );

        if mutex.is_null() {
            println!("Failed to create mutex, error: {}", GetLastError());
            return true;
        }

        let last_error = GetLastError();

        if last_error == ERROR_ALREADY_EXISTS {
            println!("Another instance detected, activating existing window...");

            let hwnd = FindWindowW(std::ptr::null(), window_title.as_ptr());
            if !hwnd.is_null() {
                println!("Found window, activating...");
                // 使用 SW_RESTORE 确保窗口从最小化恢复
                ShowWindow(hwnd, SW_RESTORE);
                SetForegroundWindow(hwnd);
                SetFocus(hwnd);
            } else {
                println!("Window not found");
            }

            winapi::um::handleapi::CloseHandle(mutex);
            return false;
        }

        println!("First instance, starting...");
    }

    true
}

#[cfg(not(target_os = "windows"))]
fn check_single_instance() -> bool {
    true
}

fn main() {
    let is_first_instance = check_single_instance();

    if !is_first_instance {
        return;
    }

    app_lib::run();
}
