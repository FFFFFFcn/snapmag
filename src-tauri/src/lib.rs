mod types;
mod image_manager;
mod clipboard;

use std::sync::{Arc, Mutex};
use tauri::{
    Manager,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{Menu, MenuItem},
    image::Image,
};
use types::ImageMetadata;
use image_manager::ImageManager;
use clipboard::ClipboardListener;

struct AppState {
    image_manager: Arc<Mutex<ImageManager>>,
    clipboard_listener: Arc<Mutex<ClipboardListener>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            
            let icon_bytes = include_bytes!("../icons/256x256.png");
            let img = image::load_from_memory(icon_bytes).expect("Failed to load window icon");
            let rgba = img.to_rgba8();
            let icon_image = Image::new_owned(rgba.to_vec(), 256, 256);
            window.set_icon(icon_image).expect("Failed to set window icon");
            
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            
            let image_manager = Arc::new(Mutex::new(ImageManager::new()?));
            let clipboard_listener = Arc::new(Mutex::new(ClipboardListener::new(image_manager.clone())));
            
            clipboard_listener.lock().unwrap().start(app.handle().clone());
            
            let app_state = AppState {
                image_manager: image_manager.clone(),
                clipboard_listener: clipboard_listener.clone(),
            };
            
            app.manage(app_state);

