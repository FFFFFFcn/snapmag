use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use crate::types::ClipboardEvent;
use crate::image_manager::ImageManager;
use log::{info, error, debug};

pub struct ClipboardListener {
    handle: Arc<Mutex<Option<AppHandle>>>,
    running: Arc<Mutex<bool>>,
    image_manager: Arc<Mutex<ImageManager>>,
    last_hash: Arc<Mutex<Option<String>>>,
    last_detection_time: Arc<Mutex<u64>>,
}

const CLIPBOARD_COOLDOWN_MS: u64 = 2000;

impl ClipboardListener {
    pub fn new(image_manager: Arc<Mutex<ImageManager>>) -> Self {
        Self {
            handle: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
            image_manager,
            last_hash: Arc::new(Mutex::new(None)),
            last_detection_time: Arc::new(Mutex::new(0)),
        }
    }

    pub fn reset_hash(&self) {
        let mut last = self.last_hash.lock().unwrap();
        *last = None;
        info!("Clipboard listener hash reset");
    }

    pub fn start(&mut self, app_handle: AppHandle) {
        *self.handle.lock().unwrap() = Some(app_handle.clone());
        *self.running.lock().unwrap() = true;
        
        info!("Clipboard listener started");
        
        let handle = self.handle.clone();
        let running = self.running.clone();
        let image_manager = self.image_manager.clone();
        let last_hash = self.last_hash.clone();
        let last_detection_time = self.last_detection_time.clone();
        
        thread::spawn(move || {
            Self::listen_loop(handle, running, image_manager, last_hash, last_detection_time);
        });
    }

