use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use image::ImageFormat;
use crate::types::ImageMetadata;

pub struct ImageManager {
    storage_dir: PathBuf,
    images: HashMap<String, ImageMetadata>,
}

impl ImageManager {
    pub fn new() -> anyhow::Result<Self> {
        let storage_dir = std::env::temp_dir().join("screenshot-hub");
        
        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir)?;
        }
        
        log::info!("ImageManager initialized with storage_dir: {}", storage_dir.display());
        
        Ok(Self {
            storage_dir,
            images: HashMap::new(),
        })
    }

    pub fn save_image(&mut self, image_data: &[u8]) -> anyhow::Result<(ImageMetadata, bool)> {
        let hash = Self::calculate_hash(image_data);
        
        log::debug!("[ImageManager] save_image called - hash: {}, data size: {} bytes", hash, image_data.len());
        
        if let Some(metadata) = self.images.get(&hash) {
            log::debug!("[ImageManager] Found in cache (hash: {}), returning cached metadata", hash);
            return Ok((metadata.clone(), true));
        }
        
        log::debug!("[ImageManager] Hash not in cache, scanning storage dir for duplicates...");
        
        let mut found_duplicate = false;
        let mut existing_file_path = None;
        let mut existing_file_hash = None;
        
        for entry in fs::read_dir(&self.storage_dir).map_err(|e| {
            log::error!("Failed to read storage directory: {}", e);
            anyhow::anyhow!("Failed to read storage directory: {}", e)
        })? {
            let entry = entry.map_err(|e| {
                log::error!("Failed to read directory entry: {}", e);
                e
            })?;
            let path = entry.path();
            if path.is_file() {
                let file_hash = path.file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                
                log::debug!("[ImageManager] Checking file: {} (hash from filename: {})", path.display(), file_hash);
                
                if let Ok(existing_data) = fs::read(&path) {
                    let data_equal = existing_data == image_data;
                    log::debug!("[ImageManager] Data comparison: data_equal={}, sizes: existing={}, new={}", 
                        data_equal, existing_data.len(), image_data.len());
                    
                    if data_equal || file_hash == hash {
                        log::info!("[ImageManager] Found duplicate! hash match: {}, content match: {}", file_hash == hash, data_equal);
                        
                        found_duplicate = true;
                        existing_file_path = Some(path.to_string_lossy().to_string());
                        existing_file_hash = Some(file_hash);
                        break;
                    }
                }
            }
        }
        
        if found_duplicate {
            let existing_path = existing_file_path.unwrap();
            let existing_hash = existing_file_hash.unwrap();
            
            log::info!("[ImageManager] Found duplicate image at: {}", existing_path);
            
            if let Some(existing_metadata) = self.images.get(&existing_hash) {
                return Ok((existing_metadata.clone(), true));
            }
            
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs() as i64;
            
            let metadata = ImageMetadata {
                id: existing_hash,
                path: existing_path,
                created_at: now,
                ocr_result: None,
            };
            
            return Ok((metadata, true));
        }
        
        log::debug!("[ImageManager] No duplicate found, checking if file already exists on disk...");
        
        let format = image::guess_format(image_data).unwrap_or(ImageFormat::Png);
        let extension = match format {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Gif => "gif",
            ImageFormat::WebP => "webp",
            ImageFormat::Bmp => "bmp",
            _ => "png",
        };
        
        let file_name = format!("{}.{}", hash, extension);
        let file_path = self.storage_dir.join(&file_name);
        
        if file_path.exists() {
            log::info!("[ImageManager] File already exists on disk (hash: {}), returning as duplicate", hash);
            
            if let Some(existing_metadata) = self.images.get(&hash) {
                return Ok((existing_metadata.clone(), true));
            }
            
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs() as i64;
            
            let metadata = ImageMetadata {
                id: hash.clone(),
                path: file_path.to_string_lossy().to_string(),
                created_at: now,
                ocr_result: None,
            };
            
            return Ok((metadata, true));
        }
        
        log::debug!("File does not exist, saving new image with hash: {} to path: {} (format: {:?})", hash, file_path.display(), format);
        
        if matches!(format, ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::Gif | ImageFormat::WebP | ImageFormat::Bmp) {
            fs::write(&file_path, image_data).map_err(|e| {
                log::error!("Failed to write image file: {}", e);
                anyhow::anyhow!("Failed to write image file: {}", e)
            })?;
            log::debug!("Saved original format image to: {}", file_path.display());
        } else {
            let image = image::load_from_memory(image_data).map_err(|e| {
                log::error!("Failed to load image from memory: {}", e);
                anyhow::anyhow!("Failed to load image from memory: {}", e)
            })?;
            image.save_with_format(&file_path, ImageFormat::Png).map_err(|e| {
                log::error!("Failed to save image: {}", e);
                anyhow::anyhow!("Failed to save image: {}", e)
            })?;
            log::debug!("Converted and saved image to PNG: {}", file_path.display());
        }
        
        log::debug!("Image saved successfully to: {}", file_path.display());
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() as i64;
        
        let metadata = ImageMetadata {
            id: hash.clone(),
            path: file_path.to_string_lossy().to_string(),
            created_at: now,
            ocr_result: None,
        };
        
        log::info!("Created metadata with path: {}", metadata.path);
        
        self.images.insert(hash.clone(), metadata.clone());
        
        Ok((metadata, false))
    }

    pub fn get_images(&self) -> Vec<ImageMetadata> {
        let mut images: Vec<ImageMetadata> = self.images.values().cloned().collect();
        images.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        images
    }

    pub fn delete_image(&mut self, id: &str) -> anyhow::Result<()> {
        if let Some(metadata) = self.images.remove(id) {
            let path = Path::new(&metadata.path);
            if path.exists() {
                fs::remove_file(path).map_err(|e| {
                    log::error!("Failed to delete image file: {}", e);
                    anyhow::anyhow!("Failed to delete image file: {}", e)
                })?;
            }
        }
        Ok(())
    }

    pub fn clear_all(&mut self) -> anyhow::Result<()> {
        let paths: Vec<PathBuf> = self.images.values()
            .map(|metadata| PathBuf::from(&metadata.path))
            .collect();
        
        for path in &paths {
            if path.exists() {
                if let Err(e) = fs::remove_file(path) {
                    log::error!("Failed to delete file during clear_all: {} - {}", path.display(), e);
                }
            }
        }
        
        self.images.clear();
        Ok(())
    }

    pub fn cleanup_old_images(&mut self, hours: i64) -> anyhow::Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() as i64;
        
        let threshold = now - (hours * 3600);
        
        let to_remove: Vec<String> = self.images
            .iter()
            .filter(|(_, metadata)| metadata.created_at < threshold)
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in to_remove {
            self.delete_image(&id)?;
        }
        
        Ok(())
    }

    fn calculate_hash(data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        hex::encode(result)
    }
}
