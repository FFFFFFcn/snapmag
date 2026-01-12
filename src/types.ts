export interface ImageMetadata {
  id: string;
  path: string;
  createdAt: number;
  ocrResult?: string;
}

export interface ClipboardEvent {
  imagePath: string;
}

export interface OcrResult {
  text: string;
  confidence: number;
}

export interface ContextMenuPosition {
  x: number;
  y: number;
}