    #[cfg(target_os = "windows")]
    fn listen_loop(
        handle: Arc<Mutex<Option<AppHandle>>>,
        running: Arc<Mutex<bool>>,
        image_manager: Arc<Mutex<ImageManager>>,
        last_hash: Arc<Mutex<Option<String>>>,
        last_detection_time: Arc<Mutex<u64>>,
    ) {
        use windows::Win32::System::DataExchange::{OpenClipboard, CloseClipboard, GetClipboardData, EnumClipboardFormats, CountClipboardFormats};
        use windows::Win32::UI::Shell::HDROP;
        
        const CF_DIB: u32 = 8;
        const CF_DIBV5: u32 = 17;
        const CF_BITMAP: u32 = 2;
        const CF_HDROP: u32 = 15;
        
        info!("Clipboard listener loop started");
        
        while *running.lock().unwrap() {
            thread::sleep(Duration::from_millis(200));
            
            unsafe {
                debug!("Attempting to open clipboard");
                if OpenClipboard(None).is_ok() {
                    debug!("Clipboard opened successfully");
                    
                    let format_count = CountClipboardFormats();
                    debug!("Clipboard contains {} format(s)", format_count);
                    
                    let mut formats = Vec::new();
                    let mut format = EnumClipboardFormats(0);
                    while format != 0 {
                        formats.push(format);
                        debug!("Found clipboard format: {}", format);
                        format = EnumClipboardFormats(format);
                    }
                    debug!("Available formats: {:?}", formats);
                    
                    let mut image_data = None;
                    
                    if formats.contains(&CF_HDROP) {
                        debug!("Found CF_HDROP format (file copy)");
                        if let Ok(hdrop_handle) = GetClipboardData(CF_HDROP) {
                            debug!("Processing file drop handle: {:?}", hdrop_handle);
                            image_data = Self::extract_image_from_files(HDROP(hdrop_handle.0));
                            if image_data.is_some() {
                                debug!("Successfully extracted image from file drop");
                            } else {
                                debug!("No image found in file drop");
                            }
                        } else {
                            debug!("Failed to get CF_HDROP data");
                        }
                    }
                    
                    if image_data.is_none() && formats.contains(&CF_DIBV5) {
                        debug!("Found CF_DIBV5 format (likely screenshot)");
                        if let Ok(dib_handle) = GetClipboardData(CF_DIBV5) {
                            let dib_ptr = dib_handle.0 as *const u8;
                            debug!("CF_DIBV5 data handle: {:?}", dib_handle);
                            image_data = Self::extract_image_from_dibv5(dib_ptr);
                            if image_data.is_some() {
                                debug!("Successfully extracted image from CF_DIBV5");
                            } else {
                                debug!("Failed to extract image from CF_DIBV5");
                            }
                        } else {
                            debug!("Failed to get CF_DIBV5 data");
                        }
                    }
                    
                    if image_data.is_none() && formats.contains(&CF_DIB) {
                        debug!("Found CF_DIB format");
                        if let Ok(dib_handle) = GetClipboardData(CF_DIB) {
                            let dib_ptr = dib_handle.0 as *const u8;
                            debug!("CF_DIB data handle: {:?}", dib_handle);
                            image_data = Self::extract_image_from_dib(dib_ptr);
                            if image_data.is_some() {
                                debug!("Successfully extracted image from CF_DIB");
                            } else {
                                debug!("Failed to extract image from CF_DIB");
                            }
                        } else {
                            debug!("Failed to get CF_DIB data");
                        }
                    }
                    
                    if image_data.is_none() && formats.contains(&CF_BITMAP) {
                        debug!("Found CF_BITMAP format");
                        if let Ok(bitmap_handle) = GetClipboardData(CF_BITMAP) {
                            debug!("CF_BITMAP data handle: {:?}", bitmap_handle);
                            image_data = Self::extract_image_from_bitmap(bitmap_handle.0 as isize);
                            if image_data.is_some() {
                                debug!("Successfully extracted image from CF_BITMAP");
                            } else {
                                debug!("Failed to extract image from CF_BITMAP");
                            }
                        } else {
                            debug!("Failed to get CF_BITMAP data");
                        }
                    }
                    
                    let _ = CloseClipboard();
                    
                    if let Some(data) = image_data {
                        debug!("Successfully extracted image data, size: {} bytes", data.len());
                        
                        let hash = Self::calculate_hash(&data);
                        debug!("Calculated image hash: {}", hash);
                        
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0);
                        
                        let mut last_detection = last_detection_time.lock().unwrap();
                        if now < *last_detection + CLIPBOARD_COOLDOWN_MS {
                            debug!("Within cooldown window ({}ms), ignoring detection", CLIPBOARD_COOLDOWN_MS);
                            continue;
                        }
                        
                        let mut last = last_hash.lock().unwrap();
                        
                        if *last != Some(hash.clone()) {
                            info!("New image detected (hash: {})", hash);
                            *last = Some(hash.clone());
                            *last_detection = now;
                            drop(last);
                            drop(last_detection);
                            
                            match image_manager.lock().unwrap().save_image(&data) {
                                Ok((metadata, is_duplicate)) => {
                                    if is_duplicate {
                                        debug!("Duplicate image detected (hash: {}), not emitting event", hash);
                                    } else {
                                        info!("New image saved to: {}", metadata.path);
                                        let app_handle = handle.lock().unwrap();
                                        if let Some(handle) = app_handle.as_ref() {
                                            if let Err(e) = handle.emit("clipboard-update", ClipboardEvent {
                                                image_path: metadata.path.clone(),
                                            }) {
                                                error!("Failed to emit clipboard-update event: {}", e);
                                            } else {
                                                debug!("Emitted clipboard-update event for: {}", metadata.path);
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to save image: {}", e);
                                }
                            };
                        } else {
                            debug!("Same image detected (hash unchanged), skipping");
                        }
                    } else {
                        debug!("No image data extracted from clipboard");
                    }
                } else {
                    debug!("Failed to open clipboard");
                }
            }
        }
        
        info!("Clipboard listener loop stopped");
    }

    #[cfg(not(target_os = "windows"))]
    fn listen_loop(
        handle: Arc<Mutex<Option<AppHandle>>>,
        running: Arc<Mutex<bool>>,
        _image_manager: Arc<Mutex<ImageManager>>,
        _last_hash: Arc<Mutex<Option<String>>>,
    ) {
        while *running.lock().unwrap() {
            thread::sleep(Duration::from_secs(1));
            
            if let Some(app_handle) = &*handle.lock().unwrap() {
                let _ = app_handle.emit("clipboard-update", ClipboardEvent {
                    image_path: String::new(),
                });
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn calculate_hash(data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        
        hex::encode(result)
    }

    #[cfg(target_os = "windows")]
    unsafe fn extract_image_from_dib(dib_ptr: *const u8) -> Option<Vec<u8>> {
        use image::{ImageBuffer, Rgb, Rgba};
        
        debug!("Starting DIB extraction from pointer: {:?}", dib_ptr);
        
        if dib_ptr.is_null() {
            error!("DIB pointer is null");
            return None;
        }
        
        let dib_header = dib_ptr as *const u32;
        let bi_size = *dib_header as usize;
        
        debug!("DIB header size: {}", bi_size);
        
        if bi_size < 40 {
            error!("Invalid DIB header size: {}", bi_size);
            return None;
        }
        
        let bi_width = *dib_header.add(1) as i32;
        let bi_height = *dib_header.add(2) as i32;
        let bi_planes = *(dib_ptr.add(12) as *const u16);
        let bi_bit_count = *(dib_ptr.add(14) as *const u16);
        let bi_compression = *dib_header.add(4) as u32;
        
        debug!("DIB info - width: {}, height: {}, planes: {}, bit_count: {}, compression: {}", 
              bi_width, bi_height, bi_planes, bi_bit_count, bi_compression);
        
        if bi_width <= 0 || bi_height == 0 || bi_bit_count == 0 || bi_width > 10000 || bi_height.abs() > 10000 {
            error!("Invalid DIB parameters: width={}, height={}, bit_count={}", 
                  bi_width, bi_height, bi_bit_count);
            return None;
        }
        
        if bi_compression != 0 {
            error!("Compressed DIB not supported: compression={}", bi_compression);
            return None;
        }
        
        let abs_height = bi_height.abs();
        let width = bi_width as u32;
        let height = abs_height as u32;
        
        let bytes_per_pixel = (bi_bit_count as usize) / 8;
        if bytes_per_pixel == 0 {
            error!("Invalid bytes per pixel: {}", bytes_per_pixel);
            return None;
        }
        
        let row_size = ((width as usize * bytes_per_pixel + 3) / 4) * 4;
        let expected_data_size = row_size * height as usize;
        
        debug!("Image dimensions: {}x{}, bytes_per_pixel: {}, row_size: {}, expected_data_size: {}", 
              width, height, bytes_per_pixel, row_size, expected_data_size);
        
        let pixel_data_offset = bi_size;
        let pixel_data_ptr = dib_ptr.add(pixel_data_offset);
        
        match bi_bit_count {
            24 => {
                let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
                
                debug!("Processing 24-bit DIB");
                
                for y in 0..height {
                    let dib_y = if bi_height > 0 { height - 1 - y } else { y };
                    let dib_row_offset = dib_y as usize * row_size;
                    
                    for x in 0..width {
                        let dib_x = x as usize * 3;
                        let pixel_offset = dib_row_offset + dib_x;
                        
                        let b = *pixel_data_ptr.add(pixel_offset);
                        let g = *pixel_data_ptr.add(pixel_offset + 1);
                        let r = *pixel_data_ptr.add(pixel_offset + 2);
                        
                        img.put_pixel(x, y, Rgb([r, g, b]));
                    }
                }
                
                let mut output = Vec::new();
                if let Ok(()) = image::write_buffer_with_format(
                    &mut std::io::Cursor::new(&mut output),
                    img.as_raw(),
                    width,
                    height,
                    image::ExtendedColorType::Rgb8,
                    image::ImageFormat::Png,
                ) {
                    info!("Successfully converted DIB 24-bit to PNG, size: {} bytes", output.len());
                    return Some(output);
                } else {
                    error!("Failed to write PNG buffer for DIB 24-bit");
                }
            }
            32 => {
                let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);
                
                debug!("Processing 32-bit DIB");
                
                for y in 0..height {
                    let dib_y = if bi_height > 0 { height - 1 - y } else { y };
                    let dib_row_offset = dib_y as usize * row_size;
                    
                    for x in 0..width {
                        let dib_x = x as usize * 4;
                        let pixel_offset = dib_row_offset + dib_x;
                        
                        let b = *pixel_data_ptr.add(pixel_offset);
                        let g = *pixel_data_ptr.add(pixel_offset + 1);
                        let r = *pixel_data_ptr.add(pixel_offset + 2);
                        let a = *pixel_data_ptr.add(pixel_offset + 3);
                        
                        img.put_pixel(x, y, Rgba([r, g, b, a]));
                    }
                }
                
                let mut output = Vec::new();
                if let Ok(()) = image::write_buffer_with_format(
                    &mut std::io::Cursor::new(&mut output),
                    img.as_raw(),
                    width,
                    height,
                    image::ExtendedColorType::Rgba8,
                    image::ImageFormat::Png,
                ) {
                    info!("Successfully converted DIB 32-bit to PNG, size: {} bytes", output.len());
                    return Some(output);
                } else {
                    error!("Failed to write PNG buffer for DIB 32-bit");
                }
            }
            _ => {
                error!("Unsupported bit count: {}", bi_bit_count);
            }
        }
        
        None
    }

    #[cfg(target_os = "windows")]
    unsafe fn extract_image_from_dibv5(dib_ptr: *const u8) -> Option<Vec<u8>> {
        use image::{ImageBuffer, Rgb, Rgba};
        
        debug!("Starting DIBV5 extraction from pointer: {:?}", dib_ptr);
        
        if dib_ptr.is_null() {
            error!("DIBV5 pointer is null");
            return None;
        }
        
        let dib_header = dib_ptr as *const u32;
        let bi_size = *dib_header as usize;
        
        debug!("DIBV5 header size: {}", bi_size);
        
        if bi_size < 124 {
            error!("Invalid DIBV5 header size: {}", bi_size);
            return None;
        }
        
        let bi_width = *dib_header.add(1) as i32;
        let bi_height = *dib_header.add(2) as i32;
        let bi_planes = *(dib_ptr.add(12) as *const u16);
        let bi_bit_count = *(dib_ptr.add(14) as *const u16);
        let bi_compression = *dib_header.add(4) as u32;
        
        debug!("DIBV5 info - width: {}, height: {}, planes: {}, bit_count: {}, compression: {}", 
              bi_width, bi_height, bi_planes, bi_bit_count, bi_compression);
        
        // 验证参数合理性
        if bi_width <= 0 || bi_height == 0 || bi_bit_count == 0 || bi_width > 10000 || bi_height.abs() > 10000 {
            error!("Invalid DIBV5 parameters: width={}, height={}, bit_count={}", 
                  bi_width, bi_height, bi_bit_count);
            return None;
        }
        
        // 只支持无压缩的位图
        if bi_compression != 0 {
            error!("Compressed DIBV5 not supported: compression={}", bi_compression);
            return None;
        }
        
        let abs_height = bi_height.abs();
        let width = bi_width as u32;
        let height = abs_height as u32;
        
        let bytes_per_pixel = (bi_bit_count as usize) / 8;
        if bytes_per_pixel == 0 {
            error!("Invalid bytes per pixel: {}", bytes_per_pixel);
            return None;
        }
        
        let row_size = ((width as usize * bytes_per_pixel + 3) / 4) * 4;
        let expected_data_size = row_size * height as usize;
        
        debug!("DIBV5 Image dimensions: {}x{}, bytes_per_pixel: {}, row_size: {}, expected_data_size: {}", 
              width, height, bytes_per_pixel, row_size, expected_data_size);
        
        let pixel_data_offset = bi_size;
        let pixel_data_ptr = dib_ptr.add(pixel_data_offset);
        
        match bi_bit_count {
            24 => {
                let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
                
                debug!("Processing DIBV5 24-bit");
                
                for y in 0..height {
                    let dib_y = if bi_height > 0 { height - 1 - y } else { y };
                    let dib_row_offset = dib_y as usize * row_size;
                    
                    for x in 0..width {
                        let dib_x = x as usize * 3;
                        let pixel_offset = dib_row_offset + dib_x;
                        
                        let b = *pixel_data_ptr.add(pixel_offset);
                        let g = *pixel_data_ptr.add(pixel_offset + 1);
                        let r = *pixel_data_ptr.add(pixel_offset + 2);
                        
                        img.put_pixel(x, y, Rgb([r, g, b]));
                    }
                }
                
                let mut output = Vec::new();
                if let Ok(()) = image::write_buffer_with_format(
                    &mut std::io::Cursor::new(&mut output),
                    img.as_raw(),
                    width,
                    height,
                    image::ExtendedColorType::Rgb8,
                    image::ImageFormat::Png,
                ) {
                    debug!("Successfully converted DIBV5 24-bit to PNG, size: {} bytes", output.len());
                    return Some(output);
                } else {
                    error!("Failed to write PNG buffer for DIBV5 24-bit");
                }
            }
            32 => {
                let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);
                
                debug!("Processing DIBV5 32-bit");
                
                for y in 0..height {
                    let dib_y = if bi_height > 0 { height - 1 - y } else { y };
                    let dib_row_offset = dib_y as usize * row_size;
                    
                    for x in 0..width {
                        let dib_x = x as usize * 4;
                        let pixel_offset = dib_row_offset + dib_x;
                        
                        let b = *pixel_data_ptr.add(pixel_offset);
                        let g = *pixel_data_ptr.add(pixel_offset + 1);
                        let r = *pixel_data_ptr.add(pixel_offset + 2);
                        let a = *pixel_data_ptr.add(pixel_offset + 3);
                        
                        img.put_pixel(x, y, Rgba([r, g, b, a]));
                    }
                }
                
                let mut output = Vec::new();
                if let Ok(()) = image::write_buffer_with_format(
                    &mut std::io::Cursor::new(&mut output),
                    img.as_raw(),
                    width,
                    height,
                    image::ExtendedColorType::Rgba8,
                    image::ImageFormat::Png,
                ) {
                    debug!("Successfully converted DIBV5 32-bit to PNG, size: {} bytes", output.len());
                    return Some(output);
                } else {
                    error!("Failed to write PNG buffer for DIBV5 32-bit");
                }
            }
            _ => {
                error!("Unsupported DIBV5 bit count: {}", bi_bit_count);
            }
        }
        
        None
    }

    #[cfg(target_os = "windows")]
    unsafe fn extract_image_from_bitmap(_bitmap_handle: isize) -> Option<Vec<u8>> {
        None
    }

    #[cfg(target_os = "windows")]
    unsafe fn extract_image_from_files(hdrop: windows::Win32::UI::Shell::HDROP) -> Option<Vec<u8>> {
        use windows::Win32::UI::Shell::DragQueryFileW;
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        
        debug!("Processing file drop from clipboard");
        
        let file_count = DragQueryFileW(hdrop, 0xFFFFFFFF, None);
        debug!("Found {} files in clipboard", file_count);
        
        for i in 0..file_count {
            let mut buffer = vec![0u16; 260]; // MAX_PATH
            let length = DragQueryFileW(hdrop, i, Some(&mut buffer));
            
            if length > 0 {
                buffer.truncate(length as usize);
                let file_path = OsString::from_wide(&buffer);
                let file_path_str = file_path.to_string_lossy();
                debug!("Processing file: {}", file_path_str);
                
                // 检查是否是图片文件
                let lower_path = file_path_str.to_lowercase();
                if lower_path.ends_with(".png") || 
                   lower_path.ends_with(".jpg") || 
                   lower_path.ends_with(".jpeg") || 
                   lower_path.ends_with(".bmp") || 
                   lower_path.ends_with(".gif") || 
                   lower_path.ends_with(".webp") || 
                   lower_path.ends_with(".tiff") || 
                   lower_path.ends_with(".tif") {
                    
                    debug!("Found image file: {}", file_path_str);
                    
                    if let Ok(image_bytes) = std::fs::read(&file_path_str.to_string()) {
                        debug!("Read file successfully, size: {} bytes", image_bytes.len());
                        
                        if let Ok(format) = image::guess_format(&image_bytes) {
                            debug!("Detected image format: {:?}", format);
                            
                            match format {
                                image::ImageFormat::Png | 
                                image::ImageFormat::Jpeg | 
                                image::ImageFormat::Gif | 
                                image::ImageFormat::WebP | 
                                image::ImageFormat::Bmp => {
                                    debug!("Returning original format data");
                                    return Some(image_bytes);
                                }
                                _ => {
                                    debug!("Converting unsupported format to PNG");
                                    if let Ok(img) = image::load_from_memory(&image_bytes) {
                                        let mut png_data = Vec::new();
                                        if img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png).is_ok() {
                                            debug!("Successfully converted to PNG, size: {} bytes", png_data.len());
                                            return Some(png_data);
                                        }
                                    }
                                }
                            }
                        } else {
                            debug!("Could not detect format, trying to load as image");
                            if let Ok(img) = image::load_from_memory(&image_bytes) {
                                let mut png_data = Vec::new();
                                if img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png).is_ok() {
                                    debug!("Successfully loaded and converted to PNG, size: {} bytes", png_data.len());
                                    return Some(png_data);
                                }
                            }
                        }
                    } else {
                        error!("Failed to read file: {}", file_path_str);
                    }
                } else {
                    info!("Skipping non-image file: {}", file_path_str);
                }
            }
        }
        
        info!("No valid image files found in clipboard");
        None
    }
}