            let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let icon_bytes = include_bytes!("../icons/tray-icon.png");
            let img = image::load_from_memory(icon_bytes).expect("Failed to load tray icon");
            let rgba = img.to_rgba8();
            let tray_icon = Image::new_owned(rgba.to_vec(), 64, 64);
            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .tooltip("SnapMag")
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                #[cfg(target_os = "windows")]
                                {
                                    use winapi::um::winuser::{ShowWindow, SW_RESTORE, SetForegroundWindow, SetFocus};
                                    if let Ok(hwnd) = window.hwnd() {
                                        let hwnd_ptr = hwnd.0 as *mut _;
                                        unsafe {
                                            ShowWindow(hwnd_ptr, SW_RESTORE);
                                            SetForegroundWindow(hwnd_ptr);
                                            SetFocus(hwnd_ptr);
                                        }
                                        return;
                                    }
                                }
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            #[cfg(target_os = "windows")]
                            {
                                use winapi::um::winuser::{ShowWindow, SW_RESTORE, SetForegroundWindow, SetFocus};
                                if let Ok(hwnd) = window.hwnd() {
                                    let hwnd_ptr = hwnd.0 as *mut _;
                                    unsafe {
                                        ShowWindow(hwnd_ptr, SW_RESTORE);
                                        SetForegroundWindow(hwnd_ptr);
                                        SetFocus(hwnd_ptr);
                                    }
                                    return;
                                }
                            }
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                #[cfg(target_os = "windows")]
                {
                    use winapi::um::winuser::{ShowWindow, SW_HIDE};
                    if let Ok(hwnd) = window.hwnd() {
                        unsafe {
                            ShowWindow(hwnd.0 as *mut _, SW_HIDE);
                        }
                        api.prevent_close();
                        return;
                    }
                }
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_images,
            delete_image,
            save_image_from_clipboard,
            cleanup_old_images,
            read_image_file,
            clear_all_images,
            reset_clipboard_hash,
            copy_file_to_clipboard
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn get_images(state: tauri::State<'_, AppState>) -> Result<Vec<ImageMetadata>, String> {
    let images = state.image_manager
        .lock()
        .unwrap()
        .get_images();
    
    Ok(images
        .into_iter()
        .map(|mut metadata| {
            metadata.path = convert_path_protocol(&metadata.path);
            metadata
        })
        .collect::<Vec<_>>())
}

#[tauri::command]
async fn delete_image(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.image_manager
        .lock()
        .unwrap()
        .delete_image(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_image_from_clipboard(image_data: Vec<u8>, state: tauri::State<'_, AppState>) -> Result<ImageMetadata, String> {
    let (metadata, _) = state.image_manager
        .lock()
        .unwrap()
        .save_image(&image_data)
        .map_err(|e| e.to_string())?;
    
    let metadata = ImageMetadata {
        path: convert_path_protocol(&metadata.path),
        ..metadata
    };
    
    Ok(metadata)
}

#[tauri::command]
async fn cleanup_old_images(hours: i64, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.image_manager
        .lock()
        .unwrap()
        .cleanup_old_images(hours)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn clear_all_images(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.image_manager
        .lock()
        .unwrap()
        .clear_all()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn reset_clipboard_hash(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.clipboard_listener
        .lock()
        .unwrap()
        .reset_hash();
    Ok(())
}

#[tauri::command]
async fn copy_file_to_clipboard(path: String, _state: tauri::State<'_, AppState>) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::Win32::System::DataExchange::{OpenClipboard, EmptyClipboard, SetClipboardData, CloseClipboard};
use windows::Win32::UI::Shell::DROPFILES;
use windows::Win32::Foundation::HANDLE;
    
    let actual_path = path.replace("asset://localhost/", "").replace("asset://", "");
    let path_obj = Path::new(&actual_path);
    
    if !path_obj.exists() {
        return Err(format!("File not found: {}", actual_path));
    }
    
    let file_path_wide: Vec<u16> = path_obj
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    
    let drop_struct_size = std::mem::size_of::<DROPFILES>() as u32;
    let file_path_size = (file_path_wide.len() * 2) as u32;
    let total_size = drop_struct_size + file_path_size + 2;
    
    let mut buffer = vec![0u8; total_size as usize];
    
    let drop_files = DROPFILES {
        pFiles: drop_struct_size,
        pt: windows::Win32::Foundation::POINT { x: 0, y: 0 },
        fNC: false.into(),
        fWide: true.into(),
    };
    
    let drop_files_bytes = unsafe {
        std::slice::from_raw_parts(
            &drop_files as *const _ as *const u8,
            std::mem::size_of::<DROPFILES>(),
        )
    };
    buffer[..drop_files_bytes.len()].copy_from_slice(drop_files_bytes);
    
    let offset = drop_struct_size as usize;
    for (i, &code) in file_path_wide.iter().enumerate() {
        let byte_offset = offset + i * 2;
        if byte_offset + 1 < buffer.len() {
            buffer[byte_offset] = (code & 0xFF) as u8;
            buffer[byte_offset + 1] = (code >> 8) as u8;
        }
    }
    
    unsafe {
        if let Err(e) = OpenClipboard(None).map_err(|_| "Failed to open clipboard".to_string()) {
            log::error!("Clipboard error: {}", e);
            return Err(e);
        }
        
        let _ = EmptyClipboard();
          
          let global_alloc = match windows::Win32::System::Memory::GlobalAlloc(
            windows::Win32::System::Memory::GMEM_MOVEABLE,
            total_size as usize,
        ) {
            Ok(h) => h,
            Err(e) => {
                let _ = CloseClipboard();
                let err = format!("Failed to allocate global memory: {:?}", e);
                log::error!("{}", err);
                return Err(err);
            }
        };
        
        let global_lock = windows::Win32::System::Memory::GlobalLock(global_alloc);
        if global_lock.is_null() {
            let _ = CloseClipboard();
            let err = "Failed to lock global memory".to_string();
            log::error!("{}", err);
            return Err(err);
        }
        
        std::ptr::copy_nonoverlapping(
            buffer.as_ptr(),
            global_lock as *mut u8,
            total_size as usize,
        );
        
        let _ = windows::Win32::System::Memory::GlobalUnlock(global_alloc);
        
        if SetClipboardData(15u32, Some(HANDLE(global_alloc.0 as *mut _))).is_err() {
            let _ = CloseClipboard();
            let err = "Failed to set clipboard data".to_string();
            log::error!("{}", err);
            return Err(err);
        }
        
        let _ = CloseClipboard();
      }
    
    log::info!("Copied file path to clipboard: {}", actual_path);
    Ok(())
}

#[tauri::command]
async fn read_image_file(path: String) -> Result<Vec<u8>, String> {
    let actual_path = path.replace("asset://localhost/", "").replace("asset://", "");
    log::debug!("Reading image file from: {}", actual_path);
    
    std::fs::read(&actual_path).map_err(|e| {
        log::error!("Failed to read image file: {}", e);
        e.to_string()
    })
}

fn convert_path_protocol(path: &str) -> String {
    log::debug!("Converting path: {}", path);
    let result = if path.starts_with("asset://") {
        path.to_string()
    } else {
        format!("asset://localhost/{}", path.replace('\\', "/"))
    };
    log::debug!("Converted to: {}", result);
    result
}
