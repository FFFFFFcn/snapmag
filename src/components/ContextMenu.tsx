import { useEffect, useRef, useMemo } from 'react';
import { Copy, Trash2 } from 'lucide-react';
import type { ImageMetadata, ContextMenuPosition } from '../types';

interface ContextMenuProps {
  position: ContextMenuPosition | null;
  image: ImageMetadata | null;
  onClose: () => void;
  onCopy: (image: ImageMetadata) => void;
  onDelete: (image: ImageMetadata) => void;
  onClearAll: () => void;
  onReposition?: (position: { x: number; y: number }, targetImagePath: string | null) => void;
}

const MENU_WIDTH = 192;
const MENU_OVERFLOW_THRESHOLD = MENU_WIDTH * 1;

export function ContextMenu({
  position,
  image,
  onClose,
  onCopy,
  onDelete,
  onClearAll,
  onReposition,
}: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        console.log('ContextMenu: closing menu via escape');
        onClose();
      }
    };

    const handleNativeContextMenu = (event: MouseEvent) => {
      if (event.button !== 2) return;

      event.preventDefault();
      event.stopPropagation();

      console.log('ContextMenu: global right-click detected', event.target);

      if (!onReposition) {
        onClose();
        return;
      }

      const targetImageElement = (event.target as HTMLElement).closest('[data-image-path]');

      if (targetImageElement) {
        const imagePath = targetImageElement.getAttribute('data-image-path');

        console.log('ContextMenu: repositioning to image menu (from target)', imagePath);
        onReposition({ x: event.clientX, y: event.clientY }, imagePath);
      } else {
        const elements = document.elementsFromPoint(event.clientX, event.clientY);
        const imageCard = elements.find(el => el.getAttribute('data-image-card') !== null);

        console.log('ContextMenu: reposition check - imageCard:', imageCard ? 'found' : 'null');

        if (imageCard) {
          const imagePath = imageCard.getAttribute('data-image-path');

          console.log('ContextMenu: repositioning to image menu (from DOM)', imagePath);
          onReposition({ x: event.clientX, y: event.clientY }, imagePath);
        } else {
          console.log('ContextMenu: repositioning to clear menu');
          onReposition({ x: event.clientX, y: event.clientY }, null);
        }
      }
    };

    document.addEventListener('keydown', handleEscape);
    document.addEventListener('contextmenu', handleNativeContextMenu, true);

    return () => {
      console.log('ContextMenu: cleaning up listeners');
      document.removeEventListener('keydown', handleEscape);
      document.removeEventListener('contextmenu', handleNativeContextMenu, true);
    };
  }, [onClose, onReposition]);

  const adjustedPosition = useMemo(() => {
    if (!position) return null;

    const windowWidth = window.innerWidth;
    const distanceToRightEdge = windowWidth - position.x;

    if (distanceToRightEdge < MENU_OVERFLOW_THRESHOLD) {
      return {
        x: position.x - MENU_WIDTH,
        y: position.y,
      };
    }

    return {
      x: position.x,
      y: position.y,
    };
  }, [position]);

  if (!adjustedPosition) return null;

  const hasImage = image !== null && image !== undefined;
  console.log('ContextMenu render:', { hasImage, imagePath: image?.path });

  const menuItems = hasImage
    ? [
        {
          icon: <Copy size={16} />,
          label: '复制',
          onClick: () => onCopy(image!),
        },
        {
          icon: <Trash2 size={16} />,
          label: '删除',
          onClick: () => onDelete(image!),
          danger: true,
        },
      ]
    : [
        {
          icon: <Trash2 size={16} />,
          label: '清空',
          onClick: onClearAll,
          danger: true,
        },
      ];

  return (
    <>
      <div 
        className="fixed inset-0 z-[10000] cursor-default"
        style={{ pointerEvents: 'auto' }}
        onMouseDown={(e) => {
          if (e.button === 0) {
            e.preventDefault();
            e.stopPropagation();
            console.log('ContextMenu: closing menu via overlay mousedown');
            onClose();
          }
        }}
        onMouseUp={(e) => {
          if (e.button === 0) {
            e.preventDefault();
            e.stopPropagation();
            console.log('ContextMenu: closing menu via overlay mouseup');
            onClose();
          }
        }}
        onClick={(e) => {
          e.preventDefault();
          e.stopPropagation();
        }}
        onContextMenu={(e) => {
          e.preventDefault();
          e.stopPropagation();
        }}
      />
      <div
        ref={menuRef}
        className="fixed z-[10001] min-w-48 bg-white/70 dark:bg-white/10 backdrop-blur-xl rounded-2xl 
                   shadow-2xl border border-white/50 dark:border-white/10 overflow-hidden cursor-default"
        style={{
          left: `${adjustedPosition.x}px`,
          top: `${adjustedPosition.y}px`,
          pointerEvents: 'auto',
        }}
        onContextMenu={(e) => {
          e.preventDefault();
          e.stopPropagation();
        }}
      >
        <div className="py-1">
          {menuItems.map((item, index) => (
            <button
              key={index}
              onMouseDown={(e) => {
                e.stopPropagation();
              }}
              onClick={() => {
                item.onClick();
                onClose();
              }}
              className={`
                w-full px-4 py-2.5 flex items-center gap-3 text-sm
                ${item.danger 
                  ? 'text-red-600 dark:text-red-400' 
                  : 'text-gray-700 dark:text-gray-200'
                }
              `}
            >
              <span className={item.danger ? 'text-red-500 dark:text-red-400' : 'text-gray-400 dark:text-gray-500'}>
                {item.icon}
              </span>
              <span className="font-medium">{item.label}</span>
            </button>
          ))}
        </div>
      </div>
    </>
  );
}
