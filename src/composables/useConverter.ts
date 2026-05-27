import { invoke } from "@tauri-apps/api/core";
import type {
  BatchResponse,
  ConvertResponse,
  CropRect,
  PreviewResponse,
  VtracerSettings,
} from "@/types/vtracer";
import { settingsToBackend } from "@/types/vtracer";

export function useConverter() {
  async function convertToSvg(
    inputPath: string,
    settings: VtracerSettings,
    outputPath?: string | null,
    crop?: CropRect | null,
  ): Promise<ConvertResponse> {
    return invoke<ConvertResponse>("convert_to_svg", {
      request: {
        inputPath,
        outputPath: outputPath ?? null,
        settings: settingsToBackend(settings),
        crop: crop ?? null,
      },
    });
  }

  async function convertPreview(
    inputPath: string,
    settings: VtracerSettings,
    crop?: CropRect | null,
  ): Promise<PreviewResponse> {
    return invoke<PreviewResponse>("convert_preview", {
      request: {
        inputPath,
        settings: settingsToBackend(settings),
        crop: crop ?? null,
      },
    });
  }

  async function convertBatch(
    inputPaths: string[],
    settings: VtracerSettings,
    outputMode: "same_dir" | "output_dir",
    outputDir?: string | null,
    preserveSubdirs = false,
  ): Promise<BatchResponse> {
    return invoke<BatchResponse>("convert_batch", {
      request: {
        inputPaths,
        settings: settingsToBackend(settings),
        outputMode,
        outputDir: outputDir ?? null,
        preserveSubdirs,
      },
    });
  }

  async function cancelBatch(): Promise<void> {
    await invoke("cancel_batch");
  }

  return { convertToSvg, convertPreview, convertBatch, cancelBatch };
}
