import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { isTauri as checkIsTauri } from '@tauri-apps/api/core';
import type { ImageMetadata, ClipboardEvent } from '../types';

let isTauriCache: boolean | null = null;

async function isTauriEnvironment(): Promise<boolean> {
  if (isTauriCache === null) {
    isTauriCache = await checkIsTauri();
  }
  return isTauriCache;
}

export async function getImages(): Promise<ImageMetadata[]> {
  if (!(await isTauriEnvironment())) {
    return [];
  }
  return await invoke<ImageMetadata[]>('get_images');
}

export async function deleteImage(id: string): Promise<void> {
  if (!(await isTauriEnvironment())) {
    return;
  }
  return await invoke<void>('delete_image', { id });
}

export async function clearAllImages(): Promise<void> {
  if (!(await isTauriEnvironment())) {
    return;
  }
  return await invoke<void>('clear_all_images');
}

export async function resetClipboardHash(): Promise<void> {
  if (!(await isTauriEnvironment())) {
    return;
  }
  return await invoke<void>('reset_clipboard_hash');
}

export async function copyFileToClipboard(path: string): Promise<void> {
  if (!(await isTauriEnvironment())) {
    return;
  }
  return await invoke<void>('copy_file_to_clipboard', { path });
}

export async function saveImageFromClipboard(imageData: Uint8Array): Promise<ImageMetadata> {
  if (!(await isTauriEnvironment())) {
    throw new Error('Not in Tauri environment');
  }
  return await invoke<ImageMetadata>('save_image_from_clipboard', { imageData: Array.from(imageData) });
}

export async function cleanupOldImages(hours: number): Promise<void> {
  if (!(await isTauriEnvironment())) {
    return;
  }
  return await invoke<void>('cleanup_old_images', { hours });
}

export async function readImageFile(path: string): Promise<Uint8Array> {
  if (!(await isTauriEnvironment())) {
    throw new Error('Not in Tauri environment');
  }
  const data = await invoke<number[]>('read_image_file', { path });
  return new Uint8Array(data);
}

export function listenClipboardUpdate(callback: (event: ClipboardEvent) => void) {
  if (!(checkIsTauri())) {
    return Promise.resolve(() => {});
  }
  return listen<ClipboardEvent>('clipboard-update', (event) => callback(event.payload));
}
