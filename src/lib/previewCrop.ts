export interface NormalizedCrop {
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface CropRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface DisplayedImageLayout {
  left: number;
  top: number;
  width: number;
  height: number;
}

export function getDisplayedImageLayout(
  viewportWidth: number,
  viewportHeight: number,
  imageWidth: number,
  imageHeight: number,
): DisplayedImageLayout {
  if (viewportWidth <= 0 || viewportHeight <= 0 || imageWidth <= 0 || imageHeight <= 0) {
    return { left: 0, top: 0, width: viewportWidth, height: viewportHeight };
  }

  const scale = Math.min(viewportWidth / imageWidth, viewportHeight / imageHeight);
  const width = imageWidth * scale;
  const height = imageHeight * scale;
  return {
    left: (viewportWidth - width) / 2,
    top: (viewportHeight - height) / 2,
    width,
    height,
  };
}

export function clientToNormalizedCrop(
  clientX: number,
  clientY: number,
  viewportRect: DOMRect,
  layout: DisplayedImageLayout,
  clampToImage = false,
): { x: number; y: number } | null {
  const localX = clientX - viewportRect.left - layout.left;
  const localY = clientY - viewportRect.top - layout.top;
  if (!clampToImage && (localX < 0 || localY < 0 || localX > layout.width || localY > layout.height)) {
    return null;
  }
  const cx = clamp01(localX / layout.width);
  const cy = clamp01(localY / layout.height);
  return { x: cx, y: cy };
}

export function normalizeDragRect(
  x1: number,
  y1: number,
  x2: number,
  y2: number,
): NormalizedCrop {
  const x = Math.min(x1, x2);
  const y = Math.min(y1, y2);
  const w = Math.abs(x2 - x1);
  const h = Math.abs(y2 - y1);
  return {
    x: clamp01(x),
    y: clamp01(y),
    w: clamp01(Math.min(w, 1 - x)),
    h: clamp01(Math.min(h, 1 - y)),
  };
}

export function normalizedToPixelCrop(
  crop: NormalizedCrop,
  imageWidth: number,
  imageHeight: number,
): CropRect | null {
  if (crop.w < 0.01 || crop.h < 0.01 || imageWidth < 1 || imageHeight < 1) return null;

  const x = Math.min(Math.floor(crop.x * imageWidth), imageWidth - 1);
  const y = Math.min(Math.floor(crop.y * imageHeight), imageHeight - 1);
  const width = Math.max(1, Math.min(imageWidth - x, Math.ceil(crop.w * imageWidth)));
  const height = Math.max(1, Math.min(imageHeight - y, Math.ceil(crop.h * imageHeight)));

  return { x, y, width, height };
}

export function cropOverlayStyle(
  crop: NormalizedCrop,
  layout: DisplayedImageLayout,
  viewportRect: DOMRect,
): Record<string, string> {
  return {
    left: `${viewportRect.left + layout.left + crop.x * layout.width}px`,
    top: `${viewportRect.top + layout.top + crop.y * layout.height}px`,
    width: `${crop.w * layout.width}px`,
    height: `${crop.h * layout.height}px`,
  };
}

function clamp01(value: number): number {
  return Math.min(1, Math.max(0, value));
}
