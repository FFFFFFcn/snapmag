import { useState, useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';
import Lightbox from 'yet-another-react-lightbox';
import 'yet-another-react-lightbox/styles.css';
import type { ImageMetadata, ContextMenuPosition } from '../types';
import { ContextMenu } from './ContextMenu';

interface ImageLightboxProps {
  images: ImageMetadata[];
  open: boolean;
  slideIndex: number;
  onClose: () => void;
  onCopy?: (image: ImageMetadata) => void;
  onDelete?: (image: ImageMetadata) => void;
  onContextMenuClose?: () => void;
}

// 图片缓存，放在组件外部避免重新渲染
const imageCache: Map<string, string> = new Map();

async function getImageUrl(path: string): Promise<string> {
  if (imageCache.has(path)) {
    return imageCache.get(path)!;
  }
  
  try {
    const { readImageFile } = await import('../services/api');
    const imageData = await readImageFile(path);
    const blob = new Blob([imageData.buffer as ArrayBuffer], { type: 'image/png' });
    const url = URL.createObjectURL(blob);
    imageCache.set(path, url);
    return url;
  } catch (err) {
    console.error('Failed to load image:', err);
    return '';
  }
}

// 自定义下一张按钮
function NextButton({ onClick }: { onClick: () => void }) {
  return (
    <button
      onClick={(e) => {
        e.stopPropagation();
        onClick();
      }}
      aria-label="下一张"
      className="fixed right-4 top-1/2 -translate-y-1/2 w-10 h-10 flex items-center justify-center rounded-full bg-white/10 hover:bg-white/20 transition-colors"
      style={{ cursor: 'pointer', zIndex: 10000 }}
    >
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 36 36" className="w-6 h-6 fill-white">
        <path d="m22.5597 16.9375-5.5076-5.5c-.5854-.5854-1.5323-.5825-2.1157.0039-.5835.5869-.5815 1.5366.0039 2.1211l4.4438 4.4375-4.4438 4.4375c-.5854.5845-.5874 1.5342-.0039 2.1211.2922.2944.676.4414 1.0598.4414.3818 0 .7637-.1455 1.0559-.4375l5.5076-5.5c.2815-.2812.4403-.6636.4403-1.0625s-.1588-.7812-.4403-1.0625z"></path>
      </svg>
    </button>
  );
}

// 自定义上一张按钮
function PrevButton({ onClick }: { onClick: () => void }) {
  return (
    <button
      onClick={(e) => {
        e.stopPropagation();
        onClick();
      }}
      aria-label="上一张"
      className="fixed left-4 top-1/2 -translate-y-1/2 w-10 h-10 flex items-center justify-center rounded-full bg-white/10 hover:bg-white/20 transition-colors"
      style={{ cursor: 'pointer', zIndex: 10000 }}
    >
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 36 36" className="w-6 h-6 fill-white" style={{ transform: 'rotate(180deg)' }}>
        <path d="m22.5597 16.9375-5.5076-5.5c-.5854-.5854-1.5323-.5825-2.1157.0039-.5835.5869-.5815 1.5366.0039 2.1211l4.4438 4.4375-4.4438 4.4375c-.5854.5845-.5874 1.5342-.0039 2.1211.2922.2944.676.4414 1.0598.4414.3818 0 .7637-.1455 1.0559-.4375l5.5076-5.5c.2815-.2812.4403-.6636.4403-1.0625s-.1588-.7812-.4403-1.0625z"></path>
      </svg>
    </button>
  );
}

// 自定义 slide 渲染组件
function ImageSlide({ 
  slide, 
  onClose, 
  onShowContextMenu 
}: { 
  slide: { src: string; path: string }, 
  onClose: () => void,
  onShowContextMenu: (x: number, y: number) => void
}) {
  const [src, setSrc] = useState(slide.src);

  if (!src && slide.path) {
    getImageUrl(slide.path).then(setSrc);
  }

  if (!src) {
    return (
      <div 
        className="flex items-center justify-center w-full h-full"
        onClick={onClose}
        style={{ cursor: 'default' }}
      >
        <div className="text-white">加载中...</div>
      </div>
    );
  }

  return (
    <div
      data-lightbox-image
      data-image-path={slide.path}
      style={{
        width: '100%',
        height: '100%',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        cursor: 'default',
      }}
      onClick={onClose}
      onContextMenu={(e) => {
        e.preventDefault();
        e.stopPropagation();
        console.log('ImageLightbox: right-click detected at', e.clientX, e.clientY);
        onShowContextMenu(e.clientX, e.clientY);
      }}
    >
      <img
        src={src}
        alt=""
        data-image-path={slide.path}
        style={{
          maxWidth: '50vw',
          maxHeight: '90vh',
          objectFit: 'contain',
          cursor: 'default',
        }}
        onClick={(e) => e.stopPropagation()}
      />
    </div>
  );
}

export function ImageLightbox({ 
  images, 
  open, 
  slideIndex, 
  onClose,
  onCopy,
  onDelete,
  onContextMenuClose,
}: ImageLightboxProps) {
  const [currentIndex, setCurrentIndex] = useState(slideIndex);
  const [contextMenu, setContextMenu] = useState<{
    position: ContextMenuPosition | null;
    image: ImageMetadata | null;
  }>({
    position: null,
    image: null,
  });
  const contextMenuRef = useRef(contextMenu);
  contextMenuRef.current = contextMenu;
  const isClosingRef = useRef(false);

  useEffect(() => {
    if (open) {
      setCurrentIndex(slideIndex);
    }
  }, [open, slideIndex]);

  useEffect(() => {
    if (!open) return;

    const handleContextMenu = (e: MouseEvent) => {
      if (isClosingRef.current) {
        isClosingRef.current = false;
        return;
      }

      if (contextMenuRef.current.position) {
        e.preventDefault();
        console.log('ImageLightbox: blocked system context menu');
        const currentImage = images[currentIndex];
        setContextMenu({ position: null, image: null });
        requestAnimationFrame(() => {
          if (currentImage) {
            setContextMenu({
              position: { x: e.clientX, y: e.clientY },
              image: currentImage,
            });
          }
        });
        return;
      }

      const currentImage = images[currentIndex];
      if (currentImage) {
        e.preventDefault();
        setContextMenu({
          position: { x: e.clientX, y: e.clientY },
          image: currentImage,
        });
      }
    };

    document.addEventListener('contextmenu', handleContextMenu, true);

    return () => {
      document.removeEventListener('contextmenu', handleContextMenu, true);
    };
  }, [open, currentIndex, images]);

  const showContextMenu = (x: number, y: number) => {
    isClosingRef.current = true;
    const currentImage = images[currentIndex];
    console.log('showContextMenu called:', { x, y, currentImage });
    setContextMenu({ position: null, image: null });
    requestAnimationFrame(() => {
      isClosingRef.current = false;
      if (currentImage) {
        setContextMenu({
          position: { x, y },
          image: currentImage,
        });
      }
    });
  };

  console.log('ImageLightbox render:', { 
    hasContextMenu: !!contextMenu.position, 
    position: contextMenu.position,
    image: contextMenu.image?.path 
  });

  if (!open) {
    return null;
  }

  const slides = images.map(image => ({
    src: imageCache.get(image.path) || '',
    path: image.path,
  }));

  const handlePrev = () => {
    setCurrentIndex((prev) => (prev > 0 ? prev - 1 : slides.length - 1));
  };

  const handleNext = () => {
    setCurrentIndex((prev) => (prev < slides.length - 1 ? prev + 1 : 0));
  };

  const handleCloseContextMenu = () => {
    isClosingRef.current = true;
    setContextMenu({ position: null, image: null });
    onContextMenuClose?.();
  };

  const handleDelete = () => {
    if (contextMenu.image && onDelete) {
      onDelete(contextMenu.image);
      handleCloseContextMenu();
      onClose();
    }
  };

  const handleCopy = () => {
    if (contextMenu.image && onCopy) {
      onCopy(contextMenu.image);
      handleCloseContextMenu();
    }
  };

  return (
    <div className="lightbox-overlay">
      <style>{`
        .lightbox-overlay {
          position: fixed;
          inset: 0;
          z-index: 9999;
        }
        .lightbox-overlay .yarl__root {
          background: #1D1D1F !important;
        }
        .lightbox-overlay .yarl__button {
          display: none !important;
        }
        .lightbox-overlay .yarl__slide {
          display: flex !important;
          align-items: center !important;
          justify-content: center !important;
        }
        .lightbox-overlay .yarl__slide img {
          max-width: 50vw !important;
          max-height: 90vh !important;
          object-fit: contain !important;
        }
      `}</style>

      <Lightbox
        open={open}
        close={onClose}
        index={currentIndex}
        slides={slides}
        render={{
          slide: ({ slide }) => (
            <ImageSlide 
              slide={slide as { src: string; path: string }} 
              onClose={onClose}
              onShowContextMenu={showContextMenu}
            />
          ),
          buttonClose: () => null,
          buttonPrev: () => <PrevButton onClick={handlePrev} />,
          buttonNext: () => <NextButton onClick={handleNext} />,
        }}
        controller={{
          closeOnBackdropClick: false,
        }}
        styles={{
          root: { backgroundColor: '#1D1D1F' },
          container: { backgroundColor: '#1D1D1F' },
        }}
      />

      {contextMenu.position && createPortal(
        <ContextMenu
          position={contextMenu.position}
          image={contextMenu.image}
          onClose={handleCloseContextMenu}
          onCopy={handleCopy}
          onDelete={handleDelete}
          onClearAll={() => {}}
        />,
        document.body
      )}
    </div>
  );
}
