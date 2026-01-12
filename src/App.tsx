import { useState, useEffect, useCallback, useRef } from 'react';
import { ImageCard } from './components/ImageCard';
import { ContextMenu } from './components/ContextMenu';
import { ImageLightbox } from './components/ImageLightbox';
import { getImages, deleteImage, clearAllImages, resetClipboardHash, listenClipboardUpdate, copyFileToClipboard } from './services/api';
import type { ImageMetadata, ContextMenuPosition } from './types';

function App() {
  const [images, setImages] = useState<ImageMetadata[]>([]);
  const [contextMenu, setContextMenu] = useState<{
    position: ContextMenuPosition | null;
    image: ImageMetadata | null;
  }>({
    position: null,
    image: null,
  });
  const [lightboxOpen, setLightboxOpen] = useState(false);
  const [lightboxIndex, setLightboxIndex] = useState(0);
  const contextMenuRef = useRef(contextMenu);
  contextMenuRef.current = contextMenu;
  const imagesRef = useRef(images);
  imagesRef.current = images;
  const lastCloseTimeRef = useRef(0);

  const loadImages = useCallback(async () => {
    try {
      const data = await getImages();
      setImages(data);
    } catch (error) {
      console.error('Failed to load images:', error);
    }
  }, []);

  const handleNativeContextMenu = useCallback((e: MouseEvent) => {
    const now = Date.now();
    // Ignore contextmenu events that occur within 300ms of closing a menu
    if (now - lastCloseTimeRef.current < 300) {
      return;
    }
    
    // 当 ContextMenu 已经打开时，让 ContextMenu 自己的处理器处理右键事件
    if (contextMenuRef.current.position) {
      return;
    }
    
    e.preventDefault();
    
    const x = e.clientX;
    const y = e.clientY;
    
    // First right click: check what was clicked
    const element = document.elementFromPoint(x, y);
    const imageCard = element?.closest('[data-image-card]');
    
    if (imageCard) {
      const imagePath = imageCard.getAttribute('data-image-path');
      const image = imagesRef.current.find(img => img.path === imagePath);
      
      if (image) {
        console.log('App: showing image menu');
        setContextMenu({
          position: { x, y },
          image,
        });
        return;
      }
    }
    
    // Also check if we're over a lightbox image element
    const lightboxImage = element?.closest('[data-lightbox-image]');
    if (lightboxImage) {
      const imagePath = lightboxImage.getAttribute('data-image-path');
      const image = imagesRef.current.find(img => img.path === imagePath);
      
      if (image) {
        console.log('App: showing image menu (from lightbox)');
        setContextMenu({
          position: { x, y },
          image,
        });
        return;
      }
    }
    
    // Only show clear all menu if we have images and clicked on the main app area
    const mainContent = element?.closest('main');
    if (mainContent && imagesRef.current.length > 0) {
      console.log('App: showing clear menu');
      setContextMenu({
        position: { x, y },
        image: null,
      });
      return;
    }
    
    // If clicked outside of main content or no images, don't show any menu
    console.log('App: ignoring context menu (outside main content or no images)');
  }, []);

  // 当 Lightbox 打开时，禁用 App 的 contextmenu 监听器，避免与 Lightbox 冲突
  useEffect(() => {
    if (lightboxOpen) {
      return;
    }
    
    loadImages();

    const setupClipboardListener = async () => {
      try {
        const unlisten = await listenClipboardUpdate((event) => {
          console.log('Clipboard update:', event);
          loadImages();
        });
        
        console.log('Clipboard listener set up successfully');
        
        return () => {
          console.log('Cleaning up clipboard listener');
          unlisten();
        };
      } catch (error) {
        console.error('Failed to set up clipboard listener:', error);
      }
    };

    const cleanupPromise = setupClipboardListener();

    // Use document-level capture listener to intercept contextmenu before other handlers
    document.addEventListener('contextmenu', handleNativeContextMenu, true);

    return () => {
      cleanupPromise.then((cleanup) => {
        if (cleanup) cleanup();
      });
      document.removeEventListener('contextmenu', handleNativeContextMenu, true);
    };
  }, [loadImages, handleNativeContextMenu, lightboxOpen]);

  const handleCloseContextMenu = useCallback(() => {
    lastCloseTimeRef.current = Date.now();
    setContextMenu({ position: null, image: null });
  }, []);

  const handleCopy = useCallback(async (image: ImageMetadata) => {
    try {
      await resetClipboardHash();
      await copyFileToClipboard(image.path);
      console.log('File path copied to clipboard:', image.path);
      
      await new Promise(resolve => setTimeout(resolve, 100));
      
      await loadImages();
      
      setImages(prev => {
        const contentGroups = new Map<string, ImageMetadata[]>();
        prev.forEach(img => {
          contentGroups.set(img.id, [...(contentGroups.get(img.id) || []), img]);
        });
        
        const result: ImageMetadata[] = [];
        contentGroups.forEach((imgs) => {
          const oldest = imgs.reduce((a, b) => a.createdAt < b.createdAt ? a : b);
          result.push(oldest);
        });
        
        return result.sort((a, b) => b.createdAt - a.createdAt);
      });
    } catch (error) {
      console.error('Failed to copy image:', error);
    }
  }, []);

  const handleDelete = useCallback(
    async (image: ImageMetadata) => {
      try {
        await deleteImage(image.id);
        setImages((prev) => prev.filter((img) => img.id !== image.id));
      } catch (error) {
        console.error('Failed to delete image:', error);
      }
    },
    []
  );

  const handleClearAll = useCallback(async () => {
    try {
      await clearAllImages();
      setImages([]);
    } catch (error) {
      console.error('Failed to clear all images:', error);
    }
  }, []);

  const handleImageClick = useCallback((index: number) => {
    setLightboxIndex(index);
    setLightboxOpen(true);
  }, []);

  const handleCloseLightbox = useCallback(() => {
    setLightboxOpen(false);
  }, []);

  return (
    <div className="min-h-screen relative overflow-hidden bg-gradient-to-br from-gray-50 to-gray-100 
                    dark:from-gray-900 dark:to-gray-800">
      <main 
        className="max-w-7xl mx-auto px-6 py-8 relative z-10 min-h-screen"
      >
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-5">
            {images.map((image, index) => (
              <ImageCard
                key={image.id}
                image={image}
                index={index}
                onImageClick={handleImageClick}
              />
            ))}
          </div>
      </main>

      {!lightboxOpen && (
        <ContextMenu
          position={contextMenu.position}
          image={contextMenu.image}
          onClose={handleCloseContextMenu}
          onCopy={handleCopy}
          onDelete={handleDelete}
          onClearAll={handleClearAll}
          onReposition={(position, targetImagePath) => {
            const targetImage = targetImagePath
              ? images.find(img => img.path === targetImagePath) || null
              : null;
            setContextMenu({
              position,
              image: targetImage,
            });
          }}
        />
      )}

      <ImageLightbox
        images={images}
        open={lightboxOpen}
        slideIndex={lightboxIndex}
        onClose={handleCloseLightbox}
        onCopy={handleCopy}
        onDelete={handleDelete}
        onContextMenuClose={() => {
          lastCloseTimeRef.current = Date.now();
        }}
      />
    </div>
  );
}

export default App;
