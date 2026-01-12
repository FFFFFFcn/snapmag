import { useState, useRef, useEffect, useCallback } from 'react';
import { Image as ImageIcon } from 'lucide-react';
import { readImageFile } from '../services/api';
import type { ImageMetadata } from '../types';

interface ImageCardProps {
  image: ImageMetadata;
  index: number;
  onImageClick: (index: number) => void;
}

export function ImageCard({ image, index, onImageClick }: ImageCardProps) {
  const cardRef = useRef<HTMLDivElement>(null);
  const [imageUrl, setImageUrl] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    let mounted = true;

    const loadImage = async () => {
      try {
        setLoading(true);
        setError(false);
        const imageData = await readImageFile(image.path);
        
        if (mounted) {
          const blob = new Blob([imageData.buffer as ArrayBuffer], { type: 'image/png' });
          const url = URL.createObjectURL(blob);
          setImageUrl(url);
        }
      } catch (err) {
        console.error('Failed to load image:', err);
        if (mounted) {
          setError(true);
        }
      } finally {
        if (mounted) {
          setLoading(false);
        }
      }
    };

    loadImage();

    return () => {
      mounted = false;
      if (imageUrl) {
        URL.revokeObjectURL(imageUrl);
      }
    };
  }, [image.path]);

  useEffect(() => {
    const card = cardRef.current;
    if (!card) return;

    const handleContextMenu = (_e: MouseEvent) => {
      // Don't stop propagation - let the event bubble up to App
      // App will detect which element is under the mouse and show the appropriate menu
    };

    card.addEventListener('contextmenu', handleContextMenu, true);

    return () => {
      card.removeEventListener('contextmenu', handleContextMenu, true);
    };
  }, []);

  const handleCardClick = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    onImageClick(index);
  }, [index, onImageClick]);

  return (
    <div
      ref={cardRef}
      onClick={handleCardClick}
      data-image-card
      data-image-path={image.path}
      className="liquid-glass-card rounded-2xl overflow-hidden liquid-shadow-medium transition-all duration-500 cursor-default select-none"
    >
      <div className="relative aspect-[4/3] overflow-hidden bg-gray-100 dark:bg-gray-800">
        {loading ? null : error ? (
          <div className="w-full h-full flex items-center justify-center liquid-glass-highlight">
            <div className="text-center">
              <ImageIcon size={32} className="text-gray-400 mx-auto mb-2" />
              <p className="text-sm liquid-text-tertiary">加载失败</p>
            </div>
          </div>
        ) : (
          <img
            src={imageUrl}
            alt="Screenshot"
            className="w-full h-full object-cover"
            draggable={false}
          />
        )}
      </div>
    </div>
  );
}
