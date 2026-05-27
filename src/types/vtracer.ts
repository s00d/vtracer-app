export type ColorMode = "color" | "bw";
export type HierarchicalMode = "stacked" | "cutout";
export type CurveMode = "pixel" | "polygon" | "spline";
export type VtracerPreset = "bw" | "poster" | "photo" | "";
export type OutputMode = "same_dir" | "output_dir";
export type AppTab = "file" | "batch";
export type CompareViewMode = "sideBySide" | "slider" | "overlay";
export type ConversionMode = "single" | "batch";
export type HistoryStatus = "success" | "partial" | "error";
export type BatchItemStatus = "pending" | "running" | "done" | "error";

export interface VtracerSettings {
  preset: VtracerPreset;
  colormode: ColorMode;
  hierarchical: HierarchicalMode;
  mode: CurveMode;
  filterSpeckle: number;
  colorPrecision: number;
  gradientStep: number;
  cornerThreshold: number;
  segmentLength: number;
  spliceThreshold: number;
  pathPrecision: number;
  maxIterations: number;
  trimEmptySpace: boolean;
  svgoOptimize: boolean;
}

export interface ConvertResponse {
  inputPath?: string;
  outputPath: string;
  elapsedMs: number;
  outputSizeBytes: number;
  pathCount?: number;
}

export interface PreviewResponse {
  svgContent: string;
  width: number;
  height: number;
  pathCount: number;
  elapsedMs: number;
}

export interface ImageInfo {
  width: number;
  height: number;
  fileSizeBytes: number;
  mime: string;
}

export interface CropRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface ImageDataUrlResponse {
  dataUrl: string;
  width: number;
  height: number;
  fileSizeBytes: number;
}

export interface BatchFailedItem {
  path: string;
  error: string;
}

export interface BatchResponse {
  succeeded: ConvertResponse[];
  failed: BatchFailedItem[];
  totalElapsedMs: number;
}

export interface BatchProgressPayload {
  index: number;
  total: number;
  path: string;
  status: BatchItemStatus;
  message?: string;
}

export interface HistoryStats {
  elapsedMs: number;
  pathCount?: number;
  outputBytes?: number;
  inputBytes?: number;
}

export interface HistoryEntry {
  id: string;
  timestamp: number;
  mode: ConversionMode;
  inputPaths: string[];
  outputPaths: string[];
  settings: VtracerSettings;
  stats: HistoryStats;
  status: HistoryStatus;
}

export interface NamedPreset {
  id: string;
  name: string;
  settings: VtracerSettings;
  createdAt: number;
}

export interface BatchQueueItem {
  path: string;
  status: BatchItemStatus;
  outputPath?: string;
  error?: string;
  elapsedMs?: number;
  fileSizeBytes?: number;
  width?: number;
  height?: number;
}

export const IMAGE_EXTENSIONS = [
  "png",
  "jpg",
  "jpeg",
  "webp",
  "bmp",
  "gif",
  "tif",
  "tiff",
] as const;

export const DEFAULT_SETTINGS: VtracerSettings = {
  preset: "",
  colormode: "color",
  hierarchical: "stacked",
  mode: "spline",
  filterSpeckle: 4,
  colorPrecision: 6,
  gradientStep: 16,
  cornerThreshold: 60,
  segmentLength: 4,
  spliceThreshold: 45,
  pathPrecision: 2,
  maxIterations: 10,
  trimEmptySpace: true,
  svgoOptimize: false,
};

export function settingsFromBackend(raw: Record<string, unknown>): VtracerSettings {
  return {
    preset: ((raw.preset as string | null | undefined) ?? "") as VtracerPreset,
    colormode: (raw.colormode as ColorMode) ?? "color",
    hierarchical: (raw.hierarchical as HierarchicalMode) ?? "stacked",
    mode: (raw.mode as CurveMode) ?? "spline",
    filterSpeckle: Number(raw.filterSpeckle ?? 4),
    colorPrecision: Number(raw.colorPrecision ?? 6),
    gradientStep: Number(raw.gradientStep ?? 16),
    cornerThreshold: Number(raw.cornerThreshold ?? 60),
    segmentLength: Number(raw.segmentLength ?? 4),
    spliceThreshold: Number(raw.spliceThreshold ?? 45),
    pathPrecision: Number(raw.pathPrecision ?? 2),
    maxIterations: Number(raw.maxIterations ?? 10),
    trimEmptySpace: Boolean(raw.trimEmptySpace ?? true),
    svgoOptimize: Boolean(raw.svgoOptimize ?? false),
  };
}

export function settingsToBackend(settings: VtracerSettings) {
  return {
    preset: settings.preset || null,
    colormode: settings.colormode,
    hierarchical: settings.hierarchical,
    mode: settings.mode,
    filterSpeckle: settings.filterSpeckle,
    colorPrecision: settings.colorPrecision,
    gradientStep: settings.gradientStep,
    cornerThreshold: settings.cornerThreshold,
    segmentLength: settings.segmentLength,
    spliceThreshold: settings.spliceThreshold,
    pathPrecision: settings.pathPrecision,
    maxIterations: settings.maxIterations,
    trimEmptySpace: settings.trimEmptySpace,
    svgoOptimize: settings.svgoOptimize,
  };
}

export function applyVtracerPreset(settings: VtracerSettings, preset: VtracerPreset): VtracerSettings {
  const next = { ...settings, preset };
  if (preset === "bw") {
    return {
      ...next,
      colormode: "bw",
      hierarchical: "stacked",
      mode: "spline",
      filterSpeckle: 4,
      colorPrecision: 6,
      gradientStep: 16,
      cornerThreshold: 60,
      segmentLength: 4,
      spliceThreshold: 45,
      pathPrecision: 2,
      maxIterations: 10,
    };
  }
  if (preset === "poster") {
    return {
      ...next,
      colormode: "color",
      hierarchical: "stacked",
      mode: "spline",
      filterSpeckle: 4,
      colorPrecision: 8,
      gradientStep: 16,
      cornerThreshold: 60,
      segmentLength: 4,
      spliceThreshold: 45,
      pathPrecision: 2,
      maxIterations: 10,
    };
  }
  if (preset === "photo") {
    return {
      ...next,
      colormode: "color",
      hierarchical: "stacked",
      mode: "spline",
      filterSpeckle: 10,
      colorPrecision: 8,
      gradientStep: 48,
      cornerThreshold: 180,
      segmentLength: 4,
      spliceThreshold: 45,
      pathPrecision: 2,
      maxIterations: 10,
    };
  }
  return next;
}

export function isSupportedImage(path: string): boolean {
  const extension = path.split(".").pop()?.toLowerCase();
  return extension ? (IMAGE_EXTENSIONS as readonly string[]).includes(extension) : false;
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

export function fileName(path: string): string {
  return path.split(/[/\\]/).pop() ?? path;
}
